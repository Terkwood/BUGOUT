extern crate micro_color_chooser;

use log::info;
use micro_color_chooser::stream;
use micro_color_chooser::Components;
use redis_streams::{anyhow, Message, RedisSortedStreams, XId};
use stream::topics;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn main() {
    env_logger::init();
    info!("🔢 {}", VERSION);
    let client = micro_color_chooser::create_redis_client();
    let components = Components::new(&client);

    let mut chooser_streams = stream::ColorChooserStreams::new(components);

    let stream_handlers: Vec<(&str, Box<dyn FnMut(XId, &Message) -> anyhow::Result<()>>)> = vec![
        (
            topics::GAME_READY,
            Box::new(|_xid, msg| chooser_streams.consume_game_ready(msg)),
        ),
        (
            topics::CHOOSE_COLOR_PREF,
            Box::new(|_xid, msg| chooser_streams.consume_choose_color_pref(msg)),
        ),
    ];

    let mut streams =
        RedisSortedStreams::xgroup_create_mkstreams(stream_handlers, &stream::opts(), client)
            .expect("stream creation");

    chooser_streams.process(&mut streams);
}
