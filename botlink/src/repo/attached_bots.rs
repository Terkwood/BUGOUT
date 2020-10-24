use super::redis_keys::ATTACHED_BOTS;
use super::RepoErr;
use micro_model_moves::{GameId, Player};
use redis::{Client, Commands};

use std::sync::Arc;

pub trait AttachedBotsRepo: Send + Sync {
    fn is_attached(&self, game_id: &GameId, player: Player) -> Result<bool, RepoErr>;

    fn attach(&mut self, game_id: &GameId, player: Player) -> Result<(), RepoErr>;
}

const TTL_SECS: usize = 86400;

impl AttachedBotsRepo for Arc<Client> {
    fn is_attached(&self, game_id: &GameId, player: Player) -> Result<bool, RepoErr> {
        match self.get_connection() {
            Ok(mut conn) => {
                let result = conn.sismember(ATTACHED_BOTS, member_value(game_id, player))?;
                expire(&mut conn)?;
                Ok(result)
            }
            Err(_) => Err(RepoErr::Conn),
        }
    }

    fn attach(&mut self, game_id: &GameId, player: Player) -> Result<(), RepoErr> {
        match self.get_connection() {
            Ok(mut conn) => {
                let result = conn.sadd(ATTACHED_BOTS, member_value(game_id, player))?;
                expire(&mut conn)?;
                Ok(result)
            }
            Err(_) => Err(RepoErr::Conn),
        }
    }
}

fn expire(conn: &mut redis::Connection) -> Result<(), RepoErr> {
    Ok(conn.expire(ATTACHED_BOTS, TTL_SECS)?)
}

fn member_value(game_id: &GameId, player: Player) -> String {
    format!("{}_{}", game_id.0, player.to_string())
}
