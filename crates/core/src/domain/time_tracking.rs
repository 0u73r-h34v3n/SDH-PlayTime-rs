use std::sync::Arc;

use crate::db::{Database, TimeTrackingDao};
use crate::error::Result;
use crate::models::PlaySession;

#[derive(Clone)]
pub struct TimeTrackingService {
    dao: TimeTrackingDao,
}

impl TimeTrackingService {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            dao: TimeTrackingDao::new(db),
        }
    }

    /// Add playtime for a game
    pub fn add_time(
        &self,
        game_id: &str,
        game_name: &str,
        started_at: f64,
        ended_at: f64,
        source: Option<&str>,
    ) -> Result<()> {
        self.dao
            .add_time(game_id, game_name, started_at, ended_at, source)
    }

    /// Apply manual time correction
    pub fn apply_manual_correction(
        &self,
        game_id: &str,
        game_name: &str,
        time_seconds: i64,
        source: &str,
    ) -> Result<()> {
        self.dao
            .apply_manual_time_correction(game_id, game_name, time_seconds, source)
    }

    /// Get all sessions for a game
    pub fn get_game_sessions(&self, game_id: &str) -> Result<Vec<PlaySession>> {
        self.dao.get_game_sessions(game_id)
    }

    /// Get total playtime for a game
    pub fn get_total_playtime(&self, game_id: &str) -> Result<i64> {
        self.dao.get_total_playtime(game_id)
    }
}
