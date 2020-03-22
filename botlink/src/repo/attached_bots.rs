use micro_model_moves::{GameId, Player};
use redis_conn_pool::Pool;
pub trait AttachedBotsRepo {
    fn is_attached(&self, game_id: &GameId, player: Player) -> Result<bool, RepoErr>;

    fn attach(&mut self, game_id: &GameId, player: Player) -> Result<(), RepoErr>;
}
#[derive(Debug, Clone)]
pub enum RepoErr {
    Redis,
}

pub struct RedisAttachedBotsRepo {
    pub pool: Pool,
}
impl AttachedBotsRepo for RedisAttachedBotsRepo {
    fn is_attached(&self, _game_id: &GameId, _player: Player) -> Result<bool, RepoErr> {
        todo!()
    }

    fn attach(&mut self, _game_id: &GameId, _player: Player) -> Result<(), RepoErr> {
        todo!()
    }
}
