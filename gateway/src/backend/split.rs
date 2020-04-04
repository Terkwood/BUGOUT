use super::*;
use crate::backend_commands::{BackendCommands, SessionCommands};

use crossbeam_channel::{select, Receiver, Sender};
use log::{error, trace};

const FALLBACK: Backend = Backend::Kafka;

pub fn split_commands(opts: SplitOpts) {
    loop {
        select! {
            recv(opts.session_commands_out) -> msg => match msg {
                Ok(SessionCommands::Reconnect { session_id, client_id, game_id }) => {
                    todo!("need to link this game ID to the last known backend available to the CLIENT. ");
                    todo!(" then need to update sb_repo with the found backend ")
                },
                Ok(SessionCommands::StartBotSession { session_id, bot_player: _, board_size: _ }) => {
                    if let Err(e) = opts.sb_repo.assign(&session_id, Backend::RedisStreams) {
                        error!("error in session start {:?}", e)
                    } else {
                        trace!("session started with redis bot backend ");

                        todo!(".... Actually attach the bot using an XADD ...")
                    }
                },
                Ok(SessionCommands::Backend {session_id, command: backend_command}) => {
                    trace!("Hello splitter {:?} ",session_id);
                    if let BackendCommands::SessionDisconnected(crate::backend_commands::SessionDisconnected{session_id}) = backend_command {
                        if let Err(_) = opts.sb_repo.unassign(&session_id) {
                            error!("UNASSIGN ERR")
                        } else {
                            trace!("..unassigned..")
                        }
                    }
                    match opts.sb_repo.backend_for(&session_id) {
                        Ok(Some(backend)) =>
                            split_send(backend,backend_command,&opts.kafka_commands_in,&opts.redis_commands_in)
                        ,
                        Ok(None) => {
                            let cc = backend_command.clone();
                            if let Err(e) = opts.sb_repo.assign(&session_id,  FALLBACK) {
                                error!("error in CHOSEN backend {:?}", e)
                            } else {
                                split_send( FALLBACK,cc,&opts.kafka_commands_in,&opts.redis_commands_in)
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
    pub sb_repo: Box<dyn SessionBackendRepo>,
    pub cb_repo: Box<dyn ClientBackendRepo>,
    pub gb_repo: Box<dyn GameBackendRepo>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    use crossbeam_channel::{select, unbounded};
    use std::thread;

    struct FakeSbRepo {
        sb_calls_in: Sender<SbInvocations>,
    }
    enum SbInvocations {
        BackendFor(SessionId),
        Assign(SessionId, Backend),
        Unassign(SessionId),
        UnassignAll(Backend),
    }
    impl SessionBackendRepo for FakeSbRepo {
        fn backend_for(
            &self,
            session_id: &crate::model::SessionId,
        ) -> Result<Option<Backend>, BackendRepoErr> {
            unimplemented!()
        }
        fn assign(
            &self,
            session_id: &crate::model::SessionId,
            backend: Backend,
        ) -> Result<(), BackendRepoErr> {
            unimplemented!()
        }
        fn unassign(&self, session_id: &crate::model::SessionId) -> Result<(), BackendRepoErr> {
            unimplemented!()
        }
        fn unassign_all(&mut self, backend: Backend) -> Result<(), BackendRepoErr> {
            unimplemented!()
        }
    }
    struct FakeCbRepo {
        cb_calls_in: Sender<CbInvocations>,
    }
    enum CbInvocations {
        Get(ClientId),
        Set(ClientId, Backend),
    }
    impl ClientBackendRepo for FakeCbRepo {
        fn get(&self, client_id: &ClientId) -> Result<Option<Backend>, BackendRepoErr> {
            unimplemented!()
        }
        fn set(&self, client_id: &ClientId, backend: Backend) -> Result<(), BackendRepoErr> {
            unimplemented!()
        }
    }
    struct FakeGbRepo {
        gb_calls_in: Sender<GbInvocations>,
    }
    enum GbInvocations {
        Get(GameId),
        Set(GameId, Backend),
    }
    impl GameBackendRepo for FakeGbRepo {
        fn get(&self, game_id: &GameId) -> Result<Option<Backend>, BackendRepoErr> {
            unimplemented!()
        }
        fn set(&self, game_id: &GameId, backend: Backend) -> Result<(), BackendRepoErr> {
            unimplemented!()
        }
    }

    #[test]
    fn test_split_commands() {
        let (session_commands_in, session_commands_out): (
            Sender<SessionCommands>,
            Receiver<SessionCommands>,
        ) = unbounded();

        let (kafka_commands_in, kafka_commands_out): (
            Sender<BackendCommands>,
            Receiver<BackendCommands>,
        ) = unbounded();

        let (redis_commands_in, redis_commands_out): (
            Sender<BackendCommands>,
            Receiver<BackendCommands>,
        ) = unbounded();

        let (sb_calls_in, sb_calls_out): (Sender<SbInvocations>, Receiver<SbInvocations>) =
            unbounded();
        let (cb_calls_in, cb_calls_out): (Sender<CbInvocations>, Receiver<CbInvocations>) =
            unbounded();
        let (gb_calls_in, gb_calls_out): (Sender<GbInvocations>, Receiver<GbInvocations>) =
            unbounded();

        thread::spawn(move || {
            let opts = SplitOpts {
                kafka_commands_in,
                redis_commands_in,
                session_commands_out,
                sb_repo: Box::new(FakeSbRepo { sb_calls_in }),
                cb_repo: Box::new(FakeCbRepo { cb_calls_in }),
                gb_repo: Box::new(FakeGbRepo { gb_calls_in }),
            };
            split_commands(opts)
        });
        todo!("send stuff");
        todo!("send stuff");
        todo!("send stuff");

        todo!("verify backends are called correctly");
        todo!("verify it gets to the right place");

        select! {
            recv(sb_calls_out) -> _ => todo!(),
            recv(cb_calls_out) -> _ => todo!(),
            recv(gb_calls_out) -> _ => todo!()
        }

        todo!("NO, DO NOT fake kafka backend or redis stream processing");
    }
}
