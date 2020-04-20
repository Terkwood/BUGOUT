use crate::stream::topics;
use micro_model_bot::gateway::BotAttached;
use micro_model_moves::{Coord, GameId, GameState, MakeMoveCommand};
use redis_conn_pool::redis::RedisError;
use redis_conn_pool::{redis, Pool};

use log::info;
use std::sync::Arc;

pub trait XAdder: Send + Sync {
    fn xadd_game_state(&self, game_id: &GameId, game_state: &GameState) -> Result<(), XAddError>;
    fn xadd_make_move_command(&self, command: &MakeMoveCommand) -> Result<(), XAddError>;
    fn xadd_bot_attached(&self, bot_attached: BotAttached) -> Result<(), XAddError>;
}

#[derive(Debug)]
pub enum XAddError {
    Redis(RedisError),
    Ser(Box<bincode::ErrorKind>),
}

pub struct RedisXAdder {
    pub pool: Arc<Pool>,
}
impl XAdder for RedisXAdder {
    fn xadd_game_state(&self, game_id: &GameId, game_state: &GameState) -> Result<(), XAddError> {
        let mut conn = self.pool.get().expect("redis pool");
        redis::cmd("XADD")
            .arg(topics::GAME_STATES_CHANGELOG)
            .arg("MAXLEN")
            .arg("~")
            .arg("1000")
            .arg("*")
            .arg("game_id")
            .arg(game_id.0.to_string())
            .arg("data")
            .arg(game_state.serialize()?)
            .query::<String>(&mut *conn)?;

        info!(
            "🧳 {} {}",
            &game_id.0.to_string()[0..8],
            game_state.player_up.to_string()
        );

        Ok(())
    }
    fn xadd_make_move_command(&self, command: &MakeMoveCommand) -> Result<(), XAddError> {
        let mut conn = self.pool.get().unwrap();

        let mut redis_cmd = redis::cmd("XADD");
        redis_cmd
            .arg(topics::MAKE_MOVE_CMD)
            .arg("MAXLEN")
            .arg("~")
            .arg("1000")
            .arg("*")
            .arg("game_id")
            .arg(command.game_id.0.to_string())
            .arg("player")
            .arg(command.player.to_string())
            .arg("req_id")
            .arg(command.req_id.0.to_string());
        if let Some(Coord { x, y }) = command.coord {
            redis_cmd.arg("coord_x").arg(x).arg("coord_y").arg(y);
        }
        redis_cmd.query::<String>(&mut *conn)?;

        info!(
            "🏇 {} {}",
            &command.game_id.0.to_string()[0..8],
            command.player.to_string()
        );
        Ok(())
    }
    fn xadd_bot_attached(&self, bot_attached: BotAttached) -> Result<(), XAddError> {
        let mut conn = self.pool.get().expect("redis pool");
        redis::cmd("XADD")
            .arg(topics::BOT_ATTACHED_EV)
            .arg("MAXLEN")
            .arg("~")
            .arg("1000")
            .arg("*")
            .arg("data")
            .arg(bot_attached.serialize()?)
            .query::<String>(&mut *conn)?;
        Ok(())
    }
}

impl From<RedisError> for XAddError {
    fn from(r: RedisError) -> Self {
        XAddError::Redis(r)
    }
}
impl From<Box<bincode::ErrorKind>> for XAddError {
    fn from(b: Box<bincode::ErrorKind>) -> Self {
        XAddError::Ser(b)
    }
}
