use super::{topics, GROUP_NAME};
use log::warn;
use redis::{Client, Commands};
pub fn create_consumer_group(client: &Client) {
    let mut conn = client.get_connection().expect("group create conn");
    let to_create = vec![topics::GAME_STATES_CHANGELOG, topics::ATTACH_BOT_CMD];
    for topic in to_create {
        let created: Result<(), _> = conn.xgroup_create_mkstream(topic, GROUP_NAME, "$");
        if let Err(e) = created {
            warn!(
                "Ignoring error creating {} consumer group (it probably exists already) {:?}",
                topic, e
            );
        }
    }
}
