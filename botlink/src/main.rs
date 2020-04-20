extern crate botlink;
use botlink::{stream, websocket};
use log::info;
use std::thread;
const VERSION: &str = env!("CARGO_PKG_VERSION");

use botlink::registry::Components;

#[tokio::main]
async fn main() {
    env_logger::init();
    botlink::env::init();
    info!("🔢 {}", VERSION);

    let components = Components::default();
    let ws_opts = websocket::WSOpts::from(&components);
    let mco = components.move_computed_out.clone();
    let xmm = components.xadder.clone();
    let bsr = components.board_size_repo.clone();

    thread::spawn(move || stream::write_moves(mco, xmm, bsr));
    thread::spawn(move || stream::process(&mut stream::StreamOpts::from(components)));
    websocket::listen(ws_opts).await;
}
