use crate::backend_commands::BackendCommands;
use crate::kafka_io;
use crate::redis_io;

use crossbeam_channel::{unbounded, Receiver, Sender};
use futures::executor::block_on;
use std::thread;

use super::*;

pub fn start_all(opts: BackendInitOptions) {
    let (kafka_commands_in, kafka_commands_out): (
        Sender<BackendCommands>,
        Receiver<BackendCommands>,
    ) = unbounded();

    let (redis_commands_in, redis_commands_out): (
        Sender<BackendCommands>,
        Receiver<BackendCommands>,
    ) = unbounded();

    thread::spawn(move || {
        redis_io::start(
            redis_commands_out,
            &redis_io::xadd::RedisXAddCommands::create(),
        )
    });

    let bei = opts.backend_events_in.clone();
    thread::spawn(move || {
        redis_io::stream::process(
            bei,
            redis_io::stream::StreamOpts {
                entry_id_repo: redis_io::entry_id_repo::RedisEntryIdRepo::create_boxed(),
                xreader: Box::new(redis_io::xread::RedisXReader {
                    pool: redis_io::create_pool(),
                }),
            },
        )
    });

    let soc = opts.session_commands_out;
    thread::spawn(move || {
        double_commands(super::doubler::DoublerOpts {
            session_commands_out: soc,
            kafka_commands_in,
            redis_commands_in,
        })
    });

    block_on(kafka_io::start(
        opts.backend_events_in.clone(),
        opts.shutdown_in.clone(),
        opts.kafka_activity_in.clone(),
        kafka_commands_out,
    ))
}

pub struct BackendInitOptions {
    pub backend_events_in: Sender<BackendEvents>,
    pub shutdown_in: Sender<KafkaShutdownEvent>,
    pub kafka_activity_in: Sender<KafkaActivityObserved>,
    pub session_commands_out: Receiver<BackendCommands>,
}
