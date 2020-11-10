use crate::models::{GroupVersionResourceType, List, BaseResource, Group, GroupResourceTypeResource, GroupVersionResourceTypeResource, GroupVersionNamespaceResourceTypeResource, GroupVersionNamespaceResourceType, ResourceType, Version, ListOptions, GroupResourceType};
use crate::storage::{Storage, StorageResult};
use crate::storage_sqlite::SqliteStorage;

use actix_web::{get, HttpResponse, web, post, Error};
use actix_web::error::{ErrorNotFound, ErrorBadRequest};
use bytes::Buf;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use crate::helper::{get_crd_for_resource, get_crd_resource_type};
use std::sync::mpsc;
use crate::watch::{WatchEvent, WatchStream, WrappedWatchEvent};
use std::sync::mpsc::Sender;


// TODO: Make this return an Actix Error
fn get_crd_object<T>(crd: &T, storage: &web::Data<SqliteStorage>) -> StorageResult<CustomResourceDefinition>
    where T: Group + ResourceType + Version
{
    // The "name" of a CRD is its resource type and group separated by a dot.
    let resource_name = format!("{}.{}", crd.resource_type(), crd.group());
    let key = get_crd_for_resource(resource_name);

    let result: Option<CustomResourceDefinition> = storage.get_cluster_resource(&key)?;
    match result {
        None => { Ok(None) }
        Some(storage_crd) => {
            if storage_crd.spec.versions.iter().any(|crd_version| crd_version.name == crd.version()) {
                Ok(Some(storage_crd))
            } else {
                Ok(None)
            }
        }
    }
}


//
// CRD APIs
//
#[get("/apis/apiextensions.k8s.io/v1/customresourcedefinitions/{resource}")]
pub async fn get_crd(
    resource: web::Path<String>,
    storage: web::Data<SqliteStorage>,
) -> Result<HttpResponse, Error> {
    let crd = get_crd_for_resource(resource.into_inner());
    let option: StorageResult<BaseResource> = storage.get_cluster_resource(&crd);
    Ok(HttpResponse::Ok().json(option.unwrap()))
}

#[get("/apis/apiextensions.k8s.io/v1/customresourcedefinitions")]
pub async fn list_crds(
    storage: web::Data<SqliteStorage>,
) -> Result<HttpResponse, Error> {
    let resources_list: List<CustomResourceDefinition> = List {
        api_version: "apiextensions.k8s.io/v1".to_string(),
        items: storage.list_cluster_resources(&get_crd_resource_type()),
        kind: "CustomResourceDefinition".to_string(),
        metadata: Default::default(),
    };
    Ok(HttpResponse::Ok().json(resources_list))
}

#[post("/apis/apiextensions.k8s.io/v1/customresourcedefinitions")]
pub async fn create_crd(
    storage: web::Data<SqliteStorage>,
    bytes: web::Bytes,
    //event_sender: web::Data<Sender<WatchEvent>>,
    //registered_crds: web::Data<Arc<RwLock<HashSet<CrdCoordinates>>>>,
) -> Result<HttpResponse, Error> {
    let resource: BaseResource = serde_json::from_slice(bytes.bytes())?;

    let crd = get_crd_for_resource(resource.metadata.name.ok_or(ErrorBadRequest("metadata.name is empty".to_string()))?.clone());
    storage.create_cluster_resource(&crd, bytes.bytes());

    Ok(HttpResponse::Ok().finish())
}


//
// Cluster Scoped Handlers
//
#[get("/apis/{group}/{version}/{resource_type}/{resource}")]
pub async fn get_cluster_resource(
    resource: web::Path<GroupVersionResourceTypeResource>,
    storage: web::Data<SqliteStorage>,
) -> Result<HttpResponse, Error> {
    println!("get_cluster_resource: {:?}", resource);
    let resource = resource.into_inner();

    let crd = get_crd_object(&resource, &storage)
        .map_err(|e| ErrorBadRequest(e))?
        .ok_or(ErrorNotFound("API does not exist"))?;

    let option: StorageResult<BaseResource> = storage.get_cluster_resource(&resource);

    Ok(HttpResponse::Ok().json(option.unwrap()))
}


/// This function handles all GET (LIST) requests for resources that are Cluster scoped.
#[get("/apis/{group}/{version}/{resource_type}")]
pub async fn list_cluster_resources(
    gvrt: web::Path<GroupVersionResourceType>,
    watch_register: web::Data<Sender<(GroupResourceType, Sender<WatchEvent>)>>,
    web::Query(options): web::Query<ListOptions>,
    storage: web::Data<SqliteStorage>,
) -> Result<HttpResponse, Error> {
    println!("list_cluster_scoped_resource_type: {:?}", gvrt); // TODO: Logging
    let gvrt = gvrt.into_inner();

    let crd = get_crd_object(&gvrt, &storage)
        .map_err(|e| ErrorBadRequest(e))?
        .ok_or(ErrorNotFound("API does not exist"))?;

    match options.watch {
        Some(true) => {
            // We're creating a new channel pair, the _sending_ end of which we send to the EventBroker
            // The receiving end will be given to the WatchStream which will stream the received
            // changes back to the calling client
            let (tx, rx) = mpsc::channel();
            let res = watch_register.send(
                (GroupResourceType::new(
                    gvrt.group().to_string(),
                    gvrt.resource_type().to_string()
                ),
                 tx));

            let body = WatchStream::new(rx);

            Ok(HttpResponse::Ok()
                .content_type("application/json")
                .streaming(body))
        }
        _ => {
            let resources_list: List<BaseResource> = List {
                api_version: gvrt.group_version(),
                items: storage.list_cluster_resources(&gvrt),
                kind: crd.spec.names.kind,
                metadata: Default::default(),
            };
            Ok(HttpResponse::Ok().json(resources_list))
        }
    }
}


// TODO: We need to validate the JSON to see whether names etc. are all valid URLs
#[post("/apis/{group}/{version}/{resource_type}")]
pub async fn create_cluster_resource(
    web::Path(gvrt): web::Path<GroupVersionResourceType>,
    storage: web::Data<SqliteStorage>,
    bytes: web::Bytes,
    event_sender: web::Data<Sender<WrappedWatchEvent>>,
) -> Result<HttpResponse, Error> {
    println!("create_cluster_resource: {:?}", gvrt);

    let crd = get_crd_object(&gvrt, &storage)
        .map_err(|e| ErrorBadRequest(e))?
        .ok_or(ErrorNotFound("API does not exist"))?;

    let resource: BaseResource = serde_json::from_slice(bytes.bytes())?;
    // We clone the name here because we need the resource later on for sending it to the event bus
    let cluster_resource = GroupResourceTypeResource::new(
        gvrt.group().to_string(),
        gvrt.resource_type().to_string(),
        resource.metadata.name.clone().ok_or(ErrorBadRequest("metadata.name is empty".to_string()))?,
    );
    storage.create_cluster_resource(&cluster_resource, bytes.bytes());

    let event = WrappedWatchEvent::new(
        WatchEvent::ADDED(resource),
        GroupResourceType::new(
            gvrt.group().to_string(),
            gvrt.resource_type().to_string()
        )
    );

    event_sender.into_inner().send(event);

    Ok(HttpResponse::Ok().finish())
}


//
// Namespace Scoped Handlers
//
#[get("/apis/{group}/{version}/namespaces/{namespace}/{resource_type}/{resource}")]
pub async fn get_namespaced_resource(
    web::Path(resource): web::Path<GroupVersionNamespaceResourceTypeResource>,
    storage: web::Data<SqliteStorage>,
    //event_sender: web::Data<Sender<WatchEvent>>,
    //registered_crds: web::Data<Arc<RwLock<HashSet<CrdCoordinates>>>>,
) -> Result<HttpResponse, Error> {
    println!("get_namespaced_resource: {:?}", resource);

    // TODO: Check whether the requested API has been registered
    let crd = get_crd_object(&resource, &storage)
        .map_err(|e| ErrorBadRequest(e))?
        .ok_or(ErrorNotFound("API does not exist"))?;

    let option: StorageResult<BaseResource> = storage.get_namespace_resource(&resource);

    Ok(HttpResponse::Ok().json(option.unwrap()))
}


/// This function handles all GET (LIST) requests for resources that are Namespace scoped.
#[get("/apis/{group}/{version}/namespaces/{namespace}/{resource_type}")]
pub async fn list_namespaced_resources(
    gvnrt: web::Path<GroupVersionNamespaceResourceType>,
    storage: web::Data<SqliteStorage>,
) -> Result<HttpResponse, Error> {
    println!("list_namespaced_resources {:?}", gvnrt); // TODO: Logging
    let gvnk = gvnrt.into_inner();

    let crd = get_crd_object(&gvnk, &storage)
        .map_err(|e| ErrorBadRequest(e))?
        .ok_or(ErrorNotFound("API does not exist"))?;

    let items = storage.list_namespace_resources(&gvnk).unwrap().unwrap();

    let resources_list: List<BaseResource> = List {
        api_version: gvnk.group_version(),
        items,
        kind: crd.spec.names.kind,
        metadata: Default::default(),
    };
    Ok(HttpResponse::Ok().json(resources_list))
}
