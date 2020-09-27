use super::{topics, GROUP_NAME};
use log::warn;
use redis::{Client, Commands};
pub fn create_consumer_group(client: &Client) {
    let mut conn = client.get_connection().expect("group create conn");
    let mm: Result<(), _> = conn.xgroup_create_mkstream(topics::GAME_READY, GROUP_NAME, "$");
    if let Err(e) = mm {
        warn!(
            "Ignoring error creating {} consumer group (it probably exists already) {:?}",
            topics::GAME_READY,
            e
        );
    }
    let gs: Result<(), _> = conn.xgroup_create_mkstream(topics::CHOOSE_COLOR_PREF, GROUP_NAME, "$");
    if let Err(e) = gs {
        warn!(
            "Ignoring error creating {} consumer group (it probably exists already) {:?}",
            topics::CHOOSE_COLOR_PREF,
            e
        );
    }
}
