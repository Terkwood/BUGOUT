use std::borrow::BorrowMut;

use log::info;
use micro_game_lobby::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

use redis_streams::{anyhow, Message, RedisSortedStreams, XId};

fn main() {
    env_logger::init();
    info!("🔢 {}", VERSION);
    let client = redis_client();
    let components = Components::new(client.clone());
    stream::create_consumer_group(&client);

    let mut lobby_streams = stream::LobbyStreams::new(components);

    let mut conn = client.get_connection().expect("redis conn");
    let stream_handlers: Vec<(&str, Box<dyn FnMut(XId, &Message) -> anyhow::Result<()>>)> = vec![
        (
            "some-stream",
            Box::new(|xid, msg| Ok(lobby_streams.consume_fpg(msg))),
        ),
        ("another-stream", todo!()),
        ("fix-the-names", todo!()),
    ];
    let mut sorted_streams =
        RedisSortedStreams::xgroup_create_mkstreams(stream_handlers, todo!("opts"), &mut conn)
            .expect("stream creation");

    lobby_streams.process(&mut sorted_streams)
}
