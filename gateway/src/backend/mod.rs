pub mod commands;
pub mod events;

use crate::channels::MainChannels;
use crate::redis_io::stream;
use std::sync::Arc;
use std::thread;

pub fn start(channels: &MainChannels, redis_client: Arc<redis::Client>) {
    stream::xread::create_consumer_group(&redis_client);

    let client_c = redis_client.clone();
    let c_out = channels.session_commands_out.clone();
    thread::spawn(move || {
        stream::write_loop(c_out, &stream::xadd::RedisXAddCommands::create(client_c))
    });

    let bei = channels.backend_events_in.clone();
    let client_d = redis_client.clone();
    stream::read_loop(
        bei,
        stream::StreamOpts {
            xread: Box::new(stream::xread::RedisXReader {
                client: client_d.clone(),
            }),
            xack: Box::new(client_d),
        },
    )
}
