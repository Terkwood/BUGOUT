use redis;

use crate::WakeUp;

pub fn start() {
    let redis_client = unimplemented!();
    let mut sub_conn = redis_client.get_connection().unwrap();
    let mut sub = sub_conn.as_pubsub();
    let topic = unimplemented!();
    sub.subscribe(topic).unwrap();

    println!("Subscribed to redis channel: {}", topic);

    loop {
        if let Ok(msg) = sub.get_message() {
            let payload = msg.get_payload().unwrap_or("".to_string());
            /*let revent: Result<String, _> = serde_json::from_str(&payload);
            if let Ok(e) = revent {
                tx.send(e).unwrap()
            }*/
        }
    }
}
