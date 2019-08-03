use std::thread;

use crossbeam_channel::select;
use futures::stream::Stream;
use futures::*;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer};
use rdkafka::message::Message;
use rdkafka::producer::{FutureProducer, FutureRecord};
use uuid::Uuid;

use crate::json::GameStateJson;
use crate::model::*;
use crate::router::RouterCommand;

const BROKERS: &str = "kafka:9092";
const APP_NAME: &str = "gateway";
const GAME_STATES_TOPIC: &str = "bugout-game-states";
const MAKE_MOVE_CMD_TOPIC: &str = "bugout-make-move-cmd";
const MOVE_MADE_EV_TOPIC: &str = "bugout-move-made-ev";
const PROVIDE_GAME_HISTORY_CMD_TOPIC: &str = "bugout-provide-game-history-cmd";
const CONSUME_TOPICS: &[&str] = &[MOVE_MADE_EV_TOPIC];
const NUM_PREMADE_GAMES: usize = 64;

pub fn start(
    events_in: crossbeam::Sender<Events>,
    router_commands_in: crossbeam::Sender<RouterCommand>,
    commands_out: crossbeam::Receiver<ClientCommands>,
) {
    thread::spawn(move || start_producer(router_commands_in, commands_out));

    thread::spawn(move || start_consumer(BROKERS, APP_NAME, CONSUME_TOPICS, events_in));
}

fn start_producer(
    router_commands_in: crossbeam::Sender<RouterCommand>,
    kafka_out: crossbeam::Receiver<ClientCommands>,
) {
    let producer = configure_producer(BROKERS);

    create_premade_games(&producer, router_commands_in);

    loop {
        select! {
            recv(kafka_out) -> command =>
                match command {
                    Ok(ClientCommands::MakeMove(c)) => {
                        producer.send(FutureRecord::to(MAKE_MOVE_CMD_TOPIC)
                            .payload(&serde_json::to_string(&c).unwrap())
                            .key(&c.game_id.to_string()), 0);
                        // Fire and forget
                        ()
                    },
                    Ok(ClientCommands::ProvideHistory(c)) => {
                        producer.send(FutureRecord::to(unimplemented!())
                            .payload(&serde_json::to_string(&c).unwrap())
                            .key(&c.game_id.to_string()), 0);
                        // Fire and forget
                        ()
                    }
                    Ok(_) => (),
                    Err(e) => panic!("Unable to receive command via kafka channel: {:?}", e),
                }
        }
    }
}

fn create_premade_games(
    producer: &FutureProducer,
    router_commands_in: crossbeam::Sender<RouterCommand>,
) -> Vec<GameId> {
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
        future.wait().unwrap().unwrap();
    }

    // THIS IS A BIG FAT HACK
    // (but it's less bad than hardcoding these IDs in the browser app)
    for game_id in premade_game_ids.iter() {
        router_commands_in
            .send(RouterCommand::RegisterOpenGame { game_id: *game_id })
            .expect("couldnt send open game id")
    }

    println!("📝 {:<8} {}", "PREMADES", premade_game_ids.len());

    premade_game_ids
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
    events_in: crossbeam::Sender<Events>,
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
                let payload = match msg.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => panic!("Error viewing kafka payload {:?}", e),
                };

                consumer.commit_message(&msg, CommitMode::Async).unwrap();
                if let Ok(move_made) = serde_json::from_str(payload) {
                    events_in.send(Events::MoveMade(move_made)).unwrap()
                }
            }
        }
    }
}
