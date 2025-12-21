//! PlayTime - Main PyO3 class for time tracking operations
//!
//! Stateless API that requires user_id and data_dir for each operation.
//! All methods use the global DB_CACHE for connection pooling.

use std::path::PathBuf;
use std::sync::Arc;

use playtime_core::db::Database;
use playtime_core::domain::TimeTrackingService;
use playtime_core::error::Error as CoreError;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

use crate::db::get_or_create_database;

/// Convert core errors to Python exceptions
fn to_py_err(err: CoreError) -> PyErr {
    PyException::new_err(err.to_string())
}

#[pyclass]
pub struct PlayTime {}

#[pymethods]
impl PlayTime {
    #[new]
    fn new() -> PyResult<Self> {
        Ok(Self {})
    }

    fn add_time(
        &self,
        user_id: &str,
        data_dir: &str,
        game_id: &str,
        game_name: &str,
        started_at: f64,
        ended_at: f64,
    ) -> PyResult<()> {
        let db = Self::get_database(user_id, data_dir).map_err(to_py_err)?;
        let service = TimeTrackingService::new(db);

        println!(
            "[RUST][add_time] user_id: {}, game_id: {}, started_at: {}, ended_at: {}",
            user_id, game_id, started_at, ended_at
        );

        service
            .add_time(game_id, game_name, started_at, ended_at, None)
            .map_err(to_py_err)
    }

}

impl PlayTime {
    /// Get database connection for a user (cached)
    pub fn get_database(user_id: &str, data_dir: &str) -> Result<Arc<Database>, CoreError> {
        println!("[RUST][get_database] {} | {}", user_id, data_dir);

        let db_path = PathBuf::from(data_dir)
            .join("users")
            .join(user_id)
            .join("storage.db");

        get_or_create_database(&db_path)
    }
}
