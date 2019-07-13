use std::collections::HashMap;
use std::thread;

use crossbeam::{Receiver, Sender};
use crossbeam_channel::select;

use uuid::Uuid;

use crate::model::{ClientId, Events, GameId, OpenGameReplyEvent, ReqId, RequestGameIdCommand};

/// start the select! loop responsible for sending kafka messages to relevant websocket clients
/// it must respond to requests to let it add and drop listeners
pub fn start(router_commands_out: Receiver<RouterCommand>, kafka_events_out: Receiver<Events>) {
    thread::spawn(move || {
        let mut router = Router::new();
        loop {
            select! {
                recv(router_commands_out) -> command =>
                    match command {
                        Ok(RouterCommand::AddClient{client_id, game_id, events_in}) =>
                            router.add_client(client_id , game_id , events_in),
                        Ok(RouterCommand::DeleteClient{client_id, game_id}) =>
                            router.delete_client(client_id, game_id),
                        Ok(RouterCommand::RegisterOpenGame{game_id}) =>
                            router.register_open_game(game_id),
                        Ok(RouterCommand::RequestGameId(client_id,
                            RequestGameIdCommand{ req_id})) =>
                            router.pop_open_game_reply(client_id, req_id),
                        Err(e) => panic!("Unable to receive command via router channel: {:?}", e),
                    },
                recv(kafka_events_out) -> event =>
                    match event {
                        Ok(e) =>
                            if let Some(client_senders) = router.clients_by_game.get(&e.game_id()) {
                                for cs in client_senders {
                                    cs.events_in.send(e.clone()).expect("send error")
                                }
                            },
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
}

impl Router {
    pub fn new() -> Router {
        Router {
            clients_by_game: HashMap::new(),
            available_games: vec![],
        }
    }

    pub fn add_client(&mut self, client_id: ClientId, game_id: GameId, events_in: Sender<Events>) {
        let newbie = ClientSender {
            client_id,
            events_in,
        };
        match self.clients_by_game.get_mut(&game_id) {
            Some(client_senders) => client_senders.push(newbie),
            None => {
                self.clients_by_game.insert(game_id, vec![newbie]);
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

        println!("📝 Registered open game {}", game_id)
    }

    pub fn pop_open_game_reply(&mut self, client_id: ClientId, reply_to: ReqId) {
        let cc = self.clients_by_game.clone();
        let clients = cc.iter().map(|(_, v)| v).flatten();
        let mut ccc = clients.clone();
        if let Some(client_sender) = ccc.find(|cs| cs.client_id == client_id) {
            let popped = self.available_games.pop();
            if let Some(open_game_id) = popped {
                client_sender
                    .events_in
                    .send(Events::OpenGameReply(OpenGameReplyEvent {
                        game_id: open_game_id,
                        reply_to,
                        event_id: Uuid::new_v4(),
                    }))
                    .expect("could not send game id reply from router")
            } else {
                panic!("Out of game IDs! ")
            }
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
    AddClient {
        client_id: ClientId,
        game_id: GameId,
        events_in: Sender<Events>,
    },
    DeleteClient {
        client_id: ClientId,
        game_id: GameId,
    },
    RegisterOpenGame {
        game_id: GameId,
    },
    RequestGameId(ClientId, RequestGameIdCommand),
}
