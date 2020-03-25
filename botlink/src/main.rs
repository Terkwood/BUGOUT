extern crate botlink;

use botlink::{stream, websocket};
use log::info;
use std::thread;
const VERSION: &str = env!("CARGO_PKG_VERSION");

use botlink::registry::Components;
fn main() {
    env_logger::init();
    botlink::env::init();
    info!("🔢 {}", VERSION);
    let components = Components::default();
    let ws_opts = websocket::WSOpts::from(&components);
    thread::spawn(move || websocket::listen(ws_opts));
    let mco = components.move_computed_out.clone();
    let xmm = components.xadder_mm.clone();
    thread::spawn(move || stream::write_moves(mco, xmm));
    stream::process(&mut stream::StreamOpts::from(components));
}
