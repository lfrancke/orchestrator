use crate::models::{Metadata, GroupKind, ClusterResource, NamespaceResource};

use serde::de::DeserializeOwned;
use thiserror::Error;

// This can't be generic over some type because we don't know most types at compile-time.
// They will only be provided at runtime via CRDs.
pub trait Storage {
    // TODO: Should this be a Result<Option<T>> instead?
    fn get_cluster_resource<T>(&self, key: &ClusterResource) -> StorageResult<T>
        where T: Metadata + DeserializeOwned;

    fn list_cluster_resources<T>(&self, key: &GroupKind) -> Vec<T>
        where T: Metadata + DeserializeOwned;

    fn create_cluster_resource(&self, key: &ClusterResource, obj: &[u8]) -> StorageResult<()>; // TODO: obj should not be a &[u8] but what should it be? Probably a "Resource" or something like that

    fn get_namespace_resource<T>(&self, key: &NamespaceResource) -> StorageResult<T>
        where T: Metadata + DeserializeOwned;
/*
    fn list_namespace_resources<T>(&self, key: &GroupNamespaceKind) -> Vec<T>
        where T: Metadata + DeserializeOwned;

    fn create_namespace_resource(&self, key: &ClusterResource, obj: &[u8]) -> StorageResult<()>; // TODO: obj should not be a &[u8] but what should it be? Probably a "Resource" or something like that
*/
}


#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Error accessing the database")]

    DatabaseError {
        source: Box<dyn std::error::Error>,
    }, // TODO: Should maybe take another error

    #[error("Error validating data: {0}")]
    ModelError(String)
}

impl From<r2d2::Error> for StorageError {
    fn from(source: r2d2::Error) -> Self {
        return StorageError::DatabaseError {
            source: Box::new(source)
        }
    }
}

impl From<rusqlite::Error> for StorageError {
    fn from(source: rusqlite::Error) -> Self {
        return StorageError::DatabaseError {
            source: Box::new(source)
        }
    }
}


pub type StorageResult<T> = Result<Option<T>, StorageError>;
