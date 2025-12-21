use std::sync::Arc;

use chrono::NaiveDate;

use crate::db::{Database, StatisticsDao};
use crate::error::Result;
use crate::models::{DailyStatistics, GameStatistics};

#[derive(Clone)]
pub struct StatisticsService {
    dao: StatisticsDao,
}

impl StatisticsService {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            dao: StatisticsDao::new(db),
        }
    }

    /// Get overall statistics for all games
    pub fn get_overall(&self) -> Result<Vec<GameStatistics>> {
        self.dao.get_overall_statistics()
    }

    /// Get daily statistics for a date range
    pub fn get_daily(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<DailyStatistics>> {
        self.dao.get_daily_statistics(start_date, end_date)
    }

    /// Get statistics for a specific game
    pub fn get_for_game(&self, game_id: &str) -> Result<Option<GameStatistics>> {
        self.dao.get_game_statistics(game_id)
    }
}
