use crossbeam_channel::Sender;
use futures::stream::Stream;
use rdkafka::client::ClientContext;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer, ConsumerContext, Rebalance};
use rdkafka::error::KafkaResult;
use rdkafka::message::{Headers, Message};
use rdkafka::util::get_rdkafka_version;

use crate::model::BugoutMessage;

/// Adapted from https://github.com/fede1024/rust-rdkafka/blob/master/examples/simple_consumer.rs
pub fn consume_and_forward(
    brokers: &str,
    group_id: &str,
    topics: &[&str],
    router_in: Sender<BugoutMessage>,
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
            Ok(Err(e)) => unimplemented!(),
            Ok(Ok(msg)) => unimplemented!(),
        }
    }
}
