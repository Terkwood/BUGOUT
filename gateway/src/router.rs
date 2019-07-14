use std::collections::HashMap;
use std::thread;

use crossbeam::{Receiver, Sender};
use crossbeam_channel::select;

use uuid::Uuid;

use crate::logging::short_uuid;
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
                            events_in.send(Events::OpenGameReply(OpenGameReplyEvent{game_id, reply_to:req_id, event_id: Uuid::new_v4()})).expect("could not send open game reply")
                        },
                        Ok(RouterCommand::DeleteClient{client_id, game_id}) =>
                            router.delete_client(client_id, game_id),
                        Ok(RouterCommand::RegisterOpenGame{game_id}) =>
                            router.register_open_game(game_id),
                        Ok(RouterCommand::Reconnect{client_id, game_id, events_in, req_id }) => {
                            router.reconnect_client(client_id, game_id, events_in.clone());
                            events_in.send(Events::Reconnected(ReconnectedEvent{game_id, reply_to: req_id, event_id: Uuid::new_v4(), player_up: router.playerup(game_id)})).expect("could not send reconnect reply")
                        },
                        Err(e) => panic!("Unable to receive command via router channel: {:?}", e),
                    },
                recv(kafka_events_out) -> event =>
                    match event {
                        Ok(Events::MoveMade(m)) => {
                            let u = m.clone();
                            router.set_playerup(u.game_id, u.player);
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

/// Keeps track of clients interested in various games
/// Each client has an associated crossbeam Sender for BUGOUT events
struct Router {
    pub clients_by_game: HashMap<GameId, Vec<ClientSender>>,
    pub available_games: Vec<GameId>,
    pub playerup_by_game: HashMap<GameId, Player>,
}

impl Router {
    pub fn new() -> Router {
        Router {
            clients_by_game: HashMap::new(),
            available_games: vec![],
            playerup_by_game: HashMap::new(), // TODO This is never cleaned up
        }
    }

    pub fn forward_event(&self, e: Events) {
        if let Some(client_senders) = self.clients_by_game.get(&e.game_id()) {
            for cs in client_senders {
                cs.events_in.send(e.clone()).expect("send error")
            }
        }
    }

    pub fn playerup(&self, game_id: GameId) -> Player {
        self.playerup_by_game
            .get(&game_id)
            .unwrap_or(&Player::BLACK)
            .clone()
    }

    // TODO This is never cleaned up
    pub fn set_playerup(&mut self, game_id: GameId, player: Player) {
        self.playerup_by_game.entry(game_id).or_insert(player);
    }

    pub fn add_client(&mut self, client_id: ClientId, events_in: Sender<Events>) -> GameId {
        let newbie = ClientSender {
            client_id,
            events_in,
        };

        let game_id = self.pop_open_game_id();
        match self.clients_by_game.get_mut(&game_id) {
            Some(client_senders) => client_senders.push(newbie),
            None => {
                self.clients_by_game.insert(game_id, vec![newbie]);
                self.playerup_by_game.insert(game_id, Player::BLACK);
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

        match self.clients_by_game.get_mut(&game_id) {
            Some(client_senders) => client_senders.push(cs),
            None => {
                self.clients_by_game.insert(game_id, vec![cs]);
                self.playerup_by_game.insert(game_id, Player::BLACK);
            }
        }
    }

    pub fn delete_client(&mut self, client_id: ClientId, game_id: GameId) {
        if let Some(client_senders) = self.clients_by_game.get(&game_id) {
            let mut without: Vec<ClientSender> = vec![];
            for cs in client_senders {
                if cs.client_id != client_id {
                    without.push(cs.clone());
                }
            }
            *self.clients_by_game.get_mut(&game_id).unwrap() = without;
        }
    }

    /// This is a big fat hack...
    /// We want the game IDs to be data driven via kafka.
    /// But this is better than having the game IDs hardcoded.
    pub fn register_open_game(&mut self, game_id: GameId) {
        // Register duplicates as we'll plan to consume two at a time
        self.available_games.push(game_id);
        self.available_games.push(game_id);

        println!("📝 GAME {}", short_uuid(game_id))
    }

    fn pop_open_game_id(&mut self) -> GameId {
        let popped = self.available_games.pop();
        if let Some(open_game_id) = popped {
            open_game_id
        } else {
            panic!("⚰️ Out of game IDs! ⚰️")
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
