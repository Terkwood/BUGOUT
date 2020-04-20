use super::redis_keys::KeyProvider;
use super::RepoErr;
use micro_model_moves::{GameId, Player};
use redis::Commands;
use redis_conn_pool::{r2d2, r2d2_redis, redis, Pool};
use std::sync::Arc;

pub trait BoardSizeRepo: Send + Sync {
    fn get(&self, game_id: &GameId) -> Result<u16, RepoErr>;

    fn set(&mut self, game_id: &GameId, board_size: u16) -> Result<(), RepoErr>;
}

pub struct RedisBoardSizeRepo {
    pub pool: Arc<Pool>,
    pub key_provider: KeyProvider,
}

impl BoardSizeRepo for RedisBoardSizeRepo {
    fn get(&self, game_id: &GameId) -> Result<u16, RepoErr> {
        let mut conn = self.pool.get().expect("pool");
        let result = conn.get(self.key_provider.board_size(&game_id.0))?;
        self.expire(game_id, &mut conn)?;
        Ok(result)
    }
    fn set(&mut self, game_id: &GameId, board_size: u16) -> Result<(), RepoErr> {
        let mut conn = self.pool.get().expect("pool");
        let result = conn.set(self.key_provider.board_size(&game_id.0), board_size)?;
        self.expire(game_id, &mut conn)?;
        Ok(result)
    }
}

const TTL_SECS: usize = 86400;

impl RedisBoardSizeRepo {
    fn expire(
        &self,
        game_id: &GameId,
        conn: &mut r2d2::PooledConnection<r2d2_redis::RedisConnectionManager>,
    ) -> Result<(), RepoErr> {
        Ok(conn.expire(self.key_provider.board_size(&game_id.0), TTL_SECS)?)
    }
}
