use chrono::{Local, NaiveDateTime, TimeZone};

#[derive(Debug, Clone)]
pub struct PlaySession {
    pub game_id: String,
    pub started_at: f64,
    pub ended_at: f64,
    pub duration: f64,
    pub checksum: Option<String>,
}

impl PlaySession {
    pub fn new(game_id: String, started_at: f64, ended_at: f64) -> Self {
        let duration = ended_at - started_at;
        Self {
            game_id,
            started_at,
            ended_at,
            duration,
            checksum: None,
        }
    }

    pub fn with_checksum(mut self, checksum: String) -> Self {
        self.checksum = Some(checksum);

        self
    }

    pub fn started_date(&self) -> NaiveDateTime {
        let secs = self.started_at.trunc() as i64;
        let nanos = ((self.started_at.fract() * 1_000_000_000.0) as u32).min(999_999_999);
        Local
            .timestamp_opt(secs, nanos)
            .single()
            .map(|dt| dt.naive_local())
            .unwrap_or_else(|| Local::now().naive_local())
    }

    pub fn ended_date(&self) -> NaiveDateTime {
        let secs = self.ended_at.trunc() as i64;
        let nanos = ((self.ended_at.fract() * 1_000_000_000.0) as u32).min(999_999_999);
        Local
            .timestamp_opt(secs, nanos)
            .single()
            .map(|dt| dt.naive_local())
            .unwrap_or_else(|| Local::now().naive_local())
    }

    pub fn is_multi_day(&self) -> bool {
        let start_date = self.started_date().date();
        let end_date = self.ended_date().date();

        start_date != end_date
    }
}
