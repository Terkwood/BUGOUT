use futures::executor::block_on;

use crate::backend_commands::{BackendCommands, SessionCommands};
use crate::backend_events::{BackendEvents, KafkaShutdownEvent};
use crate::idle_status::KafkaActivityObserved;
use crate::kafka_io;
use crate::redis_io;

use crossbeam_channel::{select, unbounded, Receiver, Sender};
use log::{error, trace};
use std::thread;

pub use client_repo::ClientBackendRepo;
pub use game_repo::GameBackendRepo;
pub use session_repo::SessionBackendRepo;

pub mod client_repo;
pub mod game_repo;
pub mod session_repo;

mod choose;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Backend {
    RedisStreams,
    Kafka,
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Backend::RedisStreams => "rs",
                Backend::Kafka => "k",
            }
        )
    }
}

pub struct BackendInitOptions {
    pub backend_events_in: Sender<BackendEvents>,
    pub shutdown_in: Sender<KafkaShutdownEvent>,
    pub kafka_activity_in: Sender<KafkaActivityObserved>,
    pub session_commands_out: Receiver<SessionCommands>,
}

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
        split_commands(SplitOpts {
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

fn split_commands(opts: SplitOpts) {
    loop {
        select! {
            recv(opts.session_commands_out) -> msg => match msg {
                Ok(SessionCommands::StartBotSession { session_id, bot_player: _, board_size: _ }) => {
                    if let Err(e) = opts.sb_repo.assign(&session_id, Backend::RedisStreams) {
                        error!("error in session start {:?}", e)
                    } else {
                        trace!("session started with redis bot backend ");

                        todo!(".... Actually attach the bot using an XADD ...")
                    }
                },
                Ok(SessionCommands::Backend {session_id, command}) => {
                    trace!("Hello splitter {:?} ",session_id);
                    if let BackendCommands::SessionDisconnected(crate::backend_commands::SessionDisconnected{session_id}) = command {
                        if let Err(_) = opts.sb_repo.unassign(&session_id) {
                            error!("UNASSIGN ERR")
                        } else {
                            trace!("..unassigned..")
                        }
                    }
                    match opts.sb_repo.backend_for(&session_id) {
                        Ok(Some(backend)) =>
                            split_send(backend,command,&opts.kafka_commands_in,&opts.redis_commands_in)
                        ,
                        Ok(None) => {
                            let cc = command.clone();
                            let chosen_backend = choose::fallback(&SessionCommands::Backend {
                                session_id, command });
                            if let Err(e) = opts.sb_repo.assign(&session_id, chosen_backend) {
                                error!("error in CHOSEN backend {:?}", e)
                            } else {
                                split_send(chosen_backend,cc,&opts.kafka_commands_in,&opts.redis_commands_in)
                            }
                        },
                        Err(_) => error!("backend fetch err")
                    }
                },
                Err(e) => error!("session command out: {:?}",e)
            }
        }
    }
}

pub struct SplitOpts {
    pub session_commands_out: Receiver<SessionCommands>,
    pub kafka_commands_in: Sender<BackendCommands>,
    pub redis_commands_in: Sender<BackendCommands>,
    sb_repo: Box<dyn SessionBackendRepo>,
    cb_repo: Box<dyn ClientBackendRepo>,
    gb_repo: Box<dyn GameBackendRepo>,
}

fn split_send(
    backend: Backend,
    command: BackendCommands,
    kafka_commands_in: &Sender<BackendCommands>,
    redis_commands_in: &Sender<BackendCommands>,
) {
    if let Err(e) = match backend {
        Backend::RedisStreams => redis_commands_in.send(command),
        Backend::Kafka => kafka_commands_in.send(command),
    } {
        error!("FAILED SPLIT TO BACKEND {:?}", e)
    } else {
        trace!("..split ok..")
    }
}
