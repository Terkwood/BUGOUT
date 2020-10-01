extern crate micro_sync;

use log::info;
use micro_sync::stream;
use micro_sync::Components;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn main() {
    env_logger::init();
    info!("🔢 {}", VERSION);
    let client = micro_sync::create_redis_client();
    let components = Components::new(&client);
    stream::init::create_consumer_group(&client);
    stream::process(&components)
}
