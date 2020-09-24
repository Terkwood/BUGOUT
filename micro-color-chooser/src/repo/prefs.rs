use super::*;
pub trait PrefsRepo {
    fn get(&self, game_id: &GameId) -> Result<GameColorPref, FetchErr>;
    fn add(&self, scp: SessionColorPref) -> Result<(), WriteErr>;
}

impl PrefsRepo for Rc<Client> {
    fn get(&self, game_id: &GameId) -> Result<GameColorPref, FetchErr> {
        todo!("get redis list")
    }

    fn add(&self, scp: SessionColorPref) -> Result<(), WriteErr> {
        todo!("write redis list")
    }
}
