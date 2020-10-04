extern crate core_model;
extern crate move_model;
extern crate redis_streams;
extern crate sync_model;

mod components;
pub mod player;
mod repo;
pub mod stream;
mod sync;
mod time;
mod to_history;

pub use components::Components;
pub use to_history::ToHistory;

use redis::Client;
use std::rc::Rc;

const REDIS_URL: &str = "redis://redis/";
pub fn create_redis_client() -> Rc<Client> {
    Rc::new(Client::open(REDIS_URL).expect("redis client"))
}
