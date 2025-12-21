use chrono::{NaiveDate, NaiveDateTime};

use crate::models::Game;

#[derive(Debug, Clone)]
pub struct GameStatistics {
    pub game: Game,
    pub total_time: i64,
    pub total_sessions: i64,
    pub last_played: Option<NaiveDateTime>,
    pub last_session_duration: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct DailyStatistics {
    pub date: NaiveDate,
    pub games: Vec<DailyGameStats>,
}

#[derive(Debug, Clone)]
pub struct DailyGameStats {
    pub game: Game,
    pub time: i64,
    pub sessions: Vec<SessionInfo>,
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub date: NaiveDateTime,
    pub duration: f64,
    pub migrated: Option<String>,
    pub checksum: Option<String>,
}

impl SessionInfo {
    pub fn new(date: NaiveDateTime, duration: f64) -> Self {
        Self {
            date,
            duration,
            migrated: None,
            checksum: None,
        }
    }
}
