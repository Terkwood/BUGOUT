use r2d2_redis::{r2d2, redis, RedisConnectionManager};
use redis::Commands;
use serde_derive::{Deserialize, Serialize};
use serde_json;

use crate::model::ClientId;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct WakeUpEvent {
    pub client_id: ClientId,
}

type RedisPool = r2d2_redis::r2d2::Pool<r2d2_redis::RedisConnectionManager>;

const REDIS_URL: &str = "redis://redis";
const TOPIC: &str = "wakeup-ev";

pub struct RedisWakeup {
    pool: RedisPool,
}

impl RedisWakeup {
    pub fn new() -> RedisWakeup {
        let manager = RedisConnectionManager::new(REDIS_URL).unwrap();

        RedisWakeup {
            pool: r2d2::Pool::builder().build(manager).unwrap(),
        }
    }
    pub fn publish(&self, client_id: ClientId) -> Result<(), r2d2_redis::redis::RedisError> {
        let mut conn = self.pool.get().unwrap();

        conn.publish(
            TOPIC,
            serde_json::to_string(&WakeUpEvent { client_id }).unwrap(),
        )
    }
}
