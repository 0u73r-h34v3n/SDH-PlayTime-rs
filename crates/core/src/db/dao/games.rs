use std::sync::Arc;

use rusqlite::{OptionalExtension, params};

use crate::db::Database;
use crate::error::Result;
use crate::models::{ChecksumAlgorithm, Game, GameChecksum, GameStatistics};

#[derive(Clone)]
pub struct GamesDao {
    db: Arc<Database>,
}

impl GamesDao {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn get_game(&self, game_id: &str) -> Result<Option<Game>> {
        self.db.with_connection(|conn| {
            let mut stmt =
                conn.prepare("SELECT game_id, name FROM game_dict WHERE game_id = ?1")?;

            let game = stmt
                .query_row(params![game_id], |row| {
                    Ok(Game {
                        id: row.get(0)?,
                        name: row.get(1)?,
                    })
                })
                .optional()?;

            Ok(game)
        })
    }

    pub fn save_game(&self, game: &Game) -> Result<()> {
        self.db.with_connection(|conn| {
            conn.execute(
                "INSERT INTO game_dict (game_id, name)
                 VALUES (?1, ?2)
                 ON CONFLICT(game_id) DO UPDATE SET name = ?2",
                params![&game.id, &game.name],
            )?;
            Ok(())
        })
    }

    pub fn get_all_games(&self) -> Result<Vec<Game>> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare("SELECT game_id, name FROM game_dict ORDER BY name")?;

            let games = stmt
                .query_map([], |row| {
                    Ok(Game {
                        id: row.get(0)?,
                        name: row.get(1)?,
                    })
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?;

            Ok(games)
        })
    }

    pub fn get_game_with_stats(&self, game_id: &str) -> Result<Option<GameStatistics>> {
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
                WHERE g.game_id = ?1
                GROUP BY g.game_id, g.name
                "#,
            )?;

            let stats = stmt
                .query_row(params![game_id], |row| {
                    Ok(GameStatistics {
                        game: Game {
                            id: row.get(0)?,
                            name: row.get(1)?,
                        },
                        total_time: row.get(2)?,
                        total_sessions: row.get(3)?,
                        last_played: row
                            .get::<_, Option<String>>(4)?
                            .and_then(|s| s.parse().ok()),
                        last_session_duration: None,
                    })
                })
                .optional()?;

            Ok(stats)
        })
    }

    pub fn save_game_checksum(&self, checksum: &GameChecksum) -> Result<()> {
        self.db.with_connection(|conn| {
            self.save_game(&checksum.game)?;

            conn.execute(
                r#"
                INSERT INTO game_file_checksum
                    (game_id, checksum, algorithm, chunk_size, created_at, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                ON CONFLICT(game_id, checksum, algorithm) DO UPDATE SET
                    updated_at = ?6
                "#,
                params![
                    &checksum.game.id,
                    &checksum.checksum,
                    checksum.algorithm.to_string(),
                    checksum.chunk_size as i64,
                    checksum.created_at.map(|dt| dt.to_rfc3339()),
                    checksum.updated_at.map(|dt| dt.to_rfc3339()),
                ],
            )?;
            Ok(())
        })
    }

    pub fn get_game_checksums(&self, game_id: &str) -> Result<Vec<GameChecksum>> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                r#"
                SELECT
                    g.game_id, g.name,
                    gfc.checksum, gfc.algorithm, gfc.chunk_size,
                    gfc.created_at, gfc.updated_at
                FROM game_file_checksum gfc
                JOIN game_dict g ON gfc.game_id = g.game_id
                WHERE gfc.game_id = ?1
                "#,
            )?;

            let checksums = stmt
                .query_map(params![game_id], |row| {
                    Ok(GameChecksum {
                        game: Game {
                            id: row.get(0)?,
                            name: row.get(1)?,
                        },
                        checksum: row.get(2)?,
                        algorithm: match row.get::<_, String>(3)?.as_str() {
                            "sha256" => ChecksumAlgorithm::Sha256,
                            "md5" => ChecksumAlgorithm::Md5,
                            _ => ChecksumAlgorithm::Sha256,
                        },
                        chunk_size: row.get::<_, i64>(4)? as usize,
                        created_at: row
                            .get::<_, Option<String>>(5)?
                            .and_then(|s| s.parse().ok()),
                        updated_at: row
                            .get::<_, Option<String>>(6)?
                            .and_then(|s| s.parse().ok()),
                    })
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?;

            Ok(checksums)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn setup_test_db() -> Arc<Database> {
        let temp_dir = env::temp_dir();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let db_path = temp_dir.join(format!("test_games_{}.db", timestamp));
        let db = Arc::new(Database::new(&db_path).unwrap());

        // Create tables (simplified)
        db.with_connection(|conn| {
            conn.execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS game_dict (
                    game_id TEXT PRIMARY KEY,
                    name TEXT NOT NULL
                );
                "#,
            )?;
            Ok(())
        })
        .unwrap();

        db
    }

    #[test]
    fn test_save_and_get_game() {
        let db = setup_test_db();
        let dao = GamesDao::new(db);

        let game = Game::new("123", "Test Game");
        dao.save_game(&game).unwrap();

        let retrieved = dao.get_game("123").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Game");
    }
}
