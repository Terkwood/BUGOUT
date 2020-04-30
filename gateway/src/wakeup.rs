use crate::redis_io::RedisPool;
use log::info;
use r2d2_redis::redis;
use redis::Commands;
use serde_derive::{Deserialize, Serialize};

use crate::model::ClientId;
use crate::{short_uuid, EMPTY_SHORT_UUID};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct WakeUpEvent {
    pub client_id: ClientId,
}

const TOPIC: &str = "wakeup-ev";

pub struct RedisWakeup {
    pool: RedisPool,
}

impl RedisWakeup {
    pub fn new(pool: &RedisPool) -> RedisWakeup {
        RedisWakeup { pool: pool.clone() }
    }
    pub fn publish(&self, client_id: ClientId) -> Result<(), r2d2_redis::redis::RedisError> {
        let mut conn = self.pool.get().unwrap();

        let p: Result<(), r2d2_redis::redis::RedisError> = conn.publish(
            TOPIC,
            serde_json::to_string(&WakeUpEvent { client_id }).unwrap(),
        );

        p.map(|_| {
            info!(
                "☀️  {} {:<8} WAKEUP",
                short_uuid(client_id),
                EMPTY_SHORT_UUID
            )
        })
    }
}
