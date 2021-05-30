mod xadd;

use redis_streams::ConsumerGroupOpts;
use redis_streams::SortedStreams;
pub use xadd::*;

use crate::components::Components;
use crate::game_lobby::GameLobbyOps;
use crate::PUBLIC_GAME_BOARD_SIZE;
use core_model::*;
use lobby_model::api::*;
use lobby_model::*;
use log::{error, trace};
use move_model::GameState;
use redis_streams::Message;

pub const GROUP_NAME: &str = "micro-game-lobby";

#[derive(Debug, Clone, PartialEq)]
pub enum StreamOutput {
    WFO(WaitForOpponent),
    GR(GameReady),
    PGR(PrivateGameRejected),
    LOG(GameState),
}

pub struct LobbyStreams {
    pub reg: Components,
}

use redis_streams::Group;
const BLOCK_MS: usize = 5000;
pub fn opts() -> ConsumerGroupOpts {
    ConsumerGroupOpts {
        block_ms: BLOCK_MS,
        group: Group {
            group_name: GROUP_NAME.to_string(),
            consumer_name: "singleton".to_string(),
        },
    }
}

use redis_streams::anyhow;
impl LobbyStreams {
    pub fn new(reg: Components) -> Self {
        Self { reg }
    }

    pub fn process(&self, streams: &mut dyn SortedStreams) {
        loop {
            if let Err(e) = streams.consume() {
                error!("Stream err {:?}", e)
            }
        }
    }

    pub fn consume_fpg(&self, msg: &Message) -> anyhow::Result<()> {
        let maybe_value = msg.get("data");
        Ok(if let Some(redis::Value::Data(data)) = maybe_value {
            let fpg: FindPublicGame = bincode::deserialize(&data)?;
            let reg = &self.reg;
            let visibility = Visibility::Public;
            let session_id = &fpg.session_id;
            if let Ok(lobby) = reg.game_lobby_repo.get() {
                if let Some(queued) = lobby
                    .games
                    .iter()
                    .find(|g| g.visibility == Visibility::Public)
                {
                    ready_game(session_id, &lobby, queued, &reg)
                } else {
                    let game_id = GameId::new();
                    let updated: GameLobby = lobby.open(Game {
                        board_size: PUBLIC_GAME_BOARD_SIZE,
                        creator: session_id.clone(),
                        visibility,
                        game_id: game_id.clone(),
                    });
                    if let Err(_) = reg.game_lobby_repo.put(&updated) {
                        error!("game lobby write F2");
                    } else {
                        if let Err(_) = reg.xadd.xadd(StreamOutput::WFO(WaitForOpponent {
                            event_id: EventId::new(),
                            game_id,
                            session_id: session_id.clone(),
                            visibility,
                        })) {
                            error!("XADD: Wait for oppo")
                        } else {
                            trace!("Public game open. Lobby: {:?}", &updated)
                        }
                    }
                }
            } else {
                error!("Failed to fetch game lobby: FPG")
            }
        } else {
            error!("could not deserialize FPG data field")
        })
    }

    pub fn consume_cg(&self, msg: &Message) -> anyhow::Result<()> {
        let maybe_value = msg.get("data");
        Ok(if let Some(redis::Value::Data(data)) = maybe_value {
            let cg: CreateGame = bincode::deserialize(&data)?;

            let session_id = &cg.session_id;
            let game_id = cg.game_id.clone().unwrap_or(GameId::new());
            if let Ok(lobby) = self.reg.game_lobby_repo.get() {
                let updated: GameLobby = lobby.open(Game {
                    game_id: game_id.clone(),
                    board_size: cg.board_size,
                    creator: session_id.clone(),
                    visibility: cg.visibility,
                });
                if let Err(_) = self.reg.game_lobby_repo.put(&updated) {
                    error!("game lobby write F1");
                } else {
                    if let Err(_) = self.reg.xadd.xadd(StreamOutput::WFO(WaitForOpponent {
                        game_id: game_id.clone(),
                        session_id: session_id.clone(),
                        event_id: EventId::new(),
                        visibility: cg.visibility,
                    })) {
                        error!("XADD Game ready")
                    } else {
                        trace!("Game created. Lobby: {:?}", &updated);
                    }
                }
            } else {
                error!("CG GAME REPO GET")
            }
        } else {
            error!("could not deser create game data field")
        })
    }

    /// Consumes the command to join a private game.
    /// In the event that the game is invalid,
    /// we will simply log a warning.
    /// Consider implementing logic related to handling
    /// private game rejection: https://github.com/Terkwood/BUGOUT/issues/304
    pub fn consume_jpg(&self, msg: &Message) -> anyhow::Result<()> {
        let maybe_value = msg.get("data");
        Ok(if let Some(redis::Value::Data(data)) = maybe_value {
            let jpg: JoinPrivateGame = bincode::deserialize(&data)?;

            let reg = &self.reg;
            if let Ok(lobby) = reg.game_lobby_repo.get() {
                if let Some(queued) = lobby
                    .games
                    .iter()
                    .find(|g| g.visibility == Visibility::Private && g.game_id == jpg.game_id)
                {
                    ready_game(&jpg.session_id, &lobby, queued, reg)
                } else {
                    if let Err(e) = reg.xadd.xadd(StreamOutput::PGR(PrivateGameRejected {
                        client_id: jpg.client_id.clone(),
                        event_id: EventId::new(),
                        game_id: jpg.game_id.clone(),
                        session_id: jpg.session_id.clone(),
                    })) {
                        error!("Error writing private game rejection to stream {:?}", e)
                    }
                }
            } else {
                error!("game lobby JPG get")
            }
        } else {
            error!("could not consume JoinPrivateGame data field")
        })
    }

    pub fn consume_sd(&self, msg: &Message) -> anyhow::Result<()> {
        let maybe_value = msg.get("data");
        Ok(if let Some(redis::Value::Data(data)) = maybe_value {
            let sd: SessionDisconnected = bincode::deserialize(&data)?;
            let reg = &self.reg;
            if let Ok(game_lobby) = reg.game_lobby_repo.get() {
                let updated: GameLobby = game_lobby.abandon(&sd.session_id);
                if let Err(_) = reg.game_lobby_repo.put(&updated) {
                    error!("game lobby write F1");
                } else {
                    trace!("session {} abandoned: {:?}", sd.session_id.0, &updated);
                }
            } else {
                error!("SD GAME REPO GET")
            }
        } else {
            error!("could not deser session disconn data field")
        })
    }
}
fn ready_game(session_id: &SessionId, lobby: &GameLobby, queued: &Game, reg: &Components) {
    let updated: GameLobby = lobby.ready(queued);
    if let Err(_) = reg.game_lobby_repo.put(&updated) {
        error!("game lobby write F1");
    } else {
        if let Err(_) = reg.xadd.xadd(StreamOutput::GR(GameReady {
            game_id: queued.game_id.clone(),
            event_id: EventId::new(),
            board_size: queued.board_size,
            sessions: (queued.creator.clone(), session_id.clone()),
        })) {
            error!("XADD Game ready")
        } else {
            trace!("Game ready. Lobby: {:?}", &updated);
            init_changelog(&queued.game_id, queued.board_size, &reg)
        }
    }
}

fn init_changelog(game_id: &GameId, board_size: u16, reg: &Components) {
    if let Err(chgerr) = reg.xadd.xadd(StreamOutput::LOG(GameState {
        game_id: game_id.clone(),
        board: move_model::Board {
            size: board_size,
            ..Default::default()
        },
        captures: move_model::Captures::default(),
        turn: 1,
        moves: vec![],
        player_up: move_model::Player::BLACK,
    })) {
        error!("could not write game state changelog {:?}", chgerr)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::components::Components;
    use crate::repo::*;
    use crossbeam_channel::{select, unbounded, Sender};
    use redis_streams::anyhow;
    use redis_streams::SortedStreamGuts;
    use redis_streams::StreamHandler;
    use redis_streams::StreamMessage;
    use redis_streams::XAck;
    use redis_streams::XId;
    use redis_streams::XReadGroupSorted;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use std::thread;

    struct FakeSortedStreams<F>
    where
        F: FnMut(XId, &Message) -> anyhow::Result<()>,
    {
        pub guts: SortedStreamGuts<F>,
    }
    impl<F> SortedStreams for FakeSortedStreams<F>
    where
        F: FnMut(XId, &Message) -> anyhow::Result<()>,
    {
        fn consume(&mut self) -> anyhow::Result<()> {
            self.guts.consume()
        }
    }

    struct FakeGameLobbyRepo {
        pub contents: Arc<Mutex<GameLobby>>,
    }

    impl GameLobbyRepo for FakeGameLobbyRepo {
        fn get(&self) -> Result<GameLobby, FetchErr> {
            Ok(self.contents.lock().expect("mutex lock").clone())
        }
        fn put(&self, game_lobby: &GameLobby) -> Result<(), WriteErr> {
            let mut data = self.contents.lock().expect("lock");
            *data = game_lobby.clone();
            Ok(())
        }
    }

    struct FakeXRead {
        sorted_data: Arc<Mutex<Vec<(XId, StreamMessage)>>>,
    }
    impl XReadGroupSorted for FakeXRead {
        fn read(
            &mut self,
            _stream_names: &[String],
        ) -> anyhow::Result<Vec<(redis_streams::XId, StreamMessage)>> {
            {
                let mut data = self.sorted_data.lock().expect("lock");
                if data.is_empty() {
                    // stop the test thread from spinning like crazy
                    std::thread::sleep(Duration::from_millis(1));
                    Ok(vec![])
                } else {
                    let result = data.clone();
                    *data = vec![];
                    Ok(result)
                }
            }
        }
    }
    struct FakeXAdd(Sender<StreamOutput>);
    impl XAdd for FakeXAdd {
        fn xadd(&self, data: StreamOutput) -> Result<(), XAddErr> {
            if let Err(_) = self.0.send(data) {}
            Ok(())
        }
    }

    struct ItWasAcked;
    struct FakeXAck(Sender<ItWasAcked>);
    impl XAck for FakeXAck {
        fn ack(&mut self, _stream_name: &str, _ids: &[String]) -> anyhow::Result<()> {
            Ok(())
        }
    }
    #[test]
    fn test_process() {
        let (xadd_in, xadd_out) = unbounded();
        let (xack_in, _) = unbounded();

        let sorted_fake_stream = Arc::new(Mutex::new(vec![]));

        let timeout = Duration::from_millis(160);

        // set up a loop to process game lobby requests
        let fake_game_lobby_contents = Arc::new(Mutex::new(GameLobby::default()));

        let sfs = sorted_fake_stream.clone();
        let fgl = fake_game_lobby_contents.clone();
        use crate::topics;
        thread::spawn(move || {
            let components = Components {
                game_lobby_repo: Box::new(FakeGameLobbyRepo { contents: fgl }),
                xadd: Box::new(FakeXAdd(xadd_in)),
            };
            let lobby = LobbyStreams::new(components);

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

            let mut handlers = HashMap::new();
            for (stream, handler) in stream_handlers {
                handlers.insert(stream.to_string(), StreamHandler::new(stream, handler));
            }

            let guts = SortedStreamGuts {
                handlers,
                xack: Box::new(FakeXAck(xack_in)),
                xreadgroup_sorted: Box::new(FakeXRead {
                    sorted_data: sfs.clone(),
                }),
            };
            let mut streams = FakeSortedStreams { guts };

            lobby.process(&mut streams)
        });

        // emit some events in a time-ordered fashion
        let mut fake_time_ms = 100;
        let incr_ms = 100;

        let session_b = SessionId::new();
        let session_w = SessionId::new();
        let client_b = ClientId::new();
        let client_w = ClientId::new();
        let xid0 = quick_xid(fake_time_ms);
        sorted_fake_stream.lock().expect("lock").push((
            xid0,
            StreamMessage(
                topics::FIND_PUBLIC_GAME.to_string(),
                to_message(FindPublicGame {
                    client_id: client_w.clone(),
                    session_id: session_w.clone(),
                }),
            ),
        ));

        thread::sleep(timeout);

        // The game lobby repo should now contain one game
        assert_eq!(
            fake_game_lobby_contents
                .clone()
                .lock()
                .expect("gl")
                .games
                .iter()
                .collect::<Vec<_>>()
                .len(),
            1
        );

        // There should be an XADD triggered for a wait-for-opponent
        // message
        select! {
            recv(xadd_out) -> msg => match msg {
                Ok(StreamOutput::WFO(_)) => assert!(true),
                _ => panic!("wrong output")
            },
            default(timeout) => panic!("WAIT timeout")
        }

        fake_time_ms += incr_ms;
        sorted_fake_stream.lock().expect("lock").push((
            quick_xid(fake_time_ms),
            StreamMessage(
                topics::FIND_PUBLIC_GAME.to_string(),
                to_message(FindPublicGame {
                    client_id: client_b,
                    session_id: session_b,
                }),
            ),
        ));

        // There should now be GameReady in stream
        select! {
            recv(xadd_out) -> msg => match msg {
                Ok(StreamOutput::GR(_)) => assert!(true),
                _ => assert!(false)
            },
            default(timeout) => panic!("GR time out")
        }
    }

    fn quick_xid(ms: u64) -> XId {
        XId {
            millis_time: ms,
            seq_no: 0,
        }
    }

    use serde;
    use std::collections::HashMap;
    fn to_message<T: serde::ser::Serialize>(v: T) -> redis_streams::Message {
        let bin = bincode::serialize(&v).expect("ser");
        let mut out = HashMap::new();
        out.insert("data".to_string(), redis::Value::Data(bin));
        out
    }
}
