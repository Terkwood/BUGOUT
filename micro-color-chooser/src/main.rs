extern crate micro_color_chooser;

use log::info;
use micro_color_chooser::stream;
use micro_color_chooser::Components;
use redis_streams::{anyhow, Message, RedisSortedStreams, XId};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn main() {
    env_logger::init();
    info!("🔢 {}", VERSION);
    let client = micro_color_chooser::create_redis_client();
    let mut components = Components::new(&client);

    let stream_handlers: Vec<(&str, Box<dyn FnMut(XId, &Message) -> anyhow::Result<()>>)> = todo!();

    let mut streams =
        RedisSortedStreams::xgroup_create_mkstreams(stream_handlers, &stream::opts(), client)
            .expect("stream creation");

    stream::process(&mut components)
}
