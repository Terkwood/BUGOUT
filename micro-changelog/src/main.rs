use micro_changelog::stream;
use micro_changelog::Components;
use stream::StreamTopics;

use log::info;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    env_logger::init();
    info!("🔢 {}", VERSION);
    let topics = StreamTopics::default();
    stream::create_consumer_group(&topics);
    stream::process(topics, &Components::default())
}
