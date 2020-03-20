extern crate tinybrain;

use crossbeam_channel::{unbounded, Receiver, Sender};
use micro_model_bot::*;
use std::thread;
use tinybrain::*;

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    println!("🔢 {:<8} {}", NAME, VERSION);
    env::init();

    let (compute_move_in, compute_move_out): (Sender<ComputeMove>, Receiver<ComputeMove>) =
        unbounded();
    let (move_computed_in, move_computed_out): (Sender<MoveComputed>, Receiver<MoveComputed>) =
        unbounded();

    thread::spawn(|| katago::start(move_computed_in, compute_move_out));
    websocket::start(compute_move_in, move_computed_out)
}
