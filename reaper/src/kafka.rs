pub const BROKERS: &str = "kafka:9092";
pub const APP_NAME: &str = "reaper";

pub fn start(
) {
    thread::spawn(move || start_producer(commands_out));

    thread::spawn(move || start_consumer(BROKERS, APP_NAME, CONSUME_TOPICS, events_in));
}