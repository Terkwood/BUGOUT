use redis::Client;
use std::sync::Arc;

pub trait XAck {
    fn ack_attach_bot(&self);
    fn ack_game_states_changelog(&self);
}

impl XAck for Arc<Client> {
    fn ack_attach_bot(&self) {
        todo!()
    }

    fn ack_game_states_changelog(&self) {
        todo!()
    }
}
