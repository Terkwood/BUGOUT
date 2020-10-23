pub mod commands;
pub mod events;

use crate::channels::MainChannels;
use crate::redis_io::stream;
use std::sync::Arc;
use std::thread;

pub fn start(channels: &MainChannels, redis_client: Arc<redis::Client>) {
    let pool_c = redis_client.clone();
    let c_out = channels.session_commands_out.clone();
    thread::spawn(move || {
        stream::write::start(c_out, &stream::xadd::RedisXAddCommands::create(pool_c))
    });

    let bei = channels.backend_events_in.clone();
    let client_d = redis_client.clone();
    stream::process(
        bei,
        stream::StreamOpts {
            xread: Box::new(stream::xread::RedisXReader {
                client: client_d.clone(),
            }),
            xack: Box::new(client_d),
        },
    )
}
