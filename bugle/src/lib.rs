extern crate redis;

pub mod subscriber;

mod env;

#[derive(Copy, Clone)]
pub struct WakeUp;
