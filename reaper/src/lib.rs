#![feature(bind_by_move_pattern_guards)]
extern crate chrono;
extern crate dotenv;
#[macro_use]
extern crate lazy_static;
extern crate rusoto_core;
extern crate rusoto_ec2;
extern crate rusoto_sts;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

pub mod env;
pub mod kafka;
pub mod model;
pub mod monitor;
pub mod shutdown;
mod topics;
