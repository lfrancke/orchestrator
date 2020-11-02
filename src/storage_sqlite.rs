use crate::storage::{Storage, StorageResult};

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, named_params};
use serde::de::DeserializeOwned;
use crate::models::{GroupKind, Metadata, ClusterResource, NamespaceResource};

const GET_CLUSTER_RESOURCE_QUERY: &str = "SELECT json FROM cluster_scoped WHERE api_group=:api_group AND kind=:kind AND resource_name=:resource_name";
const CREATE_CLUSTER_RESOURCE_QUERY: &str = "INSERT INTO cluster_scoped(api_group, kind, resource_name, json) VALUES (:api_group, :kind, :resource_name, :json)";

const GET_NAMESPACE_RESOURCE_QUERY: &str = "SELECT json FROM namespace_scoped WHERE api_group=:api_group AND namespace=:namespace AND kind=:kind AND resource_name=:resource_name";

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
    fn get_cluster_resource<T>(&self, key: &ClusterResource) -> StorageResult<T>
        where T: Metadata + DeserializeOwned
    {
        //println!("storage::get_cluster_resource({:?})", key);
        let connection = &self.pool.get()?;
        let mut stmt = connection.prepare(GET_CLUSTER_RESOURCE_QUERY).unwrap();
        let params = named_params![
            ":api_group": key.group_kind.group,
            ":kind": key.group_kind.kind,
            ":resource_name": key.resource
        ];
        let result_iter = stmt.query_map_named(params, |row| row.get(0))?;

        let byte_array: Vec<u8> = result_iter.into_iter().next().unwrap().unwrap();
        Ok(Some(serde_json::from_slice(&byte_array).unwrap()))
    }

    fn list_cluster_resources<T>(&self, key: &GroupKind) -> Vec<T>
        where T: Metadata + DeserializeOwned
    {
        println!("storage::list_cluster_kinds({:?})", key);
        let connection = &self.pool.get().unwrap();

        let query = "SELECT json FROM cluster_scoped WHERE api_group=?1 AND kind=?2";
        let mut stmt = connection.prepare(query).unwrap();
        let params = params![key.group, key.kind];
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

        // Issue 1: I'd love to just return a tuple of query string and param object from the match to keep the connection outside of the match
        // But I couldn't get that o work due to this issue:
        // https://users.rust-lang.org/t/rusqlite-query-params-match-borrowing-issue/35047/2
        // Issue 2: I wanted to return the result of the query_map but it's a closure and no two closures have the same type so rust complains that both arms have different types
        // I have not yet tried the workarounds mentioned in the compiler error (https://stackoverflow.com/questions/39083375/expected-closure-found-a-different-closure)
    }


    fn create_cluster_resource(&self, key: &ClusterResource, obj: &[u8]) -> StorageResult<()> {
        let connection = &self.pool.get()?;

        let res = connection.execute_named(CREATE_CLUSTER_RESOURCE_QUERY,
                                     named_params![
                                        ":api_group": key.group_kind.group,
                                        ":kind": key.group_kind.kind,
                                        ":resource_name": key.resource,
                                        ":json": obj
                                     ])?;

        Ok(Some(()))
    }

    fn get_namespace_resource<T>(&self, key: &NamespaceResource) -> StorageResult<T> where T: Metadata + DeserializeOwned {
        //println!("storage::get_namespace_resource({:?})", key);
        let connection = &self.pool.get()?;
        let mut stmt = connection.prepare(GET_NAMESPACE_RESOURCE_QUERY).unwrap();
        let params = named_params![
            ":api_group": key.group_namespace_kind.group_kind.group,
            ":namespace": key.group_namespace_kind.namespace,
            ":kind": key.group_namespace_kind.group_kind.kind,
            ":resource_name": key.resource
        ];
        let result_iter = stmt.query_map_named(params, |row| row.get(0))?;

        let byte_array: Vec<u8> = result_iter.into_iter().next().unwrap().unwrap();
        Ok(Some(serde_json::from_slice(&byte_array).unwrap()))

    }
}
