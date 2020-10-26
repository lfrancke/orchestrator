use crate::storage::{Storage, StorageResourceType};
use crate::storage_sqlite::SqliteStorage;

use actix_web::{get, HttpResponse, Responder, web, post, Error};
use bytes::Buf;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, ListMeta};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;


#[derive(Clone, Deserialize, Debug, Eq, Hash, PartialEq)]
pub struct ApiClusterScopedResourceType {
    api_group: String,
    api_version: String,
    resource_type_name: String,
}

#[derive(Clone, Deserialize, Debug, Eq, Hash, PartialEq)]
pub struct ApiNamespacedScopedResourceType {
    api_group: String,
    api_version: String,
    namespace: String,
    resource_type_name: String,
}


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Resource {
    pub metadata: ObjectMeta,

    #[serde(flatten)]
    pub remainder: Value,
}

/// A trait applied to all Kubernetes resources that have metadata.
/// NOTE: This is a copy of the k8s-openapi version with the restriction on "Resource" removed and
/// the Metadata object fixed to `ObjectMeta`.
/// We don't want the `Resource` restriction because it requires static knowledge/lifetime of strings which
/// we don't have because we load data at runtime.
/// Unless I'm mistaken that means we need to duplicate a few traits here.
pub trait Metadata {
    /// Gets a reference to the metadata of this resource value.
    fn metadata(&self) -> &ObjectMeta;

    /// Gets a mutable reference to the metadata of this resource value.
    fn metadata_mut(&mut self) -> &mut ObjectMeta;
}

impl Metadata for Resource {
    fn metadata(&self) -> &ObjectMeta {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut ObjectMeta {
        &mut self.metadata
    }
}

impl<T: k8s_openapi::Metadata<Ty=ObjectMeta>> Metadata for T {
    fn metadata(&self) -> &ObjectMeta {
        k8s_openapi::Metadata::metadata(self)
    }

    fn metadata_mut(&mut self) -> &mut ObjectMeta {
        k8s_openapi::Metadata::metadata_mut(self)
    }
}

// TODO: The upstream list object implements a custom Serializer, look at that
#[derive(Serialize)]
pub struct List<T> {
    #[serde(rename = "apiVersion")]
    pub api_version: String,

    /// List of objects.
    pub items: Vec<T>,

    pub kind: String,

    /// Standard list metadata. More info: https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#types-kinds
    pub metadata: ListMeta,
}


struct ListOptions {
    watch: bool
}

impl From<ApiClusterScopedResourceType> for StorageResourceType {
    fn from(item: ApiClusterScopedResourceType) -> Self {
        StorageResourceType::ClusterScoped {
            group: item.api_group,
            name: item.resource_type_name,
        }
    }
}


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


//
// Cluster Scoped Handlers
//

/// This function handles all LIST requests for resources that are Cluster scoped.
#[get("/apis/{api_group}/{api_version}/{resource_type_name}")]
pub async fn list_cluster_scoped_resource_type(
    resource_type: web::Path<ApiClusterScopedResourceType>,
    storage: web::Data<SqliteStorage>,
) -> impl Responder {
    println!("list_cluster_scoped_resource_type: {:?}", resource_type);

    // First we need to check whether the requested API exists at all
    // TODO: This whole check needs to be moved into a helper (macro?)
    let key = StorageResourceType::ClusterScoped {
        group: "apiextensions.k8s.io".to_string(),
        name: "customresourcedefinitions".to_string(),
    };

    let crd: Option<CustomResourceDefinition> = storage.get(&key, &format!("{}.{}", &resource_type.resource_type_name, &resource_type.api_group));
    // TODO: There must be a more elegant way to do this and at the same time avoid nesting too deply
    if let None = crd {
        return HttpResponse::NotFound().finish()
    }
    let crd = crd.unwrap();

    let resource_type = resource_type.into_inner();
    //let kind = resource_type.resource_type_name.clone();
    let kind = crd.spec.names.kind;
    let api_version = format!("{}/{}", resource_type.api_group, resource_type.api_version);
    let resources: Vec<Resource> = storage.list(&StorageResourceType::from(resource_type));
    let resources_list = List {
        api_version,
        items: resources,
        kind,
        metadata: Default::default(),
    };
    let foo = HttpResponse::Ok().json(resources_list);
    return foo;
}


// TODO: We need to validate the JSON to see whether names etc. are all valid URLs
#[post("/apis/{api_group}/{api_version}/{resource_type_name}")]
pub async fn create_cluster_scoped_resource(
    resource_type: web::Path<ApiClusterScopedResourceType>,
    storage: web::Data<SqliteStorage>,
    bytes: web::Bytes,
    //event_sender: web::Data<Sender<WatchEvent>>,
    //registered_crds: web::Data<Arc<RwLock<HashSet<CrdCoordinates>>>>,
) -> Result<HttpResponse, Error> {
    println!("create_cluster_scoped_resource: {:?}", resource_type);
    // TODO: Check whether the requested API does exist (i.e. whether a CustomResourceDefinition object for it exists)

    let byte_array = bytes.bytes();
    let resource: Resource = serde_json::from_slice(byte_array)?;
    // We clone the name here because we need the resource later on for sending it to the event bus
    storage.create(&StorageResourceType::from(resource_type.into_inner()), resource.metadata.name.clone().unwrap(), byte_array);

    Ok(HttpResponse::Ok().finish())

    /*
    if !registered_crds.read().unwrap().contains(&coordinates) {
        return Ok(HttpResponse::NotFound().finish());
    }
    event_sender.send(WatchEvent::ADDED(crd));
     */
}

// TODO: Finish
#[get("/apis/{api_group}/{api_version}/{resource_type_name}/{resource_name}")]
pub async fn get_cluster_scoped_resource(
    resource_type: web::Path<ApiClusterScopedResourceType>,
    resource_name: web::Path<String>,
    storage: web::Data<SqliteStorage>,
    bytes: web::Bytes,
    //event_sender: web::Data<Sender<WatchEvent>>,
    //registered_crds: web::Data<Arc<RwLock<HashSet<CrdCoordinates>>>>,
) -> Result<HttpResponse, Error> {
    println!("create_cluster_scoped_resource: {:?}", resource_type);
    // TODO: Check whether the requested API has been registered

    let byte_array = bytes.bytes();
    let resource: Resource = serde_json::from_slice(byte_array)?;
    // We clone the name here because we need the resource later on for sending it to the event bus
    storage.create(&StorageResourceType::from(resource_type.into_inner()), resource.metadata.name.clone().unwrap(), byte_array);

    Ok(HttpResponse::Ok().finish())

    /*
    if !registered_crds.read().unwrap().contains(&coordinates) {
        return Ok(HttpResponse::NotFound().finish());
    }
    event_sender.send(WatchEvent::ADDED(crd));
     */
}
