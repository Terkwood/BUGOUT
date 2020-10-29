use super::{expire, RepoErr};
use bot_model::Difficulty;
use core_model::GameId;
use redis::{Client, Commands};

pub trait DifficultyRepo: Send + Sync {
    fn get(&self, game_id: &GameId) -> Result<Option<Difficulty>, RepoErr>;
    fn put(&self, game_id: &GameId, difficulty: Difficulty) -> Result<(), RepoErr>;
}

impl DifficultyRepo for Box<Client> {
    fn get(&self, game_id: &GameId) -> Result<Option<Difficulty>, RepoErr> {
        if let Ok(mut conn) = self.get_connection() {
            let ser: Option<Vec<u8>> = conn.get(difficulty_key(&game_id))?;
            expire(&difficulty_key(game_id), &mut conn)?;
            Ok(todo!())
        } else {
            Err(RepoErr::Conn)
        }
    }

    fn put(&self, game_id: &GameId, difficulty: Difficulty) -> Result<(), RepoErr> {
        todo!()
    }
}

fn difficulty_key(game_id: &GameId) -> String {
    format!("/BUGOUT/botlink/difficulty/{}", game_id.0.to_string())
}
