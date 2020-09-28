extern crate micro_history_provider;

use log::info;
use micro_history_provider::stream;
use micro_history_provider::Components;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn main() {
    env_logger::init();
    info!("🔢 {}", VERSION);
    let client = micro_history_provider::create_redis_client();
    let components = Components::new(&client);
    stream::create_consumer_group(&client);
    stream::process(&components)
}
