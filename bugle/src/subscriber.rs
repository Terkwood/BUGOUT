use crate::ec2_startup::StartupManager;
use crate::WakeUpEvent;

use r2d2_redis::{r2d2, RedisConnectionManager};

const HOST_URL: &str = "redis://redis";
const TOPIC: &str = "wakeup-ev";

pub fn start() {
    let manager = RedisConnectionManager::new(HOST_URL).unwrap();
    let pool = r2d2::Pool::builder().build(manager).unwrap();

    let mut sub_conn = pool.get().unwrap();
    let mut sub = sub_conn.as_pubsub();

    sub.subscribe(TOPIC).unwrap();

    println!("Subscribed to redis channel: {}", TOPIC);

    let mut startup_manager = StartupManager::default();

    loop {
        if let Ok(msg) = sub.get_message() {
            let payload = msg.get_payload().unwrap_or("".to_string());
            let event: Result<WakeUpEvent, _> = serde_json::from_str(&payload);
            if let Ok(e) = event {
                println!("{:?}", e);
                startup_manager.wake_up()
            }
        }
    }
}
