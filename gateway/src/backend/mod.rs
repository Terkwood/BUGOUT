pub mod commands;
pub mod events;

use crate::channels::MainChannels;
use crate::redis_io;
use std::sync::Arc;
use std::thread;

pub fn start(channels: &MainChannels, redis_client: Arc<redis::Client>) {
    let pool_c = redis_client.clone();
    let c_out = channels.session_commands_out.clone();
    thread::spawn(move || {
        redis_io::start(c_out, &redis_io::xadd::RedisXAddCommands::create(pool_c))
    });

    let bei = channels.backend_events_in.clone();
    let pool_d = redis_client.clone();
    redis_io::stream::process(
        bei,
        redis_io::stream::StreamOpts {
            xreader: Box::new(redis_io::xread::RedisXReader { client: pool_d }),
        },
    )
}
