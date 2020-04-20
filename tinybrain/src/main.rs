extern crate tinybrain;

use crossbeam_channel::{unbounded, Receiver, Sender};
use log::info;
use micro_model_bot::*;
use std::thread;
use tinybrain::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("🔢 {}", VERSION);
    env::init();

    let (compute_move_in, compute_move_out): (Sender<ComputeMove>, Receiver<ComputeMove>) =
        unbounded();
    let (move_computed_in, move_computed_out): (Sender<MoveComputed>, Receiver<MoveComputed>) =
        unbounded();

    thread::spawn(|| katago::start(move_computed_in, compute_move_out));
    websocket::start(compute_move_in, move_computed_out).await;
}
