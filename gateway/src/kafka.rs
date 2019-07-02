use std::thread;

use crossbeam_channel::select;
use futures::stream::Stream;
use futures::*;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer};
use rdkafka::message::{Headers, Message};
use rdkafka::producer::{FutureProducer, FutureRecord};
use uuid::Uuid;

use crate::json::GameStateJson;
use crate::model::*;

const BROKERS: &str = "kafka:9092";
const APP_NAME: &str = "gateway";
const GAME_STATES_TOPIC: &str = "bugout-game-states";
const MAKE_MOVE_CMD_TOPIC: &str = "bugout-make-move-cmd";
const MOVE_MADE_EV_TOPIC: &str = "bugout-move-made-ev";
const CONSUME_TOPICS: &[&str] = &[MAKE_MOVE_CMD_TOPIC, MOVE_MADE_EV_TOPIC];
const NUM_PREMADE_GAMES: usize = 10;

pub fn start(commands_out: crossbeam::Receiver<Commands>) {
    thread::spawn(move || start_producer(commands_out));

    thread::spawn(move || start_consumer(BROKERS, APP_NAME, CONSUME_TOPICS));
}

fn start_producer(kafka_out: crossbeam::Receiver<Commands>) {
    let producer = configure_producer(BROKERS);

    create_premade_games(&producer);

    loop {
        select! {
            recv(kafka_out) -> _command => unimplemented!()
        }
    }
}

fn create_premade_games(producer: &FutureProducer) -> Vec<GameId> {
    let mut premade_game_ids = vec![];
    for _ in 0..NUM_PREMADE_GAMES {
        premade_game_ids.push(Uuid::new_v4());
    }

    // Log empty game states for several games with arbitrary game IDs
    let setup_game_futures = premade_game_ids
        .iter()
        .map(|game_id| {
            producer.send(
                FutureRecord::to(GAME_STATES_TOPIC)
                    .payload(&serde_json::to_string(&GameStateJson::default()).unwrap())
                    .key(&game_id.to_string()),
                0,
            )
        })
        .collect::<Vec<_>>();

    for future in setup_game_futures {
        println!(
            "Blocked until game state message sent. Result: {:?}",
            future.wait()
        );
    }

    println!("Available game IDs:");
    for game_id in premade_game_ids.iter() {
        println!("\t{}", game_id);
    }

    premade_game_ids
}

// TODO delete on merge
fn producer_example() {
    let producer = configure_producer(BROKERS);

    let premade_game_ids = create_premade_games(&producer);

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

    let send_command_futures = (0..NUM_PREMADE_GAMES)
        .map(|i| {
            println!(
                "Sending command to kafka with req_id {}",
                &initial_move_cmds[i].req_id
            );
            producer.send(
                FutureRecord::to(MAKE_MOVE_CMD_TOPIC)
                    .payload(&serde_json::to_string(&initial_move_cmds[i]).unwrap())
                    .key(&initial_move_cmds[i].req_id.to_string()),
                0,
            )
        })
        .collect::<Vec<_>>();

    for future in send_command_futures {
        println!(
            "Blocked until kafka send completed. Result: {:?}",
            future.wait()
        );
    }

    println!("Available game IDs:");
    for game_id in premade_game_ids.iter() {
        println!("\t{}", game_id);
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

fn start_consumer(brokers: &str, group_id: &str, topics: &[&str]) {
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
