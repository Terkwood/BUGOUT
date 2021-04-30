use super::{expire, RepoErr};
use core_model::GameId;
use move_model::Player;
use redis::{Client, Commands};
use serde_derive::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum Botness {
    IsBot,
    IsHuman,
}

pub trait BotnessRepo {
    fn get(&self, game_id: &GameId, player: Player) -> Result<Botness, RepoErr>;
    fn put(&self, game_id: &GameId, player: Player, botness: Botness) -> Result<(), RepoErr>;
}

impl BotnessRepo for Rc<Client> {
    fn get(&self, game_id: &GameId, player: Player) -> Result<Botness, RepoErr> {
        let mut conn = self.get_connection()?;
        let key = bot_id(game_id, player);
        let data: Result<Option<Vec<u8>>, _> = conn.get(&key).map_err(|e| RepoErr::Redis(e));

        if data.is_ok() {
            expire(&key, &mut conn)?
        }

        match data {
            Ok(Some(bytes)) => {
                let deser: Result<Botness, _> = bincode::deserialize(&bytes);
                deser.map_err(|e| RepoErr::SerDes(e))
            }
            Ok(None) => Ok(Botness::IsHuman),
            Err(e) => Err(e),
        }
    }

    fn put(&self, game_id: &GameId, player: Player, botness: Botness) -> Result<(), RepoErr> {
        let key = bot_id(&game_id, player);
        let mut conn = self.get_connection()?;
        let bytes = bincode::serialize(&botness)?;
        let done = conn.set(&key, bytes).map_err(|e| RepoErr::Redis(e))?;
        expire(&key, &mut conn)?;
        Ok(done)
    }
}

fn bot_id(game_id: &GameId, player: Player) -> String {
    format!(
        "/BUGOUT/undo/botness/{}_{}",
        game_id.0.to_string(),
        player.to_string()
    )
}
