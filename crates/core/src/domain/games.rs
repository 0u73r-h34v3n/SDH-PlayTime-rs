use std::sync::Arc;

use crate::db::{Database, GamesDao};
use crate::error::Result;
use crate::models::{Game, GameChecksum, GameStatistics};

#[derive(Clone)]
pub struct GamesService {
    dao: GamesDao,
}

impl GamesService {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            dao: GamesDao::new(db),
        }
    }

    /// Get a game by ID
    pub fn get_by_id(&self, game_id: &str) -> Result<Option<GameStatistics>> {
        self.dao.get_game_with_stats(game_id)
    }

    /// Get all games
    pub fn get_all(&self) -> Result<Vec<Game>> {
        self.dao.get_all_games()
    }

    /// Save a game in dictionary
    pub fn save(&self, game: &Game) -> Result<()> {
        self.dao.save_game(game)
    }

    /// Save game checksum
    pub fn save_checksum(&self, checksum: &GameChecksum) -> Result<()> {
        self.dao.save_game_checksum(checksum)
    }

    /// Get checksums for a game
    pub fn get_checksums(&self, game_id: &str) -> Result<Vec<GameChecksum>> {
        self.dao.get_game_checksums(game_id)
    }
}
