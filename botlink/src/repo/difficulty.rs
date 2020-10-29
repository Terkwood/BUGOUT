use super::RepoErr;
use bot_model::Difficulty;
use core_model::GameId;
pub trait DifficultyRepo {
    fn get(game_id: &GameId) -> Result<Option<Difficulty>, RepoErr>;
    fn put(game_id: &GameId, difficulty: Difficulty) -> Result<(), RepoErr>;
}
