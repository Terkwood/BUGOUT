extern crate botlink;
use botlink::{stream, websocket};
use log::info;
use std::thread;
const VERSION: &str = env!("CARGO_PKG_VERSION");

use botlink::registry::{create_redis_client, Components};

#[tokio::main]
async fn main() {
    env_logger::init();
    botlink::env::init();
    info!("🔢 {}", VERSION);

    let client = create_redis_client();

    stream::init::create_consumer_group(&client);

    let components = Components::new(client);
    let ws_opts = websocket::WSOpts::from(&components);
    let mco = components.move_computed_out.clone();
    let xmm = components.xadder.clone();
    let bsr = components.board_size_repo.clone();

    thread::spawn(move || stream::xadd_loop(mco, xmm, bsr));
    thread::spawn(move || stream::xread_loop(&mut stream::StreamOpts::from(components)));
    websocket::listen(ws_opts).await;
}
