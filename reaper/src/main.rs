#![feature(bind_by_move_pattern_guards)]
extern crate chrono;
extern crate dotenv;
#[macro_use]
extern crate lazy_static;
extern crate rusoto_core;
extern crate rusoto_ec2;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

mod env;
mod kafka;
mod model;
mod monitor;
mod shutdown;
mod topics;

use crossbeam_channel::{unbounded, Receiver, Sender};

use crate::model::*;
const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    println!("⚰️ {:<8} {}", NAME, VERSION);

    let (activity_in, activity_out): (Sender<KafkaActivity>, Receiver<KafkaActivity>) = unbounded();

    let (shutdown_in, shutdown_out): (Sender<ShutdownCommand>, Receiver<ShutdownCommand>) =
        unbounded();

    env::init();
    kafka::start(activity_in, shutdown_out.clone());
    monitor::start(shutdown_in, activity_out);
    shutdown::listen(shutdown_out);
}
