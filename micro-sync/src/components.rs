use crate::repo::{HistoryRepo, ReplyOnMoveRepo};
use crate::stream::{XAdd, XRead};
use redis::Client;
use std::rc::Rc;

pub struct Components {
    pub history_repo: Box<dyn HistoryRepo>,
    pub reply_repo: Box<dyn ReplyOnMoveRepo>,
    pub xread: Box<dyn XRead>,
    pub xadd: Box<dyn XAdd>,
}

impl Components {
    pub fn new(client: &Rc<Client>) -> Self {
        Components {
            history_repo: Box::new(client.clone()),
            reply_repo: Box::new(client.clone()),
            xread: Box::new(client.clone()),
            xadd: Box::new(client.clone()),
        }
    }
}
