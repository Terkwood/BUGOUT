use super::*;
pub trait PrefsRepo {
    fn get(&self, session_id: &SessionId) -> Result<Option<SessionColorPref>, FetchErr>;
    fn put(&self, scp: SessionColorPref) -> Result<(), WriteErr>;
}

impl PrefsRepo for Rc<Client> {
    fn get(&self, session_id: &SessionId) -> Result<Option<SessionColorPref>, FetchErr> {
        todo!("get redis list")
    }

    fn put(&self, scp: SessionColorPref) -> Result<(), WriteErr> {
        todo!("write redis list")
    }
}
