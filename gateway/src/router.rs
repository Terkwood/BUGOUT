use std::collections::HashMap;
use std::thread;

use crossbeam::{Receiver, Sender};
use crossbeam_channel::select;

use crate::model::{ClientId, Events, GameId};

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
                        Err(e) => panic!("Unable to receive command via router channel: {:?}", e),
                    },
                recv(kafka_events_out) -> event =>
                    match event {
                        Ok(Events::MoveMade(m)) =>
                            if let Some(client_senders) = router.clients_by_game.get(&m.game_id) {
                                for cs in client_senders {
                                    cs.events_in.send(Events::MoveMade(m.clone())).expect("send error")
                                }
                            },
                        Ok(Events::MoveRejected(_m)) =>
                            unimplemented!(),
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
}

impl Router {
    pub fn new() -> Router {
        Router {
            clients_by_game: HashMap::new(),
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
}
