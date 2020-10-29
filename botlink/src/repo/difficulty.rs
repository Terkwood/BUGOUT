use super::RepoErr;
use bot_model::Difficulty;
use core_model::GameId;

pub trait DifficultyRepo: Send + Sync {
    fn get(&self, game_id: &GameId) -> Result<Option<Difficulty>, RepoErr>;
    fn put(&self, game_id: &GameId, difficulty: Difficulty) -> Result<(), RepoErr>;
}
