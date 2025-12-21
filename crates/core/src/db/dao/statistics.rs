use std::sync::Arc;

use chrono::{NaiveDate, NaiveDateTime};
use rusqlite::{OptionalExtension, params};

use crate::db::Database;
use crate::error::Result;
use crate::models::{DailyGameStats, DailyStatistics, Game, GameStatistics, SessionInfo};

#[derive(Clone)]
pub struct StatisticsDao {
    db: Arc<Database>,
}

impl StatisticsDao {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn get_overall_statistics(&self) -> Result<Vec<GameStatistics>> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                r#"
                SELECT
                    g.game_id,
                    g.name,
                    COALESCE(SUM(pt.time), 0) as total_time,
                    COUNT(pt.id) as total_sessions,
                    MAX(pt.date) as last_played
                FROM game_dict g
                LEFT JOIN play_time pt ON g.game_id = pt.game_id
                GROUP BY g.game_id, g.name
                HAVING total_time > 0
                ORDER BY total_time DESC
                "#,
            )?;

            let stats = stmt
                .query_map([], |row| {
                    Ok(GameStatistics {
                        game: Game {
                            id: row.get(0)?,
                            name: row.get(1)?,
                        },
                        total_time: row.get(2)?,
                        total_sessions: row.get(3)?,
                        last_played: row.get::<_, Option<String>>(4)?.and_then(|s| {
                            NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S").ok()
                        }),
                        last_session_duration: None,
                    })
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?;

            Ok(stats)
        })
    }

    pub fn get_daily_statistics(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<DailyStatistics>> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                r#"
                SELECT
                    DATE(pt.date) as play_date,
                    g.game_id,
                    g.name,
                    SUM(pt.time) as total_time,
                    pt.date,
                    pt.time,
                    pt.migrated,
                    pt.checksum
                FROM play_time pt
                JOIN game_dict g ON pt.game_id = g.game_id
                WHERE DATE(pt.date) BETWEEN ?1 AND ?2
                GROUP BY DATE(pt.date), g.game_id, g.name, pt.date
                ORDER BY DATE(pt.date) DESC, total_time DESC
                "#,
            )?;

            let rows = stmt.query_map(
                params![start_date.to_string(), end_date.to_string()],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,         // date
                        row.get::<_, String>(1)?,         // game_id
                        row.get::<_, String>(2)?,         // game_name
                        row.get::<_, i64>(3)?,            // total_time
                        row.get::<_, String>(4)?,         // session_date
                        row.get::<_, f64>(5)?,            // session_duration
                        row.get::<_, Option<String>>(6)?, // migrated
                        row.get::<_, Option<String>>(7)?, // checksum
                    ))
                },
            )?;

            let mut daily_stats: std::collections::HashMap<String, Vec<_>> =
                std::collections::HashMap::new();

            for row in rows {
                let (date, game_id, game_name, _total, session_date, duration, migrated, checksum) =
                    row?;
                daily_stats.entry(date).or_insert_with(Vec::new).push((
                    game_id,
                    game_name,
                    session_date,
                    duration,
                    migrated,
                    checksum,
                ));
            }

            let mut result = Vec::new();
            for (date_str, games_data) in daily_stats {
                let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                    .unwrap_or_else(|_| chrono::Local::now().date_naive());

                let mut game_map: std::collections::HashMap<String, Vec<_>> =
                    std::collections::HashMap::new();

                for (game_id, game_name, session_date, duration, migrated, checksum) in games_data {
                    game_map
                        .entry(game_id.clone())
                        .or_insert_with(Vec::new)
                        .push((game_name, session_date, duration, migrated, checksum));
                }

                let games = game_map
                    .into_iter()
                    .map(|(game_id, sessions)| {
                        let game_name = sessions[0].0.clone();
                        let total_time: f64 = sessions.iter().map(|(_, _, d, _, _)| d).sum();

                        let session_infos = sessions
                            .into_iter()
                            .map(|(_, date, duration, migrated, checksum)| SessionInfo {
                                date: NaiveDateTime::parse_from_str(&date, "%Y-%m-%dT%H:%M:%S")
                                    .unwrap_or_else(|_| {
                                        chrono::DateTime::from_timestamp(0, 0)
                                            .unwrap()
                                            .naive_local()
                                    }),
                                duration,
                                migrated,
                                checksum,
                            })
                            .collect();

                        DailyGameStats {
                            game: Game::new(game_id, game_name),
                            time: total_time as i64,
                            sessions: session_infos,
                        }
                    })
                    .collect();

                result.push(DailyStatistics { date, games });
            }

            result.sort_by(|a, b| b.date.cmp(&a.date));
            Ok(result)
        })
    }

    pub fn get_game_statistics(&self, game_id: &str) -> Result<Option<GameStatistics>> {
        self.db.with_connection(|conn| {
            let result = conn
                .query_row(
                    r#"
                SELECT
                    g.game_id,
                    g.name,
                    COALESCE(SUM(pt.time), 0) as total_time,
                    COUNT(pt.id) as total_sessions,
                    MAX(pt.date) as last_played
                FROM game_dict g
                LEFT JOIN play_time pt ON g.game_id = pt.game_id
                WHERE g.game_id = ?1
                GROUP BY g.game_id, g.name
                "#,
                    params![game_id],
                    |row| {
                        Ok(GameStatistics {
                            game: Game {
                                id: row.get(0)?,
                                name: row.get(1)?,
                            },
                            total_time: row.get(2)?,
                            total_sessions: row.get(3)?,
                            last_played: row.get::<_, Option<String>>(4)?.and_then(|s| {
                                NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S").ok()
                            }),
                            last_session_duration: None,
                        })
                    },
                )
                .optional()?;

            Ok(result)
        })
    }
}
