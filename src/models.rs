use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, ListMeta};
use serde_json::Value;
use serde::{Deserialize, Serialize};

// Derived traits:
// * Deserialize is needed for Actix to work when the struct is used in an Extractor
pub trait Group {
    fn group(&self) -> &str;
}
pub trait Namespace {
    fn namespace(&self) -> &str;
}
pub trait Version {
    fn version(&self) -> &str;
}
pub trait ResourceType {
    fn resource_type(&self) -> &str;
}
pub trait Resource {
    fn resource(&self) -> &str;
}

//
// GroupResourceType
//
#[derive(Debug, Deserialize)]
pub struct GroupResourceType {
    group: String,
    resource_type: String
}

impl GroupResourceType {
    pub fn new(group: String, resource_type: String) -> GroupResourceType {
        GroupResourceType {
            group,
            resource_type
        }
    }
}

impl Group for GroupResourceType {
    fn group(&self) -> &str {
        &self.group
    }
}
impl ResourceType for GroupResourceType {
    fn resource_type(&self) -> &str {
        &self.resource_type
    }
}


impl From<GroupVersionResourceType> for GroupResourceType {
    fn from(gvk: GroupVersionResourceType) -> Self {
        GroupResourceType {
            group: gvk.group,
            resource_type: gvk.resource_type
        }
    }
}

//
// GroupResourceTypeResource
//
#[derive(Debug, Deserialize)]
pub struct GroupResourceTypeResource {
    group: String,
    resource_type: String,
    resource: String
}

impl GroupResourceTypeResource {
    pub fn new(group: String, resource_type: String, resource: String) -> GroupResourceTypeResource {
        GroupResourceTypeResource {
            group,
            resource_type,
            resource
        }
    }
}

impl Group for GroupResourceTypeResource {
    fn group(&self) -> &str {
        &self.group
    }
}
impl ResourceType for GroupResourceTypeResource {
    fn resource_type(&self) -> &str {
        &self.resource_type
    }
}
impl Resource for GroupResourceTypeResource {
    fn resource(&self) -> &str {
        &self.resource
    }
}

//
// GroupNamespaceResourceType
//
#[derive(Debug, Deserialize)]
pub struct GroupNamespaceResourceType {
    group: String,
    namespace: String,
    resource_type: String
}

impl Group for GroupNamespaceResourceType {
    fn group(&self) -> &str {
        &self.group
    }
}
impl Namespace for GroupNamespaceResourceType {
    fn namespace(&self) -> &str {
        &self.namespace
    }
}
impl ResourceType for GroupNamespaceResourceType {
    fn resource_type(&self) -> &str {
        &self.resource_type
    }
}


//
// GroupVersionResourceType
//
#[derive(Debug, Deserialize)]
pub struct GroupVersionResourceType {
    group: String,
    version: String,
    resource_type: String,
}

impl Group for GroupVersionResourceType {
    fn group(&self) -> &str {
        &self.group
    }
}
impl Version for GroupVersionResourceType {
    fn version(&self) -> &str {
        &self.version
    }
}
impl ResourceType for GroupVersionResourceType {
    fn resource_type(&self) -> &str {
        &self.resource_type
    }
}

impl GroupVersionResourceType {
    pub fn group_version(&self) -> String {
        format!("{}/{}", self.group, self.version)
    }
}

//
// GroupNamespaceResourceTypeResource
//
#[derive(Debug, Deserialize)]
pub struct GroupNamespaceResourceTypeResource {
    group: String,
    namespace: String,
    resource_type: String,
    resource: String
}

impl Group for GroupNamespaceResourceTypeResource {
    fn group(&self) -> &str {
        &self.group
    }
}
impl Namespace for GroupNamespaceResourceTypeResource {
    fn namespace(&self) -> &str {
        &self.namespace
    }
}
impl ResourceType for GroupNamespaceResourceTypeResource {
    fn resource_type(&self) -> &str {
        &self.resource_type
    }
}
impl Resource for GroupNamespaceResourceTypeResource {
    fn resource(&self) -> &str {
        &self.resource
    }
}


//
// GroupVersionNamespaceResourceType
//
#[derive(Debug, Deserialize)]
pub struct GroupVersionNamespaceResourceType {
    group: String,
    version: String,
    namespace: String,
    resource_type: String
}

impl GroupVersionNamespaceResourceType {
    pub fn group_version(&self) -> String {
        format!("{}/{}", self.group, self.version)
    }
}

impl Group for GroupVersionNamespaceResourceType {
    fn group(&self) -> &str {
        &self.group
    }
}
impl Version for GroupVersionNamespaceResourceType {
    fn version(&self) -> &str {
        &self.version
    }
}
impl Namespace for GroupVersionNamespaceResourceType {
    fn namespace(&self) -> &str {
        &self.namespace
    }
}
impl ResourceType for GroupVersionNamespaceResourceType {
    fn resource_type(&self) -> &str {
        &self.resource_type
    }
}

//
// GroupVersionResourceTypeResource
//
#[derive(Debug, Deserialize)]
pub struct GroupVersionResourceTypeResource {
    group: String,
    version: String,
    resource_type: String,
    resource: String
}

impl Group for GroupVersionResourceTypeResource {
    fn group(&self) -> &str {
        &self.group
    }
}
impl Version for GroupVersionResourceTypeResource {
    fn version(&self) -> &str {
        &self.version
    }
}
impl ResourceType for GroupVersionResourceTypeResource {
    fn resource_type(&self) -> &str {
        &self.resource_type
    }
}
impl Resource for GroupVersionResourceTypeResource {
    fn resource(&self) -> &str {
        &self.resource
    }
}

//
// GroupVersionNamespaceResourceTypeResource
//
#[derive(Debug, Deserialize)]
pub struct GroupVersionNamespaceResourceTypeResource {
    group: String,
    version: String,
    namespace: String,
    resource_type: String,
    resource: String
}

impl Group for GroupVersionNamespaceResourceTypeResource {
    fn group(&self) -> &str {
        &self.group
    }
}
impl Version for GroupVersionNamespaceResourceTypeResource {
    fn version(&self) -> &str {
        &self.version
    }
}
impl Namespace for GroupVersionNamespaceResourceTypeResource {
    fn namespace(&self) -> &str {
        &self.namespace
    }
}
impl ResourceType for GroupVersionNamespaceResourceTypeResource {
    fn resource_type(&self) -> &str {
        &self.resource_type
    }
}
impl Resource for GroupVersionNamespaceResourceTypeResource {
    fn resource(&self) -> &str {
        &self.resource
    }
}





#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BaseResource {
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

impl Metadata for BaseResource {
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


pub struct ListOptions {
    watch: bool
}
