use crate::stream::{XAdd, XRead};
use redis::Client;
use std::rc::Rc;

pub struct Components {
    pub xread: Box<dyn XRead>,
    pub xadd: Box<dyn XAdd>,
}

impl Components {
    pub fn new(client: &Rc<Client>) -> Self {
        Components {
            xread: Box::new(client.clone()),
            xadd: Box::new(client.clone()),
        }
    }
}
