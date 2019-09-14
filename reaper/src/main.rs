#![feature(bind_by_move_pattern_guards)]
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
mod shutdown;
mod topics;

use crossbeam_channel::{unbounded, Receiver, Sender};
use serde_derive::{Deserialize, Serialize};

use crate::kafka::KafkaActivity;
use std::time::SystemTime;

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShutdownCommand(pub u128);

impl ShutdownCommand {
    pub fn new() -> ShutdownCommand {
        ShutdownCommand(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Default::default())
                .as_millis(),
        )
    }
}

fn main() {
    println!("⚰️ {:<8} {}", NAME, VERSION);

    let (activity_in, _): (Sender<KafkaActivity>, Receiver<KafkaActivity>) = unbounded();

    let (shutdown_in, shutdown_out): (Sender<ShutdownCommand>, Receiver<ShutdownCommand>) =
        unbounded();

    env::init();
    kafka::start(activity_in, shutdown_out);
    shutdown::listen(shutdown_in);
}
