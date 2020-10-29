use serde::de::DeserializeOwned;
use crate::models::{Metadata, GroupNamespaceKind, GroupKind};

// This can't be generic over some type because we don't know most types at compile-time.
// They will only be provided at runtime via CRDs.
pub trait Storage {
    // TODO: Should this be a Result<Option<T>> instead?
    fn get_cluster_resource<T>(&self, key: &GroupKind, resource_name: &str) -> Option<T>
        where T: Metadata + DeserializeOwned;

    fn list_cluster_kinds<T>(&self, key: &StorageKind) -> Vec<T>
        where T: Metadata + DeserializeOwned;

    fn list_namespaced<T>(&self, key: &GroupNamespaceKind) -> Vec<T>
        where T: Metadata + DeserializeOwned;

    fn create(&self, key: &StorageKind, name: String, obj: &[u8]); // TODO: obj should not be a &[u8] but what should it be? Probably a "Resource" or something like that
}

/// This encapsulates the coordinates to a Kind (i.e. "resource type", e.g. "Node", "Kafka")
/// but not to an instance of a Kind (i.e. "Resource").
/// It can either be namespaced or not.
#[derive(Debug)]
pub enum StorageKind {
    ClusterScoped {
        group: String,
        kind: String,
    },
    NamespaceScoped {
        group: String,
        name: String,
        namespace: String,
    },
}
