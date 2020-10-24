use super::StreamInput;
use super::StreamOpts;
use log::error;
use redis_streams::XReadEntryId;
pub struct Unacknowledged {
    attach_bot: Vec<XReadEntryId>,
    game_states: Vec<XReadEntryId>,
}

impl Unacknowledged {
    pub fn ack_all(&mut self, opts: &StreamOpts) {
        if !self.attach_bot.is_empty() {
            if let Err(_e) = opts.xack.ack_attach_bot(&self.attach_bot) {
                error!("ack for ab failed")
            } else {
                self.attach_bot.clear();
            }
        }

        if !self.game_states.is_empty() {
            if let Err(_e) = opts.xack.ack_game_states_changelog(&self.game_states) {
                error!("ack for gs failed")
            } else {
                self.game_states.clear();
            }
        }
    }
    pub fn push(&mut self, xid: XReadEntryId, event: &StreamInput) {
        match event {
            StreamInput::GS(_) => self.game_states.push(xid),
            StreamInput::AB(_) => self.attach_bot.push(xid),
        }
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
