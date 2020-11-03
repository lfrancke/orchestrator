use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, ListMeta};
use serde_json::Value;
use serde::{Deserialize, Serialize};

// Derived traits:
// * Deserialize is needed for Actix to work when the struct is used in an Extractor
pub trait Group {
    fn group(&self) -> &str;
}
pub trait Kind {
    fn kind(&self) -> &str;
}
pub trait Namespace {
    fn namespace(&self) -> &str;
}
pub trait Version {
    fn version(&self) -> &str;
}
pub trait Resource {
    fn resource(&self) -> &str;
}

//
// GroupKind
//
#[derive(Debug, Deserialize)]
pub struct GroupKind {
    group: String,
    kind: String
}

impl GroupKind {
    pub fn new(group: String, kind: String) -> GroupKind {
        GroupKind {
            group,
            kind
        }
    }
}

impl Group for GroupKind {
    fn group(&self) -> &str {
        &self.group
    }
}
impl Kind for GroupKind {
    fn kind(&self) -> &str {
        &self.kind
    }
}


impl From<GroupVersionKind> for GroupKind {
    fn from(gvk: GroupVersionKind) -> Self {
        GroupKind {
            group: gvk.group,
            kind: gvk.kind
        }
    }
}

//
// GroupKindResource
//
#[derive(Debug, Deserialize)]
pub struct GroupKindResource {
    group: String,
    kind: String,
    resource: String
}

impl GroupKindResource {
    pub fn new(group: String, kind: String, resource: String) -> GroupKindResource {
        GroupKindResource {
            group,
            kind,
            resource
        }
    }
}

impl Group for GroupKindResource {
    fn group(&self) -> &str {
        &self.group
    }
}
impl Kind for GroupKindResource {
    fn kind(&self) -> &str {
        &self.kind
    }
}
impl Resource for GroupKindResource {
    fn resource(&self) -> &str {
        &self.resource
    }
}

//
// GroupNamespaceKind
//
#[derive(Debug, Deserialize)]
pub struct GroupNamespaceKind {
    group: String,
    namespace: String,
    kind: String
}

impl Group for GroupNamespaceKind {
    fn group(&self) -> &str {
        &self.group
    }
}
impl Namespace for GroupNamespaceKind {
    fn namespace(&self) -> &str {
        &self.namespace
    }
}
impl Kind for GroupNamespaceKind {
    fn kind(&self) -> &str {
        &self.kind
    }
}


//
// GroupVersionKind
//
#[derive(Debug, Deserialize)]
pub struct GroupVersionKind {
    group: String,
    version: String,
    kind: String,
}

impl Group for GroupVersionKind {
    fn group(&self) -> &str {
        &self.group
    }
}
impl Version for GroupVersionKind {
    fn version(&self) -> &str {
        &self.version
    }
}
impl Kind for GroupVersionKind {
    fn kind(&self) -> &str {
        &self.kind
    }
}

impl GroupVersionKind {
    pub fn group_version(&self) -> String {
        format!("{}/{}", self.group, self.version)
    }
}

//
// GroupNamespaceKindResource
//
#[derive(Debug, Deserialize)]
pub struct GroupNamespaceKindResource {
    group: String,
    namespace: String,
    kind: String,
    resource: String
}

impl Group for GroupNamespaceKindResource {
    fn group(&self) -> &str {
        &self.group
    }
}
impl Namespace for GroupNamespaceKindResource {
    fn namespace(&self) -> &str {
        &self.namespace
    }
}
impl Kind for GroupNamespaceKindResource {
    fn kind(&self) -> &str {
        &self.kind
    }
}
impl Resource for GroupNamespaceKindResource {
    fn resource(&self) -> &str {
        &self.resource
    }
}


//
// GroupVersionNamespaceKind
//
#[derive(Debug, Deserialize)]
pub struct GroupVersionNamespaceKind {
    group: String,
    version: String,
    namespace: String,
    kind: String
}

impl Group for GroupVersionNamespaceKind {
    fn group(&self) -> &str {
        &self.group
    }
}
impl Version for GroupVersionNamespaceKind {
    fn version(&self) -> &str {
        &self.version
    }
}
impl Namespace for GroupVersionNamespaceKind {
    fn namespace(&self) -> &str {
        &self.namespace
    }
}
impl Kind for GroupVersionNamespaceKind {
    fn kind(&self) -> &str {
        &self.kind
    }
}

//
// GroupVersionKindResource
//
#[derive(Debug, Deserialize)]
pub struct GroupVersionKindResource {
    group: String,
    version: String,
    kind: String,
    resource: String
}

impl Group for GroupVersionKindResource {
    fn group(&self) -> &str {
        &self.group
    }
}
impl Version for GroupVersionKindResource {
    fn version(&self) -> &str {
        &self.version
    }
}
impl Kind for GroupVersionKindResource {
    fn kind(&self) -> &str {
        &self.kind
    }
}
impl Resource for GroupVersionKindResource {
    fn resource(&self) -> &str {
        &self.resource
    }
}

//
// GroupVersionNamespaceKindResource
//
#[derive(Debug, Deserialize)]
pub struct GroupVersionNamespaceKindResource {
    group: String,
    version: String,
    namespace: String,
    kind: String,
    resource: String
}

impl Group for GroupVersionNamespaceKindResource {
    fn group(&self) -> &str {
        &self.group
    }
}
impl Version for GroupVersionNamespaceKindResource {
    fn version(&self) -> &str {
        &self.version
    }
}
impl Namespace for GroupVersionNamespaceKindResource {
    fn namespace(&self) -> &str {
        &self.namespace
    }
}
impl Kind for GroupVersionNamespaceKindResource {
    fn kind(&self) -> &str {
        &self.kind
    }
}
impl Resource for GroupVersionNamespaceKindResource {
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

/*
impl From<GroupVersionKind> for StorageKind {
    fn from(item: GroupVersionKind) -> Self {
        StorageKind::ClusterScoped {
            group: item.group,
            kind: item.kind,
        }
    }
}

impl From<ClusterResource> for StorageKind {
    fn from(item: ClusterResource) -> Self {
        StorageKind::ClusterScoped {
            group: item.group_version_kind.group,
            kind: item.group_version_kind.kind,
        }
    }
}


 */
