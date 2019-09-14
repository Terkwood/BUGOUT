use std::thread;

pub const BROKERS: &str = "kafka:9092";
pub const APP_NAME: &str = "reaper";

struct KafkaActivity {
    pub topic: String,
    pub timestamp: u64,
}

struct ShutdownCommand;

pub fn start(shutdown_out: crossbeam::Receiver<ShutdownCommand>) {
    thread::spawn(move || start_producer(commands_out));

    thread::spawn(move || start_consumer(BROKERS, APP_NAME, CONSUME_TOPICS, events_in));
}

/// Pay attention to the topic keys in the loop 🔄 👀
fn start_producer(kafka_out: crossbeam::Receiver<KafkaMessage>) {
    let producer = configure_producer(BROKERS);

    loop {
        select! {
            recv(kafka_out) -> command =>
                match command {
                    Ok(_) => unimplemented!(),
                    _ => unimplemented!()
                }
        }
    }
}