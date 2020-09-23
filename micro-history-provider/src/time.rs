use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_millis() -> u128 {
    let start = SystemTime::now();
    start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
}
