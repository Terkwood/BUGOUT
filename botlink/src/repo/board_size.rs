use super::redis_keys::board_size as board_size_key;
use super::RepoErr;
use core_model::GameId;
use redis::{Client, Commands};
use std::sync::Arc;

pub trait BoardSizeRepo: Send + Sync {
    fn get(&self, game_id: &GameId) -> Result<u16, RepoErr>;

    fn put(&self, game_id: &GameId, board_size: u16) -> Result<(), RepoErr>;
}

impl BoardSizeRepo for Arc<Client> {
    fn get(&self, game_id: &GameId) -> Result<u16, RepoErr> {
        if let Ok(mut conn) = self.get_connection() {
            let result = conn.get(board_size_key(&game_id.0))?;
            expire(game_id, &mut conn)?;
            Ok(result)
        } else {
            Err(RepoErr::Conn)
        }
    }
    fn put(&self, game_id: &GameId, board_size: u16) -> Result<(), RepoErr> {
        if let Ok(mut conn) = self.get_connection() {
            conn.set(board_size_key(&game_id.0), board_size)?;
            expire(game_id, &mut conn)?;
            Ok(())
        } else {
            Err(RepoErr::Conn)
        }
    }
}

const TTL_SECS: usize = 86400;
fn expire(game_id: &GameId, conn: &mut redis::Connection) -> Result<(), RepoErr> {
    Ok(conn.expire(board_size_key(&game_id.0), TTL_SECS)?)
}
