use crate::repo::*;
use crate::stream::{XAdd, XRead};
use redis::Client;
use std::rc::Rc;

pub struct Components {
    pub prefs_repo: Rc<dyn PrefsRepo>,
    pub session_game_repo: Rc<dyn SessionGameRepo>,
    pub xread: Box<dyn XRead>,
    pub xadd: Box<dyn XAdd>,
}

impl Components {
    pub fn new(client: &Rc<Client>) -> Self {
        Components {
            prefs_repo: Rc::new(client.clone()),
            session_game_repo: Rc::new(client.clone()),
            xread: Box::new(client.clone()),
            xadd: Box::new(client.clone()),
        }
    }
}

pub struct Repos {
    pub prefs: Rc<dyn PrefsRepo>,
    pub session_game: Rc<dyn SessionGameRepo>,
}

impl From<&Components> for Repos {
    fn from(c: &Components) -> Self {
        Self {
            prefs: c.prefs_repo.clone(),
            session_game: c.session_game_repo.clone(),
        }
    }
}
