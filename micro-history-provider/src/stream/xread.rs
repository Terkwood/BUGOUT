use redis::Client;
use std::rc::Rc;
pub trait XRead {}
impl XRead for Rc<Client> {}
