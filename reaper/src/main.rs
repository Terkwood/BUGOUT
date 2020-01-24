extern crate reaper;

use crossbeam_channel::{unbounded, Receiver, Sender};
use futures::executor::block_on;

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
    println!("☠️ REAP after {} seconds", *env::ALLOWED_IDLE_SECS);

    monitor::start(shutdown_in, activity_out);
    let kafka_shutdown_out = shutdown_out.clone();
    std::thread::spawn(move || shutdown::listen(shutdown_out));
    block_on(kafka::start(activity_in, kafka_shutdown_out));
}
