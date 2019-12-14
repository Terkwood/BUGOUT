use redis;

use std::thread;

use r2d2_redis::redis::Commands;
use r2d2_redis::{r2d2, RedisConnectionManager};

use crate::WakeUp;

pub fn start() {
    let manager = RedisConnectionManager::new("redis://redis").unwrap();
    let pool = r2d2::Pool::builder().build(manager).unwrap();

    let mut sub_conn = pool.get().unwrap();
    let mut sub = sub_conn.as_pubsub();
    let topic = "wakeup-ev";
    sub.subscribe(topic).unwrap();

    println!("Subscribed to redis channel: {}", topic);

    loop {
        if let Ok(msg) = sub.get_message() {
            let payload = msg.get_payload().unwrap_or("".to_string());
            println!("{}", payload)
            /*let revent: Result<String, _> = serde_json::from_str(&payload);
            if let Ok(e) = revent {
                tx.send(e).unwrap()
            }*/
        }
    }
}
