use std::collections::HashMap;
use std::thread;

use crossbeam::{Receiver, Sender};
use crossbeam_channel::select;

use crate::model::{ClientId, Events, GameId};

/// start the select! loop responsible for sending kafka messages to relevant websocket clients
/// it must respond to requests to let it add and drop listeners
pub fn start(commands_out: Receiver<RouterCommand>) {
    thread::spawn(move || {
        let event_listeners: HashMap<GameId, Vec<Sender<Events>>> = HashMap::new();

        loop {
            select! {
                recv(commands_out) -> command =>
                    match command {
                        Ok(RouterCommand::AddClient{client_id, game_id, events_in}) => unimplemented!(),
                        Ok(RouterCommand::DeleteClient(client_id)) => unimplemented!(),
                        Err(e) => panic!("Unable to receive command via router channel: {:?}", e),
                    }
            }
        }
    });
}

pub enum RouterCommand {
    AddClient {
        client_id: ClientId,
        game_id: GameId,
        events_in: Sender<Events>,
    },
    DeleteClient(ClientId),
}
