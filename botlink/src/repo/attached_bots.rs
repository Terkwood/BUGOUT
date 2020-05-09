use super::redis_keys::ATTACHED_BOTS;
use super::RepoErr;
use micro_model_moves::{GameId, Player};
use redis::Commands;
use redis_conn_pool::{r2d2, r2d2_redis, redis, Pool};

use std::sync::Arc;

pub trait AttachedBotsRepo: Send + Sync {
    fn is_attached(&self, game_id: &GameId, player: Player) -> Result<bool, RepoErr>;

    fn attach(&mut self, game_id: &GameId, player: Player) -> Result<(), RepoErr>;
}

const TTL_SECS: usize = 86400;

impl AttachedBotsRepo for Arc<Pool> {
    fn is_attached(&self, game_id: &GameId, player: Player) -> Result<bool, RepoErr> {
        let mut conn = self.get().expect("pool");
        let result = conn.sismember(ATTACHED_BOTS, member_value(game_id, player))?;
        expire(&mut conn)?;
        Ok(result)
    }

    fn attach(&mut self, game_id: &GameId, player: Player) -> Result<(), RepoErr> {
        let mut conn = self.get().expect("pool");
        let result = conn.sadd(ATTACHED_BOTS, member_value(game_id, player))?;
        expire(&mut conn)?;
        Ok(result)
    }
}

fn expire(
    conn: &mut r2d2::PooledConnection<r2d2_redis::RedisConnectionManager>,
) -> Result<(), RepoErr> {
    Ok(conn.expire(ATTACHED_BOTS, TTL_SECS)?)
}

fn member_value(game_id: &GameId, player: Player) -> String {
    format!("{}_{}", game_id.0, player.to_string())
}
