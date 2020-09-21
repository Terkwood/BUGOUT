extern crate micro_history_provider;

use log::info;
use micro_history_provider::stream;
use micro_history_provider::Components;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn main() {
    env_logger::init();
    info!("🔢 {}", VERSION);
    let components = Components::default();
    stream::create_consumer_group(&components.client);
}
