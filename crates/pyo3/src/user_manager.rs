use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::Mutex;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

use crate::db::get_or_create_database;

const USERS_SUBDIR: &str = "users";
const STORAGE_DB_FILENAME: &str = "storage.db";

#[pyclass]
pub struct UserManager {
    data_dir: PathBuf,
    current_user_id: Arc<Mutex<Option<String>>>,
}

#[pymethods]
impl UserManager {
    #[new]
    fn new(data_dir: String) -> PyResult<Self> {
        let data_dir = PathBuf::from(data_dir);

        if !data_dir.exists() {
            fs::create_dir_all(&data_dir)
                .map_err(|e| PyException::new_err(format!("Failed to create data_dir: {}", e)))?;
        }

        Ok(Self {
            data_dir,
            current_user_id: Arc::new(Mutex::new(None)),
        })
    }

    fn set_current_user(&self, user_id: String) -> PyResult<()> {
        let user_id = user_id.trim();

        if user_id.is_empty() {
            return Err(PyException::new_err("user_id cannot be empty"));
        }

        // Validate Steam ID format (should be numeric, 17 digits for 64-bit Steam ID)
        if !user_id.chars().all(|c| c.is_ascii_digit()) {
            return Err(PyException::new_err(format!(
                "Invalid Steam ID format: {}",
                user_id
            )));
        }

        if self.has_legacy_db() && !self.has_user_db(user_id) {
            self.migrate_legacy_db_for_user(user_id)?;
        }

        let db_path = self.get_user_db_path(user_id);

        let _ = get_or_create_database(&db_path).map_err(|e| {
            PyException::new_err(format!(
                "Failed to initialize database for user {}: {}",
                user_id, e
            ))
        })?;

        *self.current_user_id.lock() = Some(user_id.to_string());

        Ok(())
    }

    fn get_current_user_id(&self) -> Option<String> {
        self.current_user_id.lock().clone()
    }

    fn list_users(&self) -> PyResult<Vec<String>> {
        let users_dir = self.users_dir();

        if !users_dir.exists() {
            return Ok(Vec::new());
        }

        let mut users = Vec::new();

        let entries = fs::read_dir(&users_dir)
            .map_err(|e| PyException::new_err(format!("Failed to read users directory: {}", e)))?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                PyException::new_err(format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let user_db_path = path.join(STORAGE_DB_FILENAME);

            if user_db_path.exists()
                && let Some(user_id) = path.file_name().and_then(|n| n.to_str())
            {
                users.push(user_id.to_string());
            }
        }

        Ok(users)
    }

    fn has_legacy_db(&self) -> bool {
        self.legacy_db_path().exists()
    }

    fn has_user_db(&self, user_id: &str) -> bool {
        self.get_user_db_path(user_id).exists()
    }

    fn get_user_db_path_str(&self, user_id: String) -> PyResult<String> {
        let path = self.get_user_db_path(&user_id);
        Ok(path.to_string_lossy().to_string())
    }

    fn get_data_dir(&self) -> String {
        self.data_dir.to_string_lossy().to_string()
    }

    fn clear_current_user(&self) {
        *self.current_user_id.lock() = None;
    }
}

impl UserManager {
    fn legacy_db_path(&self) -> PathBuf {
        self.data_dir.join(STORAGE_DB_FILENAME)
    }

    fn users_dir(&self) -> PathBuf {
        self.data_dir.join(USERS_SUBDIR)
    }

    fn get_user_db_path(&self, user_id: &str) -> PathBuf {
        self.users_dir().join(user_id).join(STORAGE_DB_FILENAME)
    }

    fn migrate_legacy_db_for_user(&self, user_id: &str) -> PyResult<()> {
        let legacy_path = self.legacy_db_path();
        let user_db_path = self.get_user_db_path(user_id);

        // Create user directory
        if let Some(parent) = user_db_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                PyException::new_err(format!(
                    "Failed to create user directory for {}: {}",
                    user_id, e
                ))
            })?;
        }

        // Get legacy DB size for logging
        let legacy_size = fs::metadata(&legacy_path).map(|m| m.len()).unwrap_or(0);
        let legacy_size_mb = legacy_size as f64 / (1024.0 * 1024.0);

        println!(
            "[UserManager] Migrating legacy DB for user {}: {} -> {} (size: {:.2} MB)",
            user_id,
            legacy_path.display(),
            user_db_path.display(),
            legacy_size_mb
        );

        fs::copy(&legacy_path, &user_db_path).map_err(|e| {
            PyException::new_err(format!(
                "Failed to migrate legacy DB for user {}: {}",
                user_id, e
            ))
        })?;

        println!(
            "[UserManager] Successfully migrated legacy DB for user: {} ({:.2} MB copied)",
            user_id, legacy_size_mb
        );

        Ok(())
    }
}

