use crate::repo::*;
use crate::service::Random;
use crate::stream::{XAdd, XRead};
use redis::Client;
use std::rc::Rc;

pub struct Components {
    pub prefs_repo: Rc<dyn PrefsRepo>,
    pub game_ready_repo: Rc<dyn GameReadyRepo>,
    pub xread: Box<dyn XRead>,
    pub xadd: Box<dyn XAdd>,
    pub random: Random,
}

impl Components {
    pub fn new(client: &Rc<Client>) -> Self {
        Components {
            prefs_repo: Rc::new(client.clone()),
            game_ready_repo: Rc::new(client.clone()),
            xread: Box::new(client.clone()),
            xadd: Box::new(client.clone()),
            random: Random::new(),
        }
    }
}

pub struct Repos {
    pub prefs: Rc<dyn PrefsRepo>,
    pub game_ready: Rc<dyn GameReadyRepo>,
}

impl Repos {
    pub fn new(c: &mut Components) -> Self {
        Self {
            prefs: c.prefs_repo.clone(),
            game_ready: c.game_ready_repo.clone(),
        }
    }
}
