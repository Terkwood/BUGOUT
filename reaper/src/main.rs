#![feature(bind_by_move_pattern_guards)]
extern crate dotenv;
#[macro_use]
extern crate lazy_static;
extern crate rusoto_core;
extern crate rusoto_ec2;

mod env;
mod shutdown;

use shutdown::shutdown;

fn main() {
    env::init();
    shutdown();
}
