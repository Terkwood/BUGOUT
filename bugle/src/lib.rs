extern crate r2d2_redis;
extern crate redis;

pub mod subscriber;

mod env;

#[derive(Copy, Clone)]
pub struct WakeUp;
