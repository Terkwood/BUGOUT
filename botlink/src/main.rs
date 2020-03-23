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
    stream::process(
        stream::topics::Topics::default(),
        &mut stream::StreamOpts::from(components),
    );
}
