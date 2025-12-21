use chrono::{Datelike, NaiveDate, NaiveDateTime};

use crate::models::PlaySession;

/// Get the end of day (23:59:59) for a given timestamp
pub fn end_of_day(dt: NaiveDateTime) -> NaiveDateTime {
    NaiveDate::from_ymd_opt(dt.year(), dt.month(), dt.day())
        .and_then(|d| d.and_hms_opt(23, 59, 59))
        .unwrap_or(dt)
}

/// Get the start of day (00:00:00) for a given timestamp
pub fn start_of_day(dt: NaiveDateTime) -> NaiveDateTime {
    NaiveDate::from_ymd_opt(dt.year(), dt.month(), dt.day())
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .unwrap_or(dt)
}

/// Split a play session that spans multiple days into separate sessions
/// Each session will be bounded by day boundaries
pub fn split_session_by_day(session: &PlaySession) -> Vec<PlaySession> {
    let start = session.started_date();
    let end = session.ended_date();

    if !session.is_multi_day() {
        return vec![session.clone()];
    }

    // Calculate number of days spanned
    let start_day = start.date();
    let end_day = end.date();
    let days_count = (end_day - start_day).num_days() as usize + 1;

    let mut sessions = Vec::with_capacity(days_count);
    let mut current_start = start;

    while current_start < end {
        let day_end = end_of_day(current_start);
        let session_end = if day_end < end { day_end } else { end };

        let duration = (session_end.and_utc().timestamp_millis()
            - current_start.and_utc().timestamp_millis()) as f64
            / 1000.0;

        if duration > 0.0 {
            sessions.push(PlaySession {
                game_id: session.game_id.clone(),
                started_at: current_start.and_utc().timestamp_millis() as f64 / 1000.0,
                ended_at: session_end.and_utc().timestamp_millis() as f64 / 1000.0,
                duration,
                checksum: session.checksum.clone(),
            });
        }

        // Move to start of next day
        current_start = start_of_day(session_end) + chrono::Duration::days(1);
    }

    sessions
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike, Timelike};

    use super::*;

    #[test]
    fn test_split_single_day_session() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1)
            .and_then(|d| d.and_hms_opt(10, 0, 0))
            .unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 1)
            .and_then(|d| d.and_hms_opt(12, 0, 0))
            .unwrap();

        let session = PlaySession::new(
            "game123".to_string(),
            start.and_utc().timestamp() as f64,
            end.and_utc().timestamp() as f64,
        );

        let splits = split_session_by_day(&session);
        assert_eq!(splits.len(), 1);
        assert_eq!(splits[0].duration, 7200.0); // 2 hours
    }

    #[test]
    fn test_split_multi_day_session() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1)
            .and_then(|d| d.and_hms_opt(22, 0, 0))
            .unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 2)
            .and_then(|d| d.and_hms_opt(2, 0, 0))
            .unwrap();

        let session = PlaySession::new(
            "game123".to_string(),
            start.and_utc().timestamp() as f64,
            end.and_utc().timestamp() as f64,
        );

        let splits = split_session_by_day(&session);
        assert_eq!(splits.len(), 2);

        // First session: 22:00 to 23:59:59
        assert!(splits[0].duration > 7100.0 && splits[0].duration < 7200.0);

        // Second session: 00:00:00 to 02:00:00
        assert!(splits[1].duration > 7100.0 && splits[1].duration < 7300.0);
    }

    #[test]
    fn test_end_of_day() {
        let dt = NaiveDate::from_ymd_opt(2024, 1, 15)
            .and_then(|d| d.and_hms_opt(10, 30, 45))
            .unwrap();
        let eod = end_of_day(dt);

        assert_eq!(eod.hour(), 23);
        assert_eq!(eod.minute(), 59);
        assert_eq!(eod.second(), 59);
        assert_eq!(eod.day(), 15);
    }

    #[test]
    fn test_start_of_day() {
        let dt = NaiveDate::from_ymd_opt(2024, 1, 15)
            .and_then(|d| d.and_hms_opt(10, 30, 45))
            .unwrap();
        let sod = start_of_day(dt);

        assert_eq!(sod.hour(), 0);
        assert_eq!(sod.minute(), 0);
        assert_eq!(sod.second(), 0);
        assert_eq!(sod.day(), 15);
    }
}
