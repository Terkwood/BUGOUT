use super::GROUP_NAME;
use crate::topics;
use log::warn;
use redis::Commands;

pub fn create_consumer_group(client: &redis::Client) {
    let mut conn = client.get_connection().expect("group create conn");
    let to_create = vec![
        topics::FIND_PUBLIC_GAME,
        topics::CREATE_GAME,
        topics::JOIN_PRIVATE_GAME,
        topics::SESSION_DISCONNECTED,
    ];
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
