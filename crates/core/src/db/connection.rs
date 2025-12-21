use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::Mutex;
use rusqlite::{Connection, OpenFlags};

use crate::Result;

#[derive(Clone)]
pub struct Database {
    path: PathBuf,
    connection: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Self::create_connection(&path)?;

        Ok(Self {
            path,
            connection: Arc::new(Mutex::new(conn)),
        })
    }

    fn create_connection(path: &Path) -> Result<Connection> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;

        // Apply persistent settings
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;
            PRAGMA cache_size = -20000;
            "#,
        )?;

        Ok(conn)
    }

    pub fn with_connection<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Connection) -> Result<T>,
    {
        let mut guard = self.connection.lock();
        f(&mut guard)
    }

    pub fn transaction<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&rusqlite::Transaction) -> Result<T>,
    {
        self.with_connection(|conn| {
            let tx = conn.transaction()?;
            let result = f(&tx)?;
            tx.commit()?;
            Ok(result)
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Database")
            .field("path", &self.path)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_database() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_playtime.db");

        let _db = Database::new(&db_path).unwrap();
        assert!(db_path.exists());

        // Cleanup
        std::fs::remove_file(db_path).ok();
    }
}
