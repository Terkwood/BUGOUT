use redis::Client;
use std::rc::Rc;
pub trait XAdd {}
impl XAdd for Rc<Client> {}
