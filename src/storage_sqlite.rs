use crate::storage::Storage;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

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
    fn get(&self, key: &str) {
        unimplemented!()
    }

    fn create(&self, key: &str, obj: &Vec<u8>) {
        let connection = &self.pool.get().unwrap();
        let res = connection.execute(
            "INSERT INTO data(id, json) VALUES (?1, ?2) ON CONFLICT(id) DO UPDATE SET json=excluded.json",
            params![key, obj]);
    }
}
