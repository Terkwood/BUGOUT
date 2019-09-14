use std::thread;

use crossbeam_channel::select;
use futures::stream::Stream;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::Consumer;
use rdkafka::message::{Message, Timestamp};

use rdkafka::producer::{FutureProducer, FutureRecord};

use crate::topics::{CONSUME_TOPICS, SHUTDOWN_TOPIC};
use crate::ShutdownCommand;

pub const BROKERS: &str = "kafka:9092";
pub const APP_NAME: &str = "reaper";

pub struct KafkaActivity {
    pub topic: String,
    pub timestamp: i64,
}

pub fn start(
    activity_in: crossbeam::Sender<KafkaActivity>,
    shutdown_out: crossbeam::Receiver<ShutdownCommand>,
) {
    thread::spawn(move || start_producer(shutdown_out));

    thread::spawn(move || start_consumer(BROKERS, APP_NAME, CONSUME_TOPICS, activity_in));
}

/// Pay attention to the topic keys in the loop 🔄 👀
fn start_producer(shutdown_out: crossbeam::Receiver<ShutdownCommand>) {
    let producer = configure_producer(BROKERS);

    loop {
        select! {
            recv(shutdown_out) -> command =>
                match command {
                    Ok(ShutdownCommand(epoch_millis)) =>   {
                        producer.send(FutureRecord::to(SHUTDOWN_TOPIC)
                            .payload(&serde_json::to_string(&ShutdownCommand(epoch_millis)).unwrap())
                            .key(&format!("{}",epoch_millis)), 0); // fire & forget
                    },
                    _ => unimplemented!()
                }
        }
    }
}

fn configure_producer(brokers: &str) -> FutureProducer {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("produce.offset.report", "true")
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error")
}

fn start_consumer(
    brokers: &str,
    group_id: &str,
    topics: &[&str],
    activity_in: crossbeam::Sender<KafkaActivity>,
) {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create()
        .expect("Consumer creation failed");

    consumer
        .subscribe(topics)
        .expect("Can't subscribe to topics");

    let message_stream = consumer.start();
    for message in message_stream.wait() {
        match message {
            Err(e) => panic!("Error waiting on kafka stream: {:?}", e),
            Ok(Err(e)) => panic!("Nested error (!) waiting on kafka stream: {:?}", e),
            Ok(Ok(msg)) => {
                if let Err(_) = activity_in.send(KafkaActivity {
                    topic: msg.topic().to_string(),
                    timestamp: msg.timestamp().to_millis().unwrap_or(Default::default()),
                }) {
                    panic!("ERROR SENDING KAFKA ACTIVITY")
                }
            }
        }
    }
}
