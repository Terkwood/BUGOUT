use crossbeam_channel::Sender;
use futures::stream::Stream;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer};
use rdkafka::message::{Headers, Message};
use rdkafka::producer::{FutureProducer, FutureRecord};

use crate::model::BugoutMessage;

const BROKERS: &str = "kafka:9092";
const APP_NAME: &str = "gateway";
const CONSUME_TOPICS: &[&str] = &["bugout-make-move-cmd", "bugout-move-made-ev"];

pub fn start(router_in: crossbeam_channel::Sender<crate::model::BugoutMessage>) {
    consume_and_forward(BROKERS, APP_NAME, CONSUME_TOPICS, router_in);

    let _kafka_producer = configure_producer(BROKERS);
}

/// Adapted from https://github.com/fede1024/rust-rdkafka/blob/master/examples/simple_consumer.rs
pub fn consume_and_forward(
    brokers: &str,
    group_id: &str,
    topics: &[&str],
    _router_in: Sender<BugoutMessage>,
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
            Err(_) => unimplemented!(),
            Ok(Err(_e)) => unimplemented!(),
            Ok(Ok(msg)) => {
                let payload = match msg.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(_)) => unimplemented!(),
                };

                println!(
                    "key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                    msg.key(), payload, msg.topic(), msg.partition(),
                    msg.offset(), msg.timestamp());

                if let Some(headers) = msg.headers() {
                    for i in 0..headers.count() {
                        let header = headers.get(i).unwrap();
                        println!("  Header {:#?}: {:?}", header.0, header.1);
                    }
                }

                consumer.commit_message(&msg, CommitMode::Async).unwrap();
            }
        }
    }
}

pub fn configure_producer(brokers: &str) -> FutureProducer {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("produce.offset.report", "true")
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error")
}
