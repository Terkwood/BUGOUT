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
        redis_io::command_writer::process_xadds(redis_commands_out, &redis_io::create_pool())
    });

    let bei = opts.backend_events_in.clone();
    thread::spawn(move || redis_io::stream::process(bei));

    let soc = opts.session_commands_out;
    thread::spawn(move || {
        split_commands(super::split::SplitOpts {
            session_commands_out: soc,
            kafka_commands_in,
            redis_commands_in,
            sb_repo: session_repo::create(redis_io::create_pool()),
            cb_repo: todo!(),
            gb_repo: todo!(),
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
    pub session_commands_out: Receiver<SessionCommands>,
}
