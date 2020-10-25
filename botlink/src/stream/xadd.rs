use crate::stream::topics;
use bot_model::api::BotAttached;
use move_model;
use move_model::{Coord, MakeMove};
use redis::Client;
use redis::RedisError;

use log::info;
use std::sync::Arc;

pub trait XAdder: Send + Sync {
    fn xadd_game_state(&self, game_state: &move_model::GameState) -> Result<(), StreamAddError>;
    fn xadd_make_move_command(&self, command: &MakeMove) -> Result<(), StreamAddError>;
    fn xadd_bot_attached(&self, bot_attached: BotAttached) -> Result<(), StreamAddError>;
}

#[derive(Debug)]
pub enum StreamAddError {
    Redis(RedisError),
    Ser(Box<bincode::ErrorKind>),
}

impl XAdder for Arc<Client> {
    fn xadd_game_state(&self, game_state: &move_model::GameState) -> Result<(), StreamAddError> {
        match self.get_connection() {
            Ok(mut conn) => {
                redis::cmd("XADD")
                    .arg(topics::GAME_STATES_CHANGELOG)
                    .arg("MAXLEN")
                    .arg("~")
                    .arg("1000")
                    .arg("*")
                    .arg("data")
                    .arg(game_state.serialize()?)
                    .query::<String>(&mut conn)?;

                info!(
                    "🧳 {} {}",
                    &game_state.game_id.0.to_string()[0..8],
                    game_state.player_up.to_string()
                );

                Ok(())
            }
            Err(e) => Err(StreamAddError::Redis(e)),
        }
    }
    fn xadd_make_move_command(&self, command: &MakeMove) -> Result<(), StreamAddError> {
        match self.get_connection() {
            Ok(mut conn) => {
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
                redis_cmd.query::<String>(&mut conn)?;

                info!(
                    "🏇 {} {}",
                    &command.game_id.0.to_string()[0..8],
                    command.player.to_string()
                );
                Ok(())
            }
            Err(e) => Err(StreamAddError::Redis(e)),
        }
    }

    fn xadd_bot_attached(&self, bot_attached: BotAttached) -> Result<(), StreamAddError> {
        match self.get_connection() {
            Ok(mut conn) => {
                redis::cmd("XADD")
                    .arg(topics::BOT_ATTACHED_EV)
                    .arg("MAXLEN")
                    .arg("~")
                    .arg("1000")
                    .arg("*")
                    .arg("data")
                    .arg(bincode::serialize(&bot_attached)?)
                    .query::<String>(&mut conn)?;
                Ok(())
            }
            Err(e) => Err(StreamAddError::Redis(e)),
        }
    }
}

impl From<RedisError> for StreamAddError {
    fn from(r: RedisError) -> Self {
        StreamAddError::Redis(r)
    }
}
impl From<Box<bincode::ErrorKind>> for StreamAddError {
    fn from(b: Box<bincode::ErrorKind>) -> Self {
        StreamAddError::Ser(b)
    }
}
