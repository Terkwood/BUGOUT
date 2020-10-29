use super::expire;
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
            let result = conn.get(board_size_key(&game_id))?;
            expire(&board_size_key(game_id), &mut conn)?;
            Ok(result)
        } else {
            Err(RepoErr::Conn)
        }
    }
    fn put(&self, game_id: &GameId, board_size: u16) -> Result<(), RepoErr> {
        if let Ok(mut conn) = self.get_connection() {
            conn.set(board_size_key(&game_id), board_size)?;
            expire(&board_size_key(game_id), &mut conn)?;
            Ok(())
        } else {
            Err(RepoErr::Conn)
        }
    }
}

fn board_size_key(game_id: &GameId) -> String {
    format!("/BUGOUT/botlink/board_size/{}", game_id.0.to_string())
}
