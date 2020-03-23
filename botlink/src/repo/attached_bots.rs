use super::redis_keys::KeyProvider;
use super::RepoErr;
use micro_model_moves::{GameId, Player};
use redis::Commands;
use redis_conn_pool::{r2d2, r2d2_redis, redis, Pool};

pub trait AttachedBotsRepo {
    fn is_attached(&self, game_id: &GameId, player: Player) -> Result<bool, RepoErr>;

    fn attach(&mut self, game_id: &GameId, player: Player) -> Result<(), RepoErr>;
}

const TTL_SECS: usize = 86400;

pub struct RedisAttachedBotsRepo {
    pub pool: Pool,
    pub key_provider: KeyProvider,
}

impl AttachedBotsRepo for RedisAttachedBotsRepo {
    fn is_attached(&self, game_id: &GameId, player: Player) -> Result<bool, RepoErr> {
        let mut conn = self.pool.get().expect("pool");
        let result = conn.sismember(
            self.key_provider.attached_bots(),
            member_value(game_id, player),
        )?;
        self.expire(&mut conn)?;
        Ok(result)
    }

    fn attach(&mut self, game_id: &GameId, player: Player) -> Result<(), RepoErr> {
        let mut conn = self.pool.get().expect("pool");
        let result = conn.sadd(
            self.key_provider.attached_bots(),
            member_value(game_id, player),
        )?;
        self.expire(&mut conn)?;
        Ok(result)
    }
}
impl RedisAttachedBotsRepo {
    fn expire(
        &self,
        conn: &mut r2d2::PooledConnection<r2d2_redis::RedisConnectionManager>,
    ) -> Result<(), RepoErr> {
        Ok(conn.expire(self.key_provider.attached_bots(), TTL_SECS)?)
    }
}
fn member_value(game_id: &GameId, player: Player) -> String {
    format!("{}_{}", game_id.0, player.to_string())
}
