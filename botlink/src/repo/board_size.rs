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
        todo!()
    }
    fn set(&mut self, game_id: &GameId, board_size: u16) -> Result<(), RepoErr> {
        todo!()
    }
}
