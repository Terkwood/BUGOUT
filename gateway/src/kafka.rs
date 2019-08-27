use std::thread;

use crossbeam_channel::select;
use futures::stream::Stream;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer};
use rdkafka::message::Message;
use rdkafka::producer::{FutureProducer, FutureRecord};

use crate::model::*;
use crate::router::RouterCommand;
use crate::topics::*;

pub const BROKERS: &str = "kafka:9092";
pub const APP_NAME: &str = "gateway";

pub fn start(
    events_in: crossbeam::Sender<KafkaEvents>,
    commands_out: crossbeam::Receiver<KafkaCommands>,
) {
    thread::spawn(move || start_producer(commands_out));

    thread::spawn(move || start_consumer(BROKERS, APP_NAME, CONSUME_TOPICS, events_in));
}

fn start_producer(kafka_out: crossbeam::Receiver<KafkaCommands>) {
    let producer = configure_producer(BROKERS);

    loop {
        select! {
            recv(kafka_out) -> command =>
                match command {
                    Ok(KafkaCommands::MakeMove(c)) => {
                        producer.send(FutureRecord::to(MAKE_MOVE_TOPIC)
                            .payload(&serde_json::to_string(&c).unwrap())
                            .key(&c.game_id.to_string()), 0); // fire & forget
                    },
                    Ok(KafkaCommands::ProvideHistory(c)) => {
                        producer.send(FutureRecord::to(PROVIDE_HISTORY_TOPIC)
                            .payload(&serde_json::to_string(&c).unwrap())
                            .key(&c.game_id.to_string()), 0); // fire & forget
                    },
                    Ok(KafkaCommands::JoinPrivateGame(j)) => {
                        producer.send(FutureRecord::to(JOIN_PRIVATE_GAME_TOPIC)
                            .payload(&serde_json::to_string(&j).unwrap())
                            .key(&j.game_id.to_string()), 0); // fire & forget
                    },
                    Err(e) => panic!("Unable to receive command via kafka channel: {:?}", e),
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
    events_in: crossbeam::Sender<KafkaEvents>,
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

                let topic = msg.topic();

                // we match on the topic, explicitly, so that we can know
                // exactly what type of object to decode.  this lets us
                // avoid some horrid JSON annotations for our kafka-streams/jvm
                // level models
                match topic {
                    MOVE_MADE_TOPIC => {
                        let deserialized: Result<MoveMadeEvent, _> = serde_json::from_str(payload);
                        match deserialized {
                            Err(e) => println!("failed to deserialize move made {}", e),
                            Ok(m) => events_in.send(KafkaEvents::MoveMade(m)).unwrap(),
                        }
                    }
                    HISTORY_PROVIDED_TOPIC => {
                        let deserialized: Result<HistoryProvidedEvent, _> =
                            serde_json::from_str(payload);
                        match deserialized {
                            Err(e) => println!("failed to deserialize history prov {}", e),
                            Ok(h) => events_in.send(KafkaEvents::HistoryProvided(h)).unwrap(),
                        }
                    }
                    PRIVATE_GAME_REJECTED_TOPIC => {
                        let deserialized: Result<
                            PrivateGameRejectedKafkaEvent,
                            _,
                        > = serde_json::from_str(payload);
                        match deserialized {
                            Err(e) => println!("failed to deserialize priv game reject {}", e),
                            Ok(r) => events_in.send(KafkaEvents::PrivateGameRejected(r)).unwrap(),
                        }
                    }
                    GAME_READY_TOPIC => {
                        let deserialized: Result<GameReadyKafkaEvent, _> =
                            serde_json::from_str(payload);
                        match deserialized {
                            Err(e) => println!("failed to deserialize game ready {}", e),
                            Ok(g) => events_in.send(KafkaEvents::GameReady(g)).unwrap(),
                        }
                    }
                    other => println!("ERROR Couldn't match kafka events topic: {}", other),
                }
            }
        }
    }
}
