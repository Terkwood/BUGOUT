use crossbeam_channel::Sender;
use futures::stream::Stream;
use futures::*;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer};
use rdkafka::message::{Headers, Message};
use rdkafka::producer::{FutureProducer, FutureRecord};
use uuid::Uuid;

use crate::model::{BugoutMessage, Coord, MakeMoveCommand, Player};

const BROKERS: &str = "kafka:9092";
const APP_NAME: &str = "gateway";
const MAKE_MOVE_CMD_TOPIC: &str = "bugout-make-move-cmd";
const MOVE_MADE_EV_TOPIC: &str = "bugout-move-made-ev";
const CONSUME_TOPICS: &[&str] = &[MAKE_MOVE_CMD_TOPIC, MOVE_MADE_EV_TOPIC];

pub fn start(router_in: crossbeam_channel::Sender<BugoutMessage>) {
    producer_example();

    consume_and_forward(BROKERS, APP_NAME, CONSUME_TOPICS, router_in);
}

const NUM_PREMADE_GAMES: usize = 10;
fn producer_example() {
    let producer = configure_producer(BROKERS);

    let mut premade_game_ids = vec![];
    for _ in 0..NUM_PREMADE_GAMES {
        premade_game_ids.push(Uuid::new_v4());
    }

    let mut initial_move_cmds = vec![];
    for i in 0..NUM_PREMADE_GAMES {
        let example_req_id = Uuid::new_v4();
        let example_command = MakeMoveCommand {
            game_id: premade_game_ids[i],
            req_id: example_req_id,
            coord: Some(Coord { x: 0, y: 0 }),
            player: Player::BLACK,
        };
        initial_move_cmds.push(example_command);
    }

    let send_futures = (0..NUM_PREMADE_GAMES)
        .map(|i| {
            println!(
                "Sending command to kafka with req_id {}",
                &initial_move_cmds[i].req_id
            );
            producer
                .send(
                    FutureRecord::to(MAKE_MOVE_CMD_TOPIC)
                        .payload(&serde_json::to_string(&initial_move_cmds[i]).unwrap())
                        .key(&initial_move_cmds[i].req_id.to_string()),
                    0,
                )
                .map(move |delivery_status| {
                    println!("Delivery status for message received");
                    delivery_status
                })
        })
        .collect::<Vec<_>>();
    ;

    for future in send_futures {
        println!(
            "Blocked until kafka send future completed. Result: {:?}",
            future.wait()
        );
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
