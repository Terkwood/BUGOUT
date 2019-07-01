use std::collections::HashMap;
use uuid::Uuid;

use crate::model::{GameId,ClientId};

pub struct Router {
    pub gameClients: HashMap<GameId, ClientId>
}

impl Default for Router {
    fn default() -> Router {
        Router { gameClients: HashMap::new() }
    }
}
