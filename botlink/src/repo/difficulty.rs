use super::{expire, RepoErr};
use bot_model::Difficulty;
use core_model::GameId;
use redis::{Client, Commands};
use std::sync::Arc;

pub trait DifficultyRepo: Send + Sync {
    fn get(&self, game_id: &GameId) -> Result<Option<Difficulty>, RepoErr>;
    fn put(&self, game_id: &GameId, difficulty: Difficulty) -> Result<(), RepoErr>;
}

impl DifficultyRepo for Arc<Client> {
    fn get(&self, game_id: &GameId) -> Result<Option<Difficulty>, RepoErr> {
        if let Ok(mut conn) = self.get_connection() {
            let bytes: Option<Vec<u8>> = conn.get(difficulty_key(&game_id))?;
            expire(&difficulty_key(game_id), &mut conn)?;
            Ok(if let Some(b) = bytes {
                bincode::deserialize(&b)?
            } else {
                None
            })
        } else {
            Err(RepoErr::Conn)
        }
    }

    fn put(&self, game_id: &GameId, difficulty: Difficulty) -> Result<(), RepoErr> {
        if let Ok(mut conn) = self.get_connection() {
            let bytes: Vec<u8> = bincode::serialize(&difficulty)?;
            conn.set(difficulty_key(&game_id), bytes)?;
            expire(&difficulty_key(game_id), &mut conn)?;
            Ok(())
        } else {
            Err(RepoErr::Conn)
        }
    }
}

fn difficulty_key(game_id: &GameId) -> String {
    format!("/BUGOUT/botlink/difficulty/{}", game_id.0.to_string())
}
