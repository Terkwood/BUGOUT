extern crate color_model;
extern crate core_model;
extern crate rand;
extern crate redis_streams;

mod components;
mod repo;
pub mod service;
pub mod stream;

pub use components::Components;

use redis::Client;
use std::rc::Rc;

const REDIS_URL: &str = "redis://redis/";
pub fn create_redis_client() -> Rc<Client> {
    Rc::new(Client::open(REDIS_URL).expect("redis client"))
}
