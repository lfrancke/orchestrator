use crate::storage::{Storage, StorageResult};

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, named_params};
use serde::de::DeserializeOwned;
use crate::models::{Metadata, Group, Resource, Namespace, ResourceType};
use std::fmt::Debug;

const GET_CLUSTER_RESOURCE_QUERY: &str = "SELECT json FROM cluster_scoped WHERE api_group=:api_group AND resource_type=:resource_type AND resource_name=:resource_name";
const CREATE_CLUSTER_RESOURCE_QUERY: &str = "INSERT INTO cluster_scoped(api_group, resource_type, resource_name, json) VALUES (:api_group, :resource_type, :resource_name, :json)";

const GET_NAMESPACE_RESOURCE_QUERY: &str = "SELECT json FROM namespace_scoped WHERE api_group=:api_group AND namespace=:namespace AND resource_type=:resource_type AND resource_name=:resource_name";
const LIST_NAMESPACE_RESOURCES_QUERY: &str = "SELECT json FROM namespace_scoped WHERE api_group=:api_group AND namespace=:namespace AND resource_type=:resource_type";

#[derive(Clone)]
pub struct SqliteStorage {
    pool: Pool<SqliteConnectionManager>
}

impl SqliteStorage {
    pub fn new() -> SqliteStorage {
        let manager = SqliteConnectionManager::file("orchestrator.db");
        let pool = Pool::new(manager).unwrap();

        SqliteStorage {
            pool
        }
    }
}

impl Storage for SqliteStorage {
    fn get_cluster_resource<T, U>(&self, key: &U) -> StorageResult<T>
        where T: Metadata + DeserializeOwned,
              U: Debug + Group + ResourceType + Resource
    {
        //println!("storage::get_cluster_resource({:?})", key);
        let connection = &self.pool.get()?;
        let mut stmt = connection.prepare(GET_CLUSTER_RESOURCE_QUERY).unwrap();
        let params = named_params![
            ":api_group": key.group(),
            ":resource_type": key.resource_type(),
            ":resource_name": key.resource()
        ];
        let result_iter = stmt.query_map_named(params, |row| row.get(0))?;

        let result = match result_iter.into_iter().next() {
            None => Ok(None),
            Some(Err(e)) => Ok(None), // TOOD: Needs to return error
            Some(Ok(byte_array)) => {
                let byte_array: Vec<u8> = byte_array;
                Ok(Some(serde_json::from_slice(&byte_array).unwrap()))
            }
        };
        result
    }

    fn list_cluster_resources<T, U>(&self, key: &U) -> Vec<T>
        where T: Metadata + DeserializeOwned,
              U: Debug + Group + ResourceType
    {
        println!("storage::list_cluster_kinds({:?})", key);
        let connection = &self.pool.get().unwrap();

        let query = "SELECT json FROM cluster_scoped WHERE api_group=?1 AND resource_type=?2";
        let mut stmt = connection.prepare(query).unwrap();
        let params = params![key.group(), key.resource_type()];
        let result_iter = stmt.query_map(params, |row| {
            row.get(0)
        }).unwrap();

        let mut json_vec: Vec<T> = Vec::new();
        for json in result_iter {
            let byte_array: Vec<u8> = json.unwrap();
            let resource: T = serde_json::from_slice(&byte_array).unwrap();
            json_vec.push(resource);
        }
        json_vec
    }


    fn create_cluster_resource<T>(&self, key: &T, obj: &[u8]) -> StorageResult<()>
        where T: Debug + Group + ResourceType + Resource
    {
        let connection = &self.pool.get()?;

        let res = connection.execute_named(CREATE_CLUSTER_RESOURCE_QUERY,
                                           named_params![
                                        ":api_group": key.group(),
                                        ":resource_type": key.resource_type(),
                                        ":resource_name": key.resource(),
                                        ":json": obj
                                     ])?;

        Ok(Some(()))
    }

    fn get_namespace_resource<T, U>(&self, key: &U) -> StorageResult<T>
        where T: Metadata + DeserializeOwned,
              U: Debug + Group + Namespace + ResourceType + Resource
    {
        //println!("storage::get_namespace_resource({:?})", key);
        let connection = &self.pool.get()?;
        let mut stmt = connection.prepare(GET_NAMESPACE_RESOURCE_QUERY).unwrap();
        let params = named_params![
            ":api_group": key.group(),
            ":namespace": key.namespace(),
            ":resource_type": key.resource_type(),
            ":resource_name": key.resource()
        ];
        let result_iter = stmt.query_map_named(params, |row| row.get(0))?;

        let byte_array: Vec<u8> = result_iter.into_iter().next().unwrap().unwrap();
        Ok(Some(serde_json::from_slice(&byte_array).unwrap()))
    }

    fn list_namespace_resources<T, U>(&self, key: &U) -> StorageResult<Vec<T>>
        where T: Metadata + DeserializeOwned,
              U: Debug + Group + Namespace + ResourceType
    {
        let connection = &self.pool.get()?;
        let mut stmt = connection.prepare(LIST_NAMESPACE_RESOURCES_QUERY).unwrap();
        let params = named_params![
            ":api_group": key.group(),
            ":namespace": key.namespace(),
            ":resource_type": key.resource_type()
        ];

        let result_iter = stmt.query_map_named(params, |row| {
            row.get(0)
        }).unwrap();

        let mut json_vec: Vec<T> = Vec::new();
        for json in result_iter {
            let byte_array: Vec<u8> = json.unwrap();
            let resource: T = serde_json::from_slice(&byte_array).unwrap();
            json_vec.push(resource);
        }
        Ok(Some(json_vec))
    }

}
