use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

// TODO: Stop hardcoding file name
pub fn get_pool() -> Pool<SqliteConnectionManager> {
    let manager = SqliteConnectionManager::file("orchestrator.db");
    Pool::new(manager).unwrap()
}
