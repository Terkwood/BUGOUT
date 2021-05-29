mod init;
mod xadd;

pub use init::*;
use redis_streams::SortedStreams;
pub use xadd::*;

use crate::components::Components;
use crate::game_lobby::GameLobbyOps;
use crate::PUBLIC_GAME_BOARD_SIZE;
use core_model::*;
use lobby_model::api::*;
use lobby_model::*;
use log::{error, info, trace};
use move_model::GameState;
use redis_streams::{Message, XId};

pub const GROUP_NAME: &str = "micro-game-lobby";

#[derive(Clone, Debug)]
pub enum StreamInput {
    FPG(FindPublicGame),
    CG(CreateGame),
    JPG(JoinPrivateGame),
    SD(SessionDisconnected),
}

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

use redis_streams::{anyhow, RedisSortedStreams};
use std::borrow::BorrowMut;
use std::rc::Rc;
impl LobbyStreams {
    pub fn new(reg: Components) -> Self {
        Self { reg }
    }

    pub fn process(&mut self, streams: &mut dyn SortedStreams) {
        loop {
            if let Err(e) = streams.consume() {
                error!("Stream err {:?}", e)
            }
        }
    }

    pub fn consume_fpg(&mut self, msg: &Message) {
        todo!("deser");
        let fpg: FindPublicGame = todo!();
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
    }

    pub fn consume_cg(&mut self, msg: &Message) {
        todo!("deser");
        let mut cg: CreateGame = todo!();
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
    }
}
/// Consumes the command to join a private game.
/// In the event that the game is invalid,
/// we will simply log a warning.
/// Consider implementing logic related to handling
/// private game rejection: https://github.com/Terkwood/BUGOUT/issues/304
fn consume_jpg(jpg: &JoinPrivateGame, reg: &Components) {
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
}

fn consume_sd(sd: &SessionDisconnected, reg: &Components) {
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
    use redis_streams::XId;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use std::thread;

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

    struct FakeXAdd(Sender<StreamOutput>);
    impl XAdd for FakeXAdd {
        fn xadd(&self, data: StreamOutput) -> Result<(), XAddErr> {
            if let Err(_) = self.0.send(data) {}
            Ok(())
        }
    }

    struct FakeSortedStreams;
    impl redis_streams::SortedStreams for FakeSortedStreams {
        fn consume(&mut self) -> redis_streams::anyhow::Result<()> {
            todo!("write some sort of test, i guess")
        }
    }

    #[test]
    fn test_process() {
        let (xadd_in, xadd_out) = unbounded();

        let sorted_fake_stream = Arc::new(Mutex::new(vec![]));

        let timeout = Duration::from_millis(160);

        // set up a loop to process game lobby requests
        let fake_game_lobby_contents = Arc::new(Mutex::new(GameLobby::default()));

        let fgl = fake_game_lobby_contents.clone();

        thread::spawn(move || {
            let components = Components {
                game_lobby_repo: Box::new(FakeGameLobbyRepo { contents: fgl }),
                xadd: Box::new(FakeXAdd(xadd_in)),
            };
            let mut fake_sorted_streams = FakeSortedStreams;
            let mut lobby_streams = LobbyStreams::new(components);
            lobby_streams.process(&mut fake_sorted_streams);
        });

        // emit some events

        let mut fake_time_ms = 100;
        let incr_ms = 100;

        let session_b = SessionId::new();
        let session_w = SessionId::new();
        let client_b = ClientId::new();
        let client_w = ClientId::new();
        let xid0 = quick_eid(fake_time_ms);
        sorted_fake_stream.lock().expect("lock").push((
            xid0,
            StreamInput::FPG(FindPublicGame {
                client_id: client_w.clone(),
                session_id: session_w.clone(),
            }),
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
            quick_eid(fake_time_ms),
            StreamInput::FPG(FindPublicGame {
                client_id: client_b,
                session_id: session_b,
            }),
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

    fn quick_eid(ms: u64) -> XId {
        XId {
            millis_time: ms,
            seq_no: 0,
        }
    }
}
