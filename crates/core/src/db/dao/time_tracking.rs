use std::sync::Arc;

use chrono::{Local, NaiveDateTime};
use rusqlite::params;

use crate::db::Database;
use crate::error::{Error, Result};
use crate::models::PlaySession;
use crate::utils::time::split_session_by_day;

#[derive(Clone)]
pub struct TimeTrackingDao {
    db: Arc<Database>,
}

impl TimeTrackingDao {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn add_time(
        &self,
        game_id: &str,
        game_name: &str,
        started_at: f64,
        ended_at: f64,
        source: Option<&str>,
    ) -> Result<()> {
        if ended_at <= started_at {
            return Err(Error::InvalidInput(
                "End time must be after start time".into(),
            ));
        }

        let session = PlaySession::new(game_id.to_string(), started_at, ended_at);

        let sessions = if session.is_multi_day() {
            split_session_by_day(&session)
        } else {
            vec![session]
        };

        self.db.transaction(|tx| {
            tx.execute(
                "INSERT INTO game_dict (game_id, name) VALUES (?1, ?2)
                 ON CONFLICT(game_id) DO UPDATE SET name = ?2",
                params![game_id, game_name],
            )?;

            for session in sessions {
                let date = session.started_date();

                println!(
                    "Inserting playtime: game_id={}, date={}, duration={}",
                    session.game_id,
                    date.format("%Y-%m-%dT%H:%M:%S"),
                    session.duration
                );

                tx.execute(
                    r#"
                    INSERT INTO play_time(date_time, duration, game_id, migrated)
                    VALUES (?1, ?2, ?3, ?4)
                    "#,
                    params![
                        date.format("%Y-%m-%dT%H:%M:%S").to_string(),
                        session.duration,
                        session.game_id,
                        source
                    ],
                )?;

                tx.execute(
                    r#"
                    INSERT INTO overall_time (game_id, duration)
                    VALUES (?1, ?2)
                    ON CONFLICT(game_id) DO UPDATE SET duration = duration + ?2
                    "#,
                    params![session.game_id, session.duration],
                )?;
            }

            Ok(())
        })
    }

    pub fn apply_manual_time_correction(
        &self,
        game_id: &str,
        game_name: &str,
        time_seconds: i64,
        source: &str,
    ) -> Result<()> {
        let now = Local::now().naive_local();

        self.db.transaction(|tx| {
            tx.execute(
                "INSERT INTO game_dict (game_id, name) VALUES (?1, ?2)
                 ON CONFLICT(game_id) DO UPDATE SET name = ?2",
                params![game_id, game_name],
            )?;

            tx.execute(
                r#"
                INSERT INTO play_time (game_id, date, time, migrated)
                VALUES (?1, ?2, ?3, ?4)
                "#,
                params![
                    game_id,
                    now.format("%Y-%m-%dT%H:%M:%S").to_string(),
                    time_seconds,
                    source,
                ],
            )?;

            Ok(())
        })
    }

    pub fn get_game_sessions(&self, game_id: &str) -> Result<Vec<PlaySession>> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                r#"
                SELECT game_id, date, time, checksum
                FROM play_time
                WHERE game_id = ?1
                ORDER BY date DESC
                "#,
            )?;

            let sessions = stmt
                .query_map(params![game_id], |row| {
                    let date_str: String = row.get(1)?;
                    let date = NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%dT%H:%M:%S")
                        .unwrap_or_else(|_| Local::now().naive_local());

                    let started_at = date.and_local_timezone(Local).unwrap().timestamp() as f64;
                    let duration: i64 = row.get(2)?;
                    let duration_f64 = duration as f64;

                    Ok(PlaySession {
                        game_id: row.get(0)?,
                        started_at,
                        ended_at: started_at + duration_f64,
                        duration: duration_f64,
                        checksum: row.get(3)?,
                    })
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?;

            Ok(sessions)
        })
    }

    pub fn get_total_playtime(&self, game_id: &str) -> Result<i64> {
        self.db.with_connection(|conn| {
            let total: i64 = conn.query_row(
                "SELECT COALESCE(SUM(time), 0) FROM play_time WHERE game_id = ?1",
                params![game_id],
                |row| row.get(0),
            )?;

            Ok(total)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    fn setup_test_db() -> Arc<Database> {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join(format!("test_time_{}.db", uuid::Uuid::new_v4()));
        let db = Arc::new(Database::new(&db_path).unwrap());

        db.with_connection(|conn| {
            conn.execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS game_dict (
                    game_id TEXT PRIMARY KEY,
                    name TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS play_time (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    game_id TEXT NOT NULL,
                    date TEXT NOT NULL,
                    time INTEGER NOT NULL,
                    checksum TEXT,
                    migrated TEXT,
                    FOREIGN KEY (game_id) REFERENCES game_dict(game_id)
                );
                "#,
            )?;
            Ok(())
        })
        .unwrap();

        db
    }

    #[test]
    fn test_add_time() {
        let db = setup_test_db();
        let dao = TimeTrackingDao::new(db);

        let now = Local::now().timestamp() as f64;
        let result = dao.add_time("123", "Test Game", now, now + 3600.0, None);

        assert!(result.is_ok());
    }
}
