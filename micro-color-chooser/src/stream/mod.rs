mod init;
mod xadd;
mod xread;

pub use init::*;
pub use xadd::*;
pub use xread::*;

use crate::api::*;
use crate::components::*;
use crate::model::*;
use log::{error, warn};
use redis::Commands;
use redis_streams::XReadEntryId;

pub enum StreamInput {
    GR(GameReady),
    CCP(ChooseColorPref),
}

pub fn process(components: &Components) {
    todo!("ack id arrays");
    let mut gs_processed: Vec<XReadEntryId> = vec![];
    loop {
        todo!("match components.xread.xread_sorted()");
        /*Ok(_) => {
            for time_ordered_event in todo!("records") {
                todo!("match time_ordered_event")
            }
        }
        Err(_) => error!("xread"),*/

        todo!("acks");
        /*if !gs_processed.is_empty() {
            if let Err(_e) = components.xread.xack_game_states(&gs_processed) {
                error!("ack for game states failed")
            }
        }*/
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_no_conflict() {
        let game_id = GameId(Uuid::new_v4());
        let sessions = (SessionId(Uuid::new_v4()), SessionId(Uuid::new_v4()));
        let clients = (ClientId(Uuid::new_v4()), ClientId(Uuid::new_v4()));
        let game_ready_event = GameReady {
            game_id,
            sessions,
            event_id: EventId::new(),
        };
        todo!("write test");
    }
}
