use crate::models::{GroupVersionKind, List, Resource, ClusterResource, GroupKind, NamespaceResource};
use crate::storage::{Storage, StorageResult};
use crate::storage_sqlite::SqliteStorage;

use actix_web::{get, HttpResponse, web, post, Error};
use actix_web::error::{ErrorNotFound, ErrorBadRequest};
use bytes::Buf;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use serde::Deserialize;


/*
/// This lists CustomResourceDefinitions registered in the server
/// The request will currently always be a "watch" on new resources
/// That means the result will be a never-ending HTTP response with newline-separated JSON objects
#[get("/apis/apiextensions.k8s.io/v1/customresourcedefinitions")]
pub async fn list_custom_resource_definitions(
    watch_register: web::Data<Sender<Sender<WatchEvent>>>,
    options: web::Query<ListOptions>,
    storage: web::Data<SqliteStorage>
) -> impl Responder {
    return if options.watch {
        // We're creating a new channel pair, the _sending_ end of which we send to the EventBroker
        // The receiving end will be given to the WatchStream which
        let (tx, rx) = mpsc::channel();
        let res = watch_register.send(tx);

        let body = WatchStream::new(rx);

        HttpResponse::Ok()
            .content_type("application/json")
            .streaming(body)
    } else {
        let objects = storage.list();
        HttpResponse::Ok()
    }
}
 */

#[derive(Debug, Deserialize)]
struct VersionedClusterResource {
    #[serde(flatten)]
    cluster_resource: ClusterResource,
    version: String
}

#[derive(Debug, Deserialize)]
struct VersionedNamespaceResource {
    #[serde(flatten)]
    namespace_resource: NamespaceResource,
    version: String
}


fn get_api(resource_type: &GroupKind, storage: &web::Data<SqliteStorage>) -> StorageResult<CustomResourceDefinition> {
    let resource_name = format!("{}.{}", resource_type.kind, resource_type.group);
    let key = ClusterResource::new("apiextensions.k8s.io".to_string(), "customresourcedefinitions".to_string(), resource_name);

    storage.get_cluster_resource(&key)
}


//
// Cluster Scoped Handlers
//
#[get("/apis/{group}/{version}/{kind}/{resource}")]
pub async fn get_cluster_resource(
    resource: web::Path<VersionedClusterResource>,
    storage: web::Data<SqliteStorage>,
    //event_sender: web::Data<Sender<WatchEvent>>,
    //registered_crds: web::Data<Arc<RwLock<HashSet<CrdCoordinates>>>>,
) -> Result<HttpResponse, Error> {
    println!("get_cluster_scoped_resource: {:?}", resource);
    let resource = resource.into_inner();
    // TODO: Check whether the requested API has been registered

    let crd = get_api(&resource.cluster_resource.group_kind, &storage)
        .map_err(|e| ErrorBadRequest(e))?
        .ok_or(ErrorNotFound("API does not exist"))?;


    let resource_name = resource.cluster_resource.resource.clone();
    // We clone the name here because we need the resource later on for sending it to the event bus
    let option: StorageResult<Resource> = storage.get_cluster_resource(&resource.cluster_resource);

    Ok(HttpResponse::Ok().json(option.unwrap()))
}


/// This function handles all GET (LIST) requests for resources that are Cluster scoped.
#[get("/apis/{group}/{version}/{kind}")]
pub async fn list_cluster_resources(
    gvk: web::Path<GroupVersionKind>,
    storage: web::Data<SqliteStorage>,
) -> Result<HttpResponse, Error> {
    println!("list_cluster_scoped_resource_type: {:?}", gvk); // TODO: Logging
    let gvk = gvk.into_inner();

    // First we need to check whether the requested API exists at all
    let group_version = gvk.group_version();
    let gk = GroupKind::from(gvk);

    let crd = get_api(&gk, &storage)
        .map_err(|e| ErrorBadRequest(e))?
        .ok_or(ErrorNotFound("foo"))?;

    let resources_list: List<Resource> = List {
        api_version: group_version,
        items: storage.list_cluster_resources(&gk),
        kind: crd.spec.names.kind,
        metadata: Default::default(),
    };
    Ok(HttpResponse::Ok().json(resources_list))
}


// TODO: We need to validate the JSON to see whether names etc. are all valid URLs
#[post("/apis/{group}/{version}/{kind}")]
pub async fn create_cluster_resource(
    gvk: web::Path<GroupVersionKind>,
    storage: web::Data<SqliteStorage>,
    bytes: web::Bytes,
    //event_sender: web::Data<Sender<WatchEvent>>,
    //registered_crds: web::Data<Arc<RwLock<HashSet<CrdCoordinates>>>>,
) -> Result<HttpResponse, Error> {
    println!("create_cluster_scoped_resource: {:?}", gvk);
    let gvk = gvk.into_inner();

    let gk = GroupKind::from(gvk);
    let crd = get_api(&gk, &storage) // TOOD: Move the error handling to the get_api method
        .map_err(|e| ErrorBadRequest(e))?
        .ok_or(ErrorNotFound("CRD missing"))?;


    let byte_array = bytes.bytes();
    let resource: Resource = serde_json::from_slice(byte_array)?;
    // We clone the name here because we need the resource later on for sending it to the event bus
    let cluster_resource = ClusterResource::from((gk, resource.metadata.name.ok_or(ErrorBadRequest("metadata.name is empty".to_string()))?.clone()));
    storage.create_cluster_resource(&cluster_resource, byte_array);

    Ok(HttpResponse::Ok().finish())

    /*
    if !registered_crds.read().unwrap().contains(&coordinates) {
        return Ok(HttpResponse::NotFound().finish());
    }
    event_sender.send(WatchEvent::ADDED(crd));
     */
}



//
// Namespace Scoped Handlers
//

#[get("/apis/{group}/{version}/namespaces/{namespace}/{kind}/{resource}")]
pub async fn get_namespaced_resource(
    resource: web::Path<VersionedNamespaceResource>,
    storage: web::Data<SqliteStorage>,
    //event_sender: web::Data<Sender<WatchEvent>>,
    //registered_crds: web::Data<Arc<RwLock<HashSet<CrdCoordinates>>>>,
) -> Result<HttpResponse, Error> {
    println!("get_namespaced_resource: {:?}", resource);
    let resource = resource.into_inner();

    // TODO: Check whether the requested API has been registered
    let crd = get_api(&resource.namespace_resource.group_namespace_kind.group_kind, &storage)
        .map_err(|e| ErrorBadRequest(e))?
        .ok_or(ErrorNotFound("API does not exist"))?;


    // We clone the name here because we need the resource later on for sending it to the event bus
    let resource_name = resource.namespace_resource.resource.clone();

    let option: StorageResult<Resource> = storage.get_namespace_resource(&resource.namespace_resource);

    Ok(HttpResponse::Ok().json(option.unwrap()))
}


/*
/// This function handles all GET (LIST) requests for resources that are Namespace scoped.
#[get("/apis/{group}/{version}/namespaces/{namespace}/{kind}")]
pub async fn list_namespace_scoped_kind(
    gvnk: web::Path<GroupVersionNamespaceKind>,
    storage: web::Data<SqliteStorage>,
) -> Result<HttpResponse, Error> {
    println!("list_namespace_scoped_kind {:?}", gvnk); // TODO: Logging

    let gvnk = gvnk.into_inner();

    let crd = get_api(&gvnk.group_version_kind, &storage).ok_or(ErrorNotFound("foo"))?;

    let resources_list: List<Resource> = List {
        api_version: gvnk.group_version_kind.group_version(),
        items: storage.list_namespaced(&::from(gvnk)),
        kind: crd.spec.names.kind,
        metadata: Default::default(),
    };
    Ok(HttpResponse::Ok().json(resources_list))
}
   */
/*
// TODO: We need to validate the JSON to see whether names etc. are all valid URLs
#[post("/apis/{group}/{version}/{kind}")]
pub async fn create_cluster_scoped_resource(
    gvk: web::Path<GroupVersionKind>,
    storage: web::Data<SqliteStorage>,
    bytes: web::Bytes,
    //event_sender: web::Data<Sender<WatchEvent>>,
    //registered_crds: web::Data<Arc<RwLock<HashSet<CrdCoordinates>>>>,
) -> Result<HttpResponse, Error> {
    println!("create_cluster_scoped_resource: {:?}", gvk);
    let gvk = gvk.into_inner();

    let crd = get_api(&gvk, &storage).ok_or(ErrorNotFound("Crd missing"))?;

    let byte_array = bytes.bytes();
    let resource: Resource = serde_json::from_slice(byte_array)?;
    // We clone the name here because we need the resource later on for sending it to the event bus
    storage.create(&StorageKind::from(gvk), resource.metadata.name.clone().unwrap(), byte_array);

    Ok(HttpResponse::Ok().finish())

    /*
    if !registered_crds.read().unwrap().contains(&coordinates) {
        return Ok(HttpResponse::NotFound().finish());
    }
    event_sender.send(WatchEvent::ADDED(crd));
     */
}

// TODO: Finish
#[get("/apis/{group}/{version}/{kind}/{resource}")]
pub async fn get_cluster_scoped_resource(
    resource: web::Path<ClusterResource>,
    storage: web::Data<SqliteStorage>,
    //event_sender: web::Data<Sender<WatchEvent>>,
    //registered_crds: web::Data<Arc<RwLock<HashSet<CrdCoordinates>>>>,
) -> Result<HttpResponse, Error> {
    println!("get_cluster_scoped_resource: {:?}", resource);
    let resource = resource.into_inner();
    // TODO: Check whether the requested API has been registered

    let crd = get_api(&resource.group_version_kind, &storage).unwrap();

    let resource_name = resource.resource.clone();
    // We clone the name here because we need the resource later on for sending it to the event bus
    let option: Option<Resource> = storage.get(&StorageKind::from(resource.group_version_kind), &resource_name);

    Ok(HttpResponse::Ok().json(option.unwrap()))
}
*/
