use micro_model_moves::{GameId, GameState, MakeMoveCommand};
use redis_conn_pool::redis::RedisError;
use redis_conn_pool::Pool;
pub trait XAdderGS {
    fn xadd_game_state(&self, game_id: GameId, game_state: GameState) -> Result<(), RedisError>;
}

pub trait XAdderMM: Send + Sync {
    fn xadd_make_move_command(&self, command: MakeMoveCommand) -> Result<(), RedisError>;
}

pub struct RedisXAdderGS {
    pub pool: Pool,
}
impl XAdderGS for RedisXAdderGS {
    fn xadd_game_state(&self, _game_id: GameId, _game_state: GameState) -> Result<(), RedisError> {
        unimplemented!()
    }
}

pub struct RedisXAdderMM {
    pub pool: Pool,
}
impl XAdderMM for RedisXAdderMM {
    fn xadd_make_move_command(&self, _command: MakeMoveCommand) -> Result<(), RedisError> {
        unimplemented!()
    }
}
