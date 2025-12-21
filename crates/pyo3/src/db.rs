use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, LazyLock};

use parking_lot::Mutex;
use playtime_core::db::Database;
use playtime_core::error::Error as CoreError;

pub static DB_CACHE: LazyLock<Mutex<HashMap<String, Arc<Database>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Get or create a database connection
///
/// If the database already exists in the cache, returns the cached instance.
/// Otherwise, creates a new database, runs migrations, and caches it.
pub fn get_or_create_database<P: AsRef<Path>>(db_path: P) -> Result<Arc<Database>, CoreError> {
    let db_path = db_path.as_ref();
    let cache_key = db_path.to_string_lossy().to_string();

    // Try to get from cache first
    let mut cache = DB_CACHE.lock();

    if let Some(db) = cache.get(&cache_key) {
        println!("[RUST][DB_CACHE] Reusing cached database at {:?}", db_path);
        return Ok(Arc::clone(db));
    }

    // Create new database and run migrations
    let db = Database::new(db_path)?;
    db.with_connection(|conn| playtime_core::db::migrations::run_migrations(conn))?;

    let db = Arc::new(db);
    cache.insert(cache_key, Arc::clone(&db));

    println!("[RUST][DB_CACHE] Created new database at {:?}", db_path);

    Ok(db)
}

/// Clear the database cache (useful for testing)
pub fn clear_cache() {
    DB_CACHE.lock().clear();
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn test_cache_reuse() {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("storage.db");

        // Clean up any existing file
        let _ = std::fs::remove_file(&db_path);

        // First access creates the database
        let db1 = get_or_create_database(&db_path).unwrap();

        // Second access should return the same instance (same Arc)
        let db2 = get_or_create_database(&db_path).unwrap();

        assert!(Arc::ptr_eq(&db1, &db2));

        // Cleanup
        clear_cache();
        std::fs::remove_file(db_path).ok();
    }
}
