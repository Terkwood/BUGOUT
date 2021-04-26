use super::{expire, RepoErr};
use core_model::GameId;
use move_model::Player;
use redis::{Client, Commands};
use std::rc::Rc;

pub trait BotRepo {
    fn get(&self, game_id: &GameId, player: Player) -> Result<bool, RepoErr>;
    fn put(&self, game_id: &GameId, player: Player) -> Result<(), RepoErr>;
}

impl BotRepo for Rc<Client> {
    fn get(&self, game_id: &GameId, player: Player) -> Result<bool, RepoErr> {
        let mut conn = self.get_connection()?;
        let key = bot_id(game_id, player);
        let data: Result<Option<Vec<u8>>, _> = conn.get(&key).map_err(|e| RepoErr::Redis(e));

        if data.is_ok() {
            expire(&key, &mut conn)?
        }

        match data {
            Ok(Some(bytes)) => {
                let deser: Result<bool, _> = bincode::deserialize(&bytes);
                deser.map_err(|e| RepoErr::SerDes(e))
            }
            Ok(None) => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn put(&self, game_id: &GameId, player: Player) -> Result<(), RepoErr> {
        let key = bot_id(&game_id, player);
        let mut conn = self.get_connection()?;
        let bytes = bincode::serialize(&true)?;
        let done = conn.set(&key, bytes).map_err(|e| RepoErr::Redis(e))?;
        expire(&key, &mut conn)?;
        Ok(done)
    }
}

fn bot_id(game_id: &GameId, player: Player) -> String {
    format!(
        "/BUGOUT/undo/bot/{}_{}",
        game_id.0.to_string(),
        player.to_string()
    )
}
