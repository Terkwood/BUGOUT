use super::redis_keys::KeyProvider;
use micro_model_moves::{GameId, Player};
use redis::Commands;
use redis_conn_pool::{redis, Pool};

pub trait AttachedBotsRepo {
    fn is_attached(&self, game_id: &GameId, player: Player) -> Result<bool, RepoErr>;

    fn attach(&mut self, game_id: &GameId, player: Player) -> Result<(), RepoErr>;
}

pub struct RedisAttachedBotsRepo {
    pub pool: Pool,
    pub key_provider: KeyProvider,
}
impl AttachedBotsRepo for RedisAttachedBotsRepo {
    fn is_attached(&self, game_id: &GameId, player: Player) -> Result<bool, RepoErr> {
        let mut conn = self.pool.get().expect("pool");
        Ok(conn.sismember(
            self.key_provider.attached_bots(),
            member_value(game_id, player),
        )?)
    }

    fn attach(&mut self, game_id: &GameId, player: Player) -> Result<(), RepoErr> {
        todo!()
    }
}

fn member_value(game_id: &GameId, player: Player) -> String {
    format!("{}_{}", game_id.0, player.to_string())
}

#[derive(Debug)]
pub enum RepoErr {
    Redis(redis::RedisError),
}
impl From<redis::RedisError> for RepoErr {
    fn from(r: redis::RedisError) -> Self {
        RepoErr::Redis(r)
    }
}
