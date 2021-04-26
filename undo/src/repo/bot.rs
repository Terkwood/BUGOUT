use super::RepoErr;
use core_model::GameId;
use move_model::Player;
use redis::Client;
use std::rc::Rc;

pub trait BotRepo {
    fn get(&self, game_id: &GameId, player: Player) -> Result<bool, RepoErr>;
    fn put(&self, game_id: &GameId, player: Player, is_attached: bool) -> Result<(), RepoErr>;
}

impl BotRepo for Rc<Client> {
    fn get(&self, game_id: &GameId, player: Player) -> Result<bool, RepoErr> {
        todo!()
    }

    fn put(&self, game_id: &GameId, player: Player, is_attached: bool) -> Result<(), RepoErr> {
        todo!()
    }
}
