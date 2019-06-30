use crossbeam_channel::Sender;
use futures::stream::Stream;
use futures::*;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer};
use rdkafka::message::{Headers, Message};
use rdkafka::producer::{FutureProducer, FutureRecord};
use uuid::Uuid;

use crate::model::{BugoutMessage, Commands, Coord, Player};

const BROKERS: &str = "kafka:9092";
const APP_NAME: &str = "gateway";
const MAKE_MOVE_CMD_TOPIC: &str = "bugout-make-move-cmd";
const MOVE_MADE_EV_TOPIC: &str = "bugout-move-made-ev";
const CONSUME_TOPICS: &[&str] = &[MAKE_MOVE_CMD_TOPIC, MOVE_MADE_EV_TOPIC];

pub fn start(router_in: crossbeam_channel::Sender<BugoutMessage>) {
    println!("kafka::start");
    producer_example();

    consume_and_forward(BROKERS, APP_NAME, CONSUME_TOPICS, router_in);
}

fn producer_example() {
    println!("In producer example");
    let producer = configure_producer(BROKERS);

    let example_req_id = Uuid::new_v4();
    let example_command = Commands::MakeMove {
        game_id: Uuid::new_v4(),
        req_id: example_req_id,
        coord: Some(Coord { x: 0, y: 0 }),
        player: Player::BLACK,
    };

    println!("Sending command to kafka with req_id {}", example_req_id);
    let send_future = producer
        .send(
            FutureRecord::to(MAKE_MOVE_CMD_TOPIC)
                .payload(&serde_json::to_string(&example_command).unwrap())
                .key(&example_req_id.to_string()),
            0,
        )
        .map(move |delivery_status| {
            println!("Delivery status for message received");
            delivery_status
        });

    println!(
        "Kafka send future completed. Result: {:?}",
        send_future.wait()
    );
}

fn configure_producer(brokers: &str) -> FutureProducer {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("produce.offset.report", "true")
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error")
}

/// Adapted from https://github.com/fede1024/rust-rdkafka/blob/master/examples/simple_consumer.rs
fn consume_and_forward(
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
