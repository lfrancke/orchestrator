use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, ListMeta};
use serde_json::Value;
use serde::{Deserialize, Serialize};

// Derived traits:
// * Deserialize is needed for Actix to work when the struct is used in an Extractor


#[derive(Debug, Deserialize)]
pub struct GroupKind {
    pub group: String,
    pub kind: String
}

impl From<GroupVersionKind> for GroupKind {
    fn from(gvk: GroupVersionKind) -> Self {
        GroupKind {
            group: gvk.group,
            kind: gvk.kind
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GroupNamespaceKind {
    #[serde(flatten)]
    pub group_kind: GroupKind,
    pub namespace: String
}



#[derive(Debug, Deserialize)]
pub struct GroupVersionKind {
    pub group: String,
    pub version: String,
    pub kind: String,
}

impl GroupVersionKind {
    pub fn group_version(&self) -> String {
        format!("{}/{}", self.group, self.version)
    }
}

#[derive(Debug, Deserialize)]
pub struct GroupVersionNamespaceKind {
    #[serde(flatten)]
    pub group_version_kind: GroupVersionKind,
    pub namespace: String
}

#[derive(Debug, Deserialize)]
pub struct ClusterResource {
    #[serde(flatten)]
    pub group_kind: GroupKind,
    pub resource: String
}

impl ClusterResource {
    pub fn new(group: String, kind: String, resource: String) -> ClusterResource {
        ClusterResource {
            group_kind: GroupKind {
                group,
                kind
            },
            resource
        }
    }
}

impl From<(GroupKind, String)> for ClusterResource {
    fn from(incoming: (GroupKind, String)) -> Self {
        ClusterResource {
            group_kind: incoming.0,
            resource: incoming.1
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct NamespaceResource {
    #[serde(flatten)]
    pub group_namespace_kind: GroupNamespaceKind,
    pub resource: String
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
