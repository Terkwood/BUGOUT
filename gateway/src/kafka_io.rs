use std::thread;

use crossbeam_channel::select;
use futures::StreamExt;
use log::{error, trace};
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer};
use rdkafka::message::Message;
use rdkafka::producer::{FutureProducer, FutureRecord};

use crate::backend_commands::*;
use crate::backend_events::*;
use crate::env::BROKERS;
use crate::idle_status::KafkaActivityObserved;
use crate::model::*;
use crate::topics::*;

pub const APP_NAME: &str = "gateway";

pub async fn start(
    events_in: crossbeam::Sender<BackendEvents>,
    shutdown_in: crossbeam::Sender<KafkaShutdownEvent>,
    activity_in: crossbeam::Sender<KafkaActivityObserved>,
    commands_out: crossbeam::Receiver<BackendCommands>,
) {
    thread::spawn(move || start_producer(commands_out));

    start_consumer(
        &BROKERS,
        APP_NAME,
        CONSUME_TOPICS,
        events_in,
        shutdown_in,
        activity_in,
    )
    .await
}

/// Pay attention to the topic keys in the loop 🔄 👀
fn start_producer(kafka_out: crossbeam::Receiver<BackendCommands>) {
    let producer = configure_producer(&BROKERS);

    loop {
        select! {
            recv(kafka_out) -> command =>
                match command {
                    Ok(BackendCommands::MakeMove(c)) =>
                        write(&producer,MAKE_MOVE_TOPIC,&serde_json::to_string(&c),&c.game_id.to_string())
                    ,
                    Ok(BackendCommands::ProvideHistory(c)) =>
                        write(&producer,PROVIDE_HISTORY_TOPIC,&serde_json::to_string(&c),&c.game_id.to_string())
                    ,
                    Ok(BackendCommands::JoinPrivateGame(j)) =>
                        write(&producer,JOIN_PRIVATE_GAME_TOPIC,&serde_json::to_string(&j),
                            &j.session_id.to_string())
                    ,
                    Ok(BackendCommands::FindPublicGame(f)) =>
                        write(&producer,FIND_PUBLIC_GAME_TOPIC,&serde_json::to_string(&f),&f.session_id.to_string())
                    ,
                    Ok(BackendCommands::CreateGame(c)) =>
                        write(&producer,CREATE_GAME_TOPIC,&serde_json::to_string(&c), &c.session_id.to_string())
                    ,
                    Ok(BackendCommands::ChooseColorPref(c)) =>
                        write(&producer, CHOOSE_COLOR_PREF_TOPIC, &serde_json::to_string(&c),&c.session_id.to_string())
                    ,
                    Ok(BackendCommands::ClientHeartbeat(h)) =>
                        write(&producer, CLIENT_HEARTBEAT_TOPIC, &serde_json::to_string(&h),&h.client_id.to_string())
                    ,
                    Ok(BackendCommands::SessionDisconnected(c)) =>
                        write(&producer, SESSION_DISCONNECTED_TOPIC, &serde_json::to_string(&c), &c.session_id.to_string())
                    ,
                    Ok(BackendCommands::QuitGame(q)) =>
                        write(&producer, QUIT_GAME_TOPIC, &serde_json::to_string(&q), &q.game_id.to_string())
                    ,
                    Ok(BackendCommands::ReqSync(rs)) =>
                        write(
                            &producer,
                            REQ_SYNC_TOPIC,
                            &serde_json::to_string(&rs),
                            &rs.session_id.to_string())
                    ,
                    Ok(BackendCommands::AttachBot(_)) =>
                        trace!("Ignoring attach bot")
                    ,
                    Err(e) => error!("💩 Unable to receive command via kafka channel: {:?}", e)
                }
        }
    }
}

/// write some data to kafka.  fire and forget
fn write(
    producer: &FutureProducer,
    topic: &str,
    payload: &std::result::Result<std::string::String, serde_json::Error>,
    key: &str,
) {
    match payload {
        Ok(p) => {
            producer.send(FutureRecord::to(topic).payload(p).key(key), 0); // fire & forget
        }
        Err(e) => error!("💩 Failed to serialize trivial kafka command: {}", e),
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
    events_in: crossbeam::Sender<BackendEvents>,
    shutdown_in: crossbeam::Sender<KafkaShutdownEvent>,
    activity_in: crossbeam::Sender<KafkaActivityObserved>,
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
                Err(e) => error!("💩 Error waiting on kafka stream: {:?}", e),
                Ok(msg) => {
                    let payload = match msg.payload_view::<str>() {
                        None => "",
                        Some(Ok(s)) => s,
                        Some(Err(e)) => {
                            error!("💩 Error viewing kafka payload {:?}\nReturning an empty string as consumer payload", e);
                            ""
                        }
                    };

                    consumer.commit_message(&msg, CommitMode::Async).unwrap();

                    let topic = msg.topic();

                    // we match on the topic, explicitly, so that we can know
                    // exactly what type of object to decode.  this lets us
                    // avoid some horrid JSON annotations for our kafka-streams/jvm
                    // level models
                    match topic {
                        MOVE_MADE_TOPIC => {
                            let deserialized: Result<MoveMadeEvent, _> =
                                serde_json::from_str(payload);
                            match deserialized {
                                Err(e) => error!("failed to deserialize move made {}", e),
                                Ok(m) => flail_on_fail(events_in.send(BackendEvents::MoveMade(m))),
                            }
                        }
                        HISTORY_PROVIDED_TOPIC => {
                            let deserialized: Result<HistoryProvidedEvent, _> =
                                serde_json::from_str(payload);
                            match deserialized {
                                Err(e) => error!("failed to deserialize history prov {}", e),
                                Ok(h) => {
                                    flail_on_fail(events_in.send(BackendEvents::HistoryProvided(h)))
                                }
                            }
                        }
                        PRIVATE_GAME_REJECTED_TOPIC => {
                            let deserialized: Result<PrivateGameRejectedBackendEvent, _> =
                                serde_json::from_str(payload);
                            match deserialized {
                                Err(e) => error!("failed to deserialize priv game reject {}", e),
                                Ok(r) => flail_on_fail(
                                    events_in.send(BackendEvents::PrivateGameRejected(r)),
                                ),
                            }
                        }
                        GAME_READY_TOPIC => {
                            let deserialized: Result<GameReadyBackendEvent, _> =
                                serde_json::from_str(payload);

                            match deserialized {
                                Err(e) => error!("failed to deserialize game ready {}", e),
                                Ok(g) => flail_on_fail(events_in.send(BackendEvents::GameReady(g))),
                            }
                        }
                        WAIT_FOR_OPPONENT_TOPIC => {
                            let deserialized: Result<WaitForOpponentBackendEvent, _> =
                                serde_json::from_str(payload);

                            match deserialized {
                                Err(e) => error!("failed to deserialize wait for opponent {}", e),
                                Ok(w) => {
                                    flail_on_fail(events_in.send(BackendEvents::WaitForOpponent(w)))
                                }
                            }
                        }
                        COLORS_CHOSEN_TOPIC => {
                            let deserialized: Result<ColorsChosenEvent, _> =
                                serde_json::from_str(payload);

                            match deserialized {
                                Err(e) => error!("failed to deserialize wait for opponent {}", e),
                                Ok(c) => {
                                    flail_on_fail(events_in.send(BackendEvents::ColorsChosen(c)))
                                }
                            }
                        }
                        SYNC_REPLY_TOPIC => {
                            let deserialized: Result<SyncReplyBackendEvent, _> =
                                serde_json::from_str(payload);

                            match deserialized {
                                Err(e) => error!("failed to deserialize sync reply {:?}", e),
                                Ok(sr) => {
                                    flail_on_fail(events_in.send(BackendEvents::SyncReply(sr)))
                                }
                            }
                        }
                        SHUTDOWN_TOPIC => {
                            let deserialized: Result<KafkaShutdownEvent, _> =
                                serde_json::from_str(payload);

                            match deserialized {
                                Err(e) => error!("failed to deserialize shutdown event {}", e),
                                Ok(_s) => {
                                    let send_result = shutdown_in
                                        .send(KafkaShutdownEvent(std::time::SystemTime::now()));

                                    if let Err(e) = send_result {
                                        error!(
                                            "HALP! Failed to send kafka event in crossbeam: {}",
                                            e
                                        )
                                    }
                                }
                            }
                        }
                        CLIENT_HEARTBEAT_TOPIC => (), // Listen for idle tracking
                        other => println!("ERROR Couldn't match kafka events topic: {}", other),
                    }

                    if topic != SHUTDOWN_TOPIC {
                        observe(activity_in.clone())
                    }
                }
            }
        }
    }
}

/// Because no one should .unwrap() a crossbeam send result
fn flail_on_fail(send_result: std::result::Result<(), crossbeam::SendError<BackendEvents>>) {
    if let Err(e) = send_result {
        println!("HALP! Failed to send kafka event in crossbeam: {}", e)
    }
}

fn observe(activity_in: crossbeam_channel::Sender<KafkaActivityObserved>) {
    if let Err(e) = activity_in.send(KafkaActivityObserved) {
        println!(
            "Error sending kafka activity observation in crossbeam \t{}",
            e
        )
    }
}
