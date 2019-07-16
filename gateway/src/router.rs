use std::collections::HashMap;
use std::thread;

use crossbeam::{Receiver, Sender};
use crossbeam_channel::select;

use uuid::Uuid;

use crate::logging::{short_uuid, EMPTY_SHORT_UUID, MEGA_DEATH_STRING};
use crate::model::{ClientId, Events, GameId, OpenGameReplyEvent, Player, ReconnectedEvent, ReqId};

/// start the select! loop responsible for sending kafka messages to relevant websocket clients
/// it must respond to requests to let it add and drop listeners
pub fn start(router_commands_out: Receiver<RouterCommand>, kafka_events_out: Receiver<Events>) {
    thread::spawn(move || {
        let mut router = Router::new();
        loop {
            select! {
                recv(router_commands_out) -> command =>
                    match command {
                        Ok(RouterCommand::RequestOpenGame{client_id, events_in, req_id}) => {
                            let game_id = router.add_client(client_id, events_in.clone());
                            if let Err(err) = events_in.send(Events::OpenGameReply(OpenGameReplyEvent{game_id, reply_to:req_id, event_id: Uuid::new_v4()})) {
                                println!("😯 {} {} {:<8} could not send open game reply {}",
                                    short_uuid(client_id), EMPTY_SHORT_UUID, "ERROR", err)
                            }
                        },
                        Ok(RouterCommand::DeleteClient{client_id, game_id}) =>
                            router.delete_client(client_id, game_id),
                        Ok(RouterCommand::RegisterOpenGame{game_id}) =>
                            router.register_open_game(game_id),
                        Ok(RouterCommand::Reconnect{client_id, game_id, events_in, req_id }) => {
                            router.reconnect_client(client_id, game_id, events_in.clone());
                            if let Err(err) = events_in.send(Events::Reconnected(ReconnectedEvent{game_id, reply_to: req_id, event_id: Uuid::new_v4(), player_up: router.playerup(game_id)})) {
                                println!(
                                    "😦 {} {} {:<8} could not send reconnect reply {}",
                                    short_uuid(client_id),
                                    short_uuid(game_id),
                                    "ERROR", err)
                            }
                        },
                        Err(e) => panic!("Unable to receive command via router channel: {:?}", e),
                    },
                recv(kafka_events_out) -> event =>
                    match event {
                        Ok(Events::MoveMade(m)) => {
                            let u = m.clone();
                            router.set_playerup(u.game_id, u.player.other());
                            router.forward_event(Events::MoveMade(m))
                        }
                        Ok(e) =>
                            router.forward_event(e),
                        Err(e) =>
                            panic!("Unable to receive kafka event via router channel: {:?}", e),
                    }
            }
        }
    });
}

struct GameState {
    pub clients: Vec<ClientSender>,
    pub playerup: Player,
}

impl Default for GameState {
    fn default() -> GameState {
        GameState {
            clients: vec![],
            playerup: Player::BLACK,
        }
    }
}

/// Keeps track of clients interested in various games
/// Each client has an associated crossbeam Sender for BUGOUT events
struct Router {
    pub available_games: Vec<GameId>,
    pub game_states: HashMap<GameId, GameState>,
}

impl Router {
    pub fn new() -> Router {
        Router {
            available_games: vec![],
            game_states: HashMap::new(),
        }
    }

    pub fn forward_event(&self, ev: Events) {
        if let Some(GameState {
            clients,
            playerup: _,
        }) = self.game_states.get(&ev.game_id())
        {
            for c in clients {
                if let Err(err) = c.events_in.send(ev.clone()) {
                    println!(
                        "😑 {} {} {:<8} forwarding event {}",
                        short_uuid(c.client_id),
                        short_uuid(ev.game_id()),
                        "ERROR",
                        err
                    )
                }
            }
        }
    }

    pub fn playerup(&self, game_id: GameId) -> Player {
        self.game_states
            .get(&game_id)
            .map(|game_state| &game_state.playerup)
            .unwrap_or(&Player::BLACK)
            .clone()
    }

    // TODO This is never cleaned up
    pub fn set_playerup(&mut self, game_id: GameId, player: Player) {
        let c = player.clone();
        let default = GameState {
            clients: vec![],
            playerup: c,
        };
        let gs = self.game_states.get_mut(&game_id);
        match gs {
            None => {
                self.game_states.insert(game_id, default);
            }
            Some(gs) => {
                gs.playerup = player;
            }
        }
    }

    pub fn add_client(&mut self, client_id: ClientId, events_in: Sender<Events>) -> GameId {
        let newbie = ClientSender {
            client_id,
            events_in,
        };

        let game_id = self.pop_open_game_id();
        match self.game_states.get_mut(&game_id) {
            Some(gs) => gs.clients.push(newbie),
            None => {
                let with_newbie = GameState {
                    clients: vec![newbie],
                    playerup: Player::BLACK,
                };
                self.game_states.insert(game_id, with_newbie);
            }
        }

        game_id
    }

    pub fn reconnect_client(
        &mut self,
        client_id: ClientId,
        game_id: GameId,
        events_in: Sender<Events>,
    ) {
        let cs = ClientSender {
            client_id,
            events_in,
        };

        match self.game_states.get_mut(&game_id) {
            Some(gs) => gs.clients.push(cs),
            None => {
                self.game_states.insert(
                    game_id,
                    GameState {
                        clients: vec![cs],
                        playerup: Player::BLACK,
                    },
                );
            }
        }
    }

    pub fn delete_client(&mut self, client_id: ClientId, game_id: GameId) {
        if let Some(game_state) = self.game_states.get_mut(&game_id) {
            game_state.clients.retain(|c| c.client_id != client_id);
        }
    }

    /// This is a big fat hack...
    /// We want the game IDs to be data driven via kafka.
    /// But this is better than having the game IDs hardcoded.
    pub fn register_open_game(&mut self, game_id: GameId) {
        // Register duplicates as we'll plan to consume two at a time
        self.available_games.push(game_id);
        self.available_games.push(game_id);
    }

    fn pop_open_game_id(&mut self) -> GameId {
        let popped = self.available_games.pop();
        if let Some(open_game_id) = popped {
            open_game_id
        } else {
            panic!(
                "☠️ {} {} {:<8} Out of game IDs!",
                MEGA_DEATH_STRING, MEGA_DEATH_STRING, "UBERFAIL"
            )
        }
    }
}

#[derive(Debug, Clone)]
struct ClientSender {
    pub client_id: ClientId,
    pub events_in: Sender<Events>,
}

#[derive(Debug, Clone)]
pub enum RouterCommand {
    RequestOpenGame {
        client_id: ClientId,
        events_in: Sender<Events>,
        req_id: ReqId,
    },
    DeleteClient {
        client_id: ClientId,
        game_id: GameId,
    },
    RegisterOpenGame {
        game_id: GameId,
    },
    Reconnect {
        client_id: ClientId,
        game_id: GameId,
        events_in: Sender<Events>,
        req_id: ReqId,
    },
}
