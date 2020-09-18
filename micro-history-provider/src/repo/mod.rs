use redis::Client;
use std::rc::Rc;
pub trait HistoryRepo {}
impl HistoryRepo for Rc<Client> {}
