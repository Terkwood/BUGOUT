use log::info;
use micro_game_lobby::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

use redis_streams::{anyhow, Message, RedisSortedStreams, XId};

fn main() {
    env_logger::init();
    info!("🔢 {}", VERSION);
    let client = redis_client();
    let components = Components::new(client.clone());

    let lobby = stream::LobbyStreams::new(components);

    let mut conn = client.get_connection().expect("redis conn");
    let stream_handlers: Vec<(&str, Box<dyn FnMut(XId, &Message) -> anyhow::Result<()>>)> = vec![
        (
            topics::FIND_PUBLIC_GAME,
            Box::new(|_xid, msg| lobby.consume_fpg(msg)),
        ),
        (
            topics::JOIN_PRIVATE_GAME,
            Box::new(|_xid, msg| lobby.consume_jpg(msg)),
        ),
        (
            topics::CREATE_GAME,
            Box::new(|_xid, msg| lobby.consume_cg(msg)),
        ),
        (
            topics::SESSION_DISCONNECTED,
            Box::new(|_xid, msg| lobby.consume_sd(msg)),
        ),
    ];
    let mut streams =
        RedisSortedStreams::xgroup_create_mkstreams(stream_handlers, &stream::opts(), &mut conn)
            .expect("stream creation");

    lobby.process(&mut streams)
}
