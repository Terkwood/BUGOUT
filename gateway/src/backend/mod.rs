pub mod commands;
pub mod events;

use crate::channels::MainChannels;
use crate::redis_io;
use redis_io::RedisPool;
use std::sync::Arc;
use std::thread;

pub fn start(channels: &MainChannels, redis_pool: Arc<RedisPool>) {
    let pool_c = redis_pool.clone();
    let c_out = channels.session_commands_out.clone();
    thread::spawn(move || {
        redis_io::start(c_out, &redis_io::xadd::RedisXAddCommands::create(pool_c))
    });

    let bei = channels.backend_events_in.clone();
    let pool_d = redis_pool.clone();
    thread::spawn(move || {
        redis_io::stream::process(
            bei,
            redis_io::stream::StreamOpts {
                entry_id_repo: redis_io::entry_id_repo::RedisEntryIdRepo::create_boxed(
                    pool_d.clone(),
                ),
                xreader: Box::new(redis_io::xread::RedisXReader { pool: pool_d }),
            },
        )
    });
}
