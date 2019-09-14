#![feature(bind_by_move_pattern_guards)]
extern crate dotenv;
#[macro_use]
extern crate lazy_static;
extern crate rusoto_core;
extern crate rusoto_ec2;

mod env;
mod kafka;
mod shutdown;
mod topics;

use crossbeam_channel::{unbounded, Receiver, Sender};

use crate::kafka::KafkaActivity;

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub struct ShutdownCommand;

fn main() {
    println!("⚰️ {:<8} {}", NAME, VERSION);

    let (activity_in, _): (Sender<KafkaActivity>, Receiver<KafkaActivity>) = unbounded();

    let (shutdown_in, shutdown_out): (Sender<ShutdownCommand>, Receiver<ShutdownCommand>) =
        unbounded();

    env::init();
    kafka::start(activity_in, shutdown_out);
    shutdown::listen(shutdown_in);
}
