use crossbeam_channel::Sender;
use std::collections::HashMap;

use crate::model::GameId;

/// responsible for sending kafka messages to relevant websocket clients
pub struct Router {
    websocket_clients_by_game_id: HashMap<GameId, Vec<Sender<bool>>>,
}
