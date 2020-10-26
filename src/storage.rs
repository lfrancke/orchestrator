use crate::crd::Metadata;
use serde::de::DeserializeOwned;

// This can't be generic over some type because we don't know most types at compile-time.
// They will only be provided at runtime via CRDs.
pub trait Storage {
    // TODO: Should this be a Result instead?
    fn get<T>(&self, key: &StorageResourceType, resource_name: &str) -> Option<T>
        where T: Metadata + DeserializeOwned;

    fn list<T>(&self, key: &StorageResourceType) -> Vec<T>
        where T: Metadata + DeserializeOwned;

    fn create(&self, key: &StorageResourceType, name: String, obj: &[u8]); // TODO: obj should not be a &Bytes but what should it be? Probably a "CustomResource" or something like that
}

#[derive(Debug)]
pub enum StorageResourceType {
    ClusterScoped {
        group: String,
        name: String,
    },
    NamespaceScoped {
        group: String,
        name: String,
        namespace: String,
    },
}
