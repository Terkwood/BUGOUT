use super::redis_keys::RedisKeyNamespace;
use super::topics::*;
use super::xread::*;
use super::WriteErr;
use crate::game::*;
use crate::model::*;
use crate::repo::game_states::GameStatesRepo;
use redis::Client;
use std::rc::Rc;

use log::{error, info, warn};

/// Spins too much.  See https://github.com/Terkwood/BUGOUT/issues/217
pub fn process(opts: ProcessOpts) {
    create_consumer_group(&opts.topics);
    loop {
        if let Ok(xread_result) = xread_sort(&opts.topics, &opts.client) {
            for time_ordered_event in xread_result {
                match time_ordered_event {
                    (_entry_id, StreamData::MM(mm)) => {
                        let fetched_gs = opts.game_states_repo.fetch(&mm.game_id);
                        match fetched_gs {
                            Ok(Some(game_state)) => match judge(&mm, &game_state) {
                                Judgement::Accepted(move_made) => {
                                    if let Err(e) = xadd_move_accepted(
                                        &move_made,
                                        &opts.client,
                                        &opts.topics.move_accepted_ev,
                                    ) {
                                        error!("Error XADD to move_accepted {:?}", e)
                                    }
                                }
                                Judgement::Rejected => error!("MOVE REJECTED: {:#?}", mm),
                            },
                            Ok(None) => warn!("No game state for game {}", mm.game_id.0),
                            Err(e) => error!("Deser error ({:?})!", e),
                        }

                        todo!("XACK")
                    }
                    (_entry_id, StreamData::GS(game_id, game_state)) => {
                        if let Err(e) = &opts.game_states_repo.write(&game_id, &game_state) {
                            error!("error writing game state {:?}  -- advancing eid pointer", e)
                        }

                        todo!("XACK");

                        info!("Tracking {:?} {:?}", game_id, game_state);
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct ProcessOpts {
    pub topics: StreamTopics,
    pub game_states_repo: GameStatesRepo,
    pub client: Rc<Client>,
}
impl Default for ProcessOpts {
    fn default() -> Self {
        let namespace = RedisKeyNamespace::default();
        let client = Rc::new(redis::Client::open("redis://redis").expect("client"));
        ProcessOpts {
            topics: StreamTopics::default(),
            game_states_repo: GameStatesRepo {
                namespace,
                client: client.clone(),
            },
            client,
        }
    }
}

fn xadd_move_accepted(
    move_made: &MoveMade,
    client: &Client,
    stream_name: &str,
) -> Result<String, WriteErr> {
    let mut conn = client.get_connection().unwrap();
    Ok(redis::cmd("XADD")
        .arg(stream_name)
        .arg("MAXLEN")
        .arg("~")
        .arg("1000")
        .arg("*")
        .arg("game_id")
        .arg(move_made.game_id.0.to_string())
        .arg("data")
        .arg(move_made.serialize()?)
        .query::<String>(&mut conn)?)
}
