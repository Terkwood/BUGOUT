use crossbeam::Sender;
use std::collections::HashMap;

use crate::model::{Events, GameId};

pub struct AddEventsListener {
    game_id: GameId,
    events_in: Sender<Events>,
}

/// responsible for sending kafka messages to relevant websocket clients
pub struct Router {
    event_listeners: HashMap<GameId, Vec<Sender<Events>>>,
}

impl Router {
    pub fn start() -> Router {
        Router {
            event_listeners: HashMap::new(),
        }
    }
}
