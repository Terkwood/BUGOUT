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
        match self.get_connection() {
            Ok(mut conn) => {
                let key = difficulty_key(game_id);
                let data: Result<Option<Vec<u8>>, _> =
                    conn.get(&key).map_err(|e| RepoErr::Redis(e));

                if data.is_ok() {
                    expire(&key, &mut conn)?
                }

                match data {
                    Ok(Some(bytes)) => {
                        let deser: Result<Difficulty, _> = bincode::deserialize(&bytes);
                        deser.map(|d| Some(d)).map_err(|e| RepoErr::SerDes(e))
                    }
                    Ok(None) => Ok(None),
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(RepoErr::Redis(e)),
        }
    }

    fn put(&self, game_id: &GameId, difficulty: Difficulty) -> Result<(), RepoErr> {
        let key = difficulty_key(&game_id);
        let mut conn = self.get_connection()?;
        let bytes = bincode::serialize(&difficulty)?;
        let done = conn.set(&key, bytes).map_err(|e| RepoErr::Redis(e))?;
        expire(&key, &mut conn)?;
        Ok(done)
    }
}

fn difficulty_key(game_id: &GameId) -> String {
    format!("/BUGOUT/botlink/difficulty/{}", game_id.0.to_string())
}
