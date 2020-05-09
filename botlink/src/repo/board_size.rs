use super::redis_keys::board_size as board_size_key;
use super::RepoErr;
use micro_model_moves::GameId;
use redis::Commands;
use redis_conn_pool::{r2d2, r2d2_redis, redis, Pool};
use std::sync::Arc;

pub trait BoardSizeRepo: Send + Sync {
    fn get_board_size(&self, game_id: &GameId) -> Result<u16, RepoErr>;

    fn set_board_size(&self, game_id: &GameId, board_size: u16) -> Result<(), RepoErr>;
}

impl BoardSizeRepo for Arc<Pool> {
    fn get_board_size(&self, game_id: &GameId) -> Result<u16, RepoErr> {
        let mut conn = self.get().expect("pool");
        let result = conn.get(board_size_key(&game_id.0))?;
        expire(game_id, &mut conn)?;
        Ok(result)
    }
    fn set_board_size(&self, game_id: &GameId, board_size: u16) -> Result<(), RepoErr> {
        let mut conn = self.get().expect("pool");
        conn.set(board_size_key(&game_id.0), board_size)?;
        expire(game_id, &mut conn)?;
        Ok(())
    }
}

const TTL_SECS: usize = 86400;
fn expire(
    game_id: &GameId,
    conn: &mut r2d2::PooledConnection<r2d2_redis::RedisConnectionManager>,
) -> Result<(), RepoErr> {
    Ok(conn.expire(board_size_key(&game_id.0), TTL_SECS)?)
}
