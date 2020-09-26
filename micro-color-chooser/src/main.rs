extern crate micro_color_chooser;

use log::info;
use micro_color_chooser::stream;
use micro_color_chooser::Components;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn main() {
    env_logger::init();
    info!("🔢 {}", VERSION);
    let client = micro_color_chooser::create_redis_client();
    let mut components = Components::new(&client);
    stream::create_consumer_group(&client);
    stream::process(&mut components)
}
