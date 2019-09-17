extern crate reaper;

use crossbeam_channel::{unbounded, Receiver, Sender};

use reaper::model::*;
use reaper::*;
const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    println!("🔢 {:<8} {}", NAME, VERSION);

    let (activity_in, activity_out): (Sender<KafkaActivity>, Receiver<KafkaActivity>) = unbounded();

    let (shutdown_in, shutdown_out): (Sender<ShutdownCommand>, Receiver<ShutdownCommand>) =
        unbounded();

    env::init();
    kafka::start(activity_in, shutdown_out.clone());
    monitor::start(shutdown_in, activity_out);
    shutdown::listen(shutdown_out);
}
