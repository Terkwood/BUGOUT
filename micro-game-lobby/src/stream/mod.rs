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
