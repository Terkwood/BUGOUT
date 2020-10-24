use super::StreamInput;
use redis_streams::XReadEntryId;
pub struct Unacknowledged {
    attach_bot: Vec<XReadEntryId>,
    game_states: Vec<XReadEntryId>,
}

impl Unacknowledged {
    fn ack_all(&mut self) {
        todo!()
    }
    pub fn push(&mut self, xid: XReadEntryId, event: super::StreamInput) {
        todo!()
    }
}

const INIT_CAPACITY: usize = 25;
impl Default for Unacknowledged {
    fn default() -> Self {
        fn nv() -> Vec<XReadEntryId> {
            Vec::with_capacity(INIT_CAPACITY)
        }
        Self {
            attach_bot: nv(),
            game_states: nv(),
        }
    }
}
