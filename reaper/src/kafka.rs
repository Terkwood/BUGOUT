use std::thread;

use crossbeam_channel::select;
use futures::StreamExt;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::Consumer;
use rdkafka::message::Message;

use rdkafka::producer::{FutureProducer, FutureRecord};

use crate::model::ShutdownCommand;
use crate::model::*;
use crate::topics::{CONSUME_TOPICS, SHUTDOWN_TOPIC};

pub const BROKERS: &str = "kafka:9092";
pub const APP_NAME: &str = "reaper";

pub async fn start(
    activity_in: crossbeam::Sender<KafkaActivity>,
    shutdown_out: crossbeam::Receiver<ShutdownCommand>,
) {
    thread::spawn(move || start_producer(shutdown_out));

    start_consumer(BROKERS, APP_NAME, CONSUME_TOPICS, activity_in).await;
}

fn start_producer(shutdown_out: crossbeam::Receiver<ShutdownCommand>) {
    let producer = configure_producer(BROKERS);

    loop {
        select! {
            recv(shutdown_out) -> command =>
                match command {
                    Ok(shutdown) => {
                        producer.send(FutureRecord::to(SHUTDOWN_TOPIC)
                            .payload(&serde_json::to_string(&shutdown).unwrap())
                            .key(&format!("{}",shutdown.as_millis())), 0); // fire & forget
                    },
                    Err(e) => println!("😡 Error receiving shutdown command on crossbeam channel: {}", e)
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

async fn start_consumer(
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

    let mut message_stream = consumer.start();
    loop {
        for message in message_stream.next().await {
            match message {
                Err(e) => panic!("Error waiting on kafka stream: {:?}", e),
                Ok(msg) => {
                    if let Err(e) = activity_in.send(KafkaActivity {
                        topic: msg.topic().to_string(),
                        timestamp: msg.timestamp().to_millis().unwrap_or(Default::default()),
                    }) {
                        println!("ERROR SENDING CROSSBEAM KAFKA ACTIVITY {}", e)
                    }
                }
            }
        }
    }
}
