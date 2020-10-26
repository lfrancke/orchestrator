use crate::storage::{Storage, StorageResourceType};

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use crate::crd::Metadata;
use serde::de::DeserializeOwned;

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
    fn get<T>(&self, key: &StorageResourceType, resource_name: &str) -> Option<T>
        where T: Metadata + DeserializeOwned
    {
        println!("storage::get({:?})", key);
        let connection = &self.pool.get().unwrap();

        // TODO: Look at the list method for comments about this match block
        return match key {
            StorageResourceType::ClusterScoped { group: api_group, name: resource_type_name } => {
                let query = "SELECT json FROM cluster_scoped WHERE api_group=?1 AND resource_type_name=?2 AND resource_name=?3";
                let mut stmt = connection.prepare(query).unwrap();
                let params = params![api_group, resource_type_name, resource_name];
                let result_iter = stmt.query_map(params, |row| {
                    row.get(0)
                }).unwrap();

                let byte_array: Vec<u8> = result_iter.into_iter().next().unwrap().unwrap();
                Some(serde_json::from_slice(&byte_array).unwrap())
            }
            StorageResourceType::NamespaceScoped { group, name, namespace } => {
                unimplemented!()
                /*
                let query = "SELECT json FROM namespace_scoped WHERE api_group=?1 AND resource_type_name=?2 AND namespace=?3";
                let mut stmt = connection.prepare(query).unwrap();
                let params = params![group, name, namespace];
                let res = stmt.query_map(params, |row| {

                    row.get(0)

                });

                 */
            }
        };
    }

    fn list<T>(&self, key: &StorageResourceType) -> Vec<T>
        where T: Metadata + DeserializeOwned
    {
        println!("storage::list({:?})", key);
        let connection = &self.pool.get().unwrap();

        // Issue 1: I'd love to just return a tuple of query string and param object from the match to keep the connection outside of the match
        // But I couldn't get that o work due to this issue:
        // https://users.rust-lang.org/t/rusqlite-query-params-match-borrowing-issue/35047/2
        // Issue 2: I wanted to return the result of the query_map but it's a closure and no two closures have the same type so rust complains that both arms have different types
        // I have not yet tried the workarounds mentioned in the compiler error (https://stackoverflow.com/questions/39083375/expected-closure-found-a-different-closure)
        return match key {
            StorageResourceType::ClusterScoped { group, name } => {
                let query = "SELECT json FROM cluster_scoped WHERE api_group=?1 AND resource_type_name=?2";
                let mut stmt = connection.prepare(query).unwrap();
                let params = params![group, name];
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
            StorageResourceType::NamespaceScoped { group, name, namespace } => {
                let mut json_vec: Vec<T> = Vec::new();
                json_vec
                /*
                let query = "SELECT json FROM namespace_scoped WHERE api_group=?1 AND resource_type_name=?2 AND namespace=?3";
                let mut stmt = connection.prepare(query).unwrap();
                let params = params![group, name, namespace];
                let res = stmt.query_map(params, |row| {

                    row.get(0)

                });

                 */
            }
        };
    }

    fn create(&self, key: &StorageResourceType, resource_name: String, obj: &[u8]) {
        let connection = &self.pool.get().unwrap();

        match key {
            StorageResourceType::ClusterScoped { group, name: resource_type_name } => {
                let res = connection.execute(
                    "INSERT INTO cluster_scoped(api_group, resource_type_name, resource_name, json) VALUES (?1, ?2, ?3, ?4)",
                    params![group, resource_type_name, resource_name, obj]);
            }
            StorageResourceType::NamespaceScoped { group, name: resource_type_name, namespace } => {
                let res = connection.execute(
                    "INSERT INTO namespace_scoped(api_group, resource_type_name, namespace, resource_name, json) VALUES (?1, ?2, ?3, ?4)",
                    params![group, namespace, resource_type_name, resource_name, obj]);
            }
        }
        // TODO: Return something
    }
}
