use super::StreamTopics;
use crate::model::*;
use crate::redis;
use crate::repo::entry_id_repo::AllEntryIds;
use micro_model_moves::{GameId, GameState, MoveMade};
use redis_conn_pool::Pool;
use redis_streams::XReadEntryId;
// use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

const BLOCK_MSEC: u32 = 5000;

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub enum StringOrBin {
//     S(String),
//     B(Vec<u8>),
// // }
// impl redis::FromRedisValue for StringOrBin {
//     fn from_redis_value(rv: &redis::Value) -> std::result::Result<Self, redis::RedisError> {
//         match rv {
//         }
//     }
// }
// Here's your problem.  Right here.
// TODO TODO
pub type XReadResult = Vec<HashMap<String, Vec<HashMap<String, HashMap<String, String>>>>>;

pub fn xread_sorted(
    entry_ids: AllEntryIds,
    topics: &StreamTopics,
    pool: &Pool,
) -> Result<Vec<(XReadEntryId, StreamData)>, redis::RedisError> {
    let mut conn = pool.get().unwrap();
    let ser = redis::cmd("XREAD")
        .arg("BLOCK")
        .arg(&BLOCK_MSEC.to_string())
        .arg("STREAMS")
        .arg(&topics.game_ready_ev)
        .arg(&topics.move_accepted_ev)
        .arg(&topics.game_states_changelog)
        .arg(entry_ids.game_ready_eid.to_string())
        .arg(entry_ids.move_accepted_eid.to_string())
        .arg(entry_ids.game_states_eid.to_string())
        .query::<XReadResult>(&mut *conn)?;

    let unsorted = deser(ser, &topics);
    if !unsorted.is_empty() {
        println!("CHECKING ON UNSORTED STUFF!");
    }
    for x in unsorted.clone() {
        println!("\t{:#?}", x);
    }
    let mut sorted_keys: Vec<XReadEntryId> = unsorted.keys().map(|k| *k).collect();
    sorted_keys.sort();

    let mut answer = vec![];
    for sk in sorted_keys {
        if let Some(data) = unsorted.get(&sk) {
            answer.push((sk, data.clone()))
        }
    }
    Ok(answer)
}

#[derive(Clone, Debug)]
pub enum StreamData {
    MA(MoveMade),
    GS(GameId, GameState),
    GR(GameReadyEvent),
}

fn deser(xread_result: XReadResult, topics: &StreamTopics) -> HashMap<XReadEntryId, StreamData> {
    let mut stream_data = HashMap::new();
    let game_ready_topic = &topics.game_ready_ev;
    let move_accepted_topic = &topics.move_accepted_ev;
    let game_states_topic = &topics.game_states_changelog;
    for hash in xread_result.iter() {
        for (xread_topic, xread_move_data) in hash.iter() {
            if &xread_topic[..] == game_ready_topic {
                for with_timestamps in xread_move_data {
                    for (k, v) in with_timestamps {
                        if let (Ok(seq_no), Ok(gr)) =
                            (XReadEntryId::from_str(k), deser_game_ready_ev(v.clone()))
                        {
                            stream_data.insert(seq_no, StreamData::GR(gr));
                        } else {
                            println!("HELP");
                        }
                    }
                }
            } else if &xread_topic[..] == move_accepted_topic {
                if !xread_move_data.is_empty() {
                    println!(
                        "move accepted topic IF ... data empty? {}",
                        xread_move_data.is_empty()
                    );
                }
                for with_timestamps in xread_move_data {
                    for (k, v) in with_timestamps {
                        if let (Ok(seq_no), Some(move_accepted)) = (
                            XReadEntryId::from_str(k),
                            v.get("data").and_then(|mm| {
                                let move_made_deser: Option<MoveMade> =
                                    bincode::deserialize(mm.as_bytes()).ok();
                                move_made_deser
                            }),
                        ) {
                            stream_data.insert(seq_no, StreamData::MA(move_accepted));
                            println!("OK FINE THAT'S DONE");
                        } else {
                            println!("Xread: Deser err in move accepted ")
                        }
                    }
                }
            } else if &xread_topic[..] == game_states_topic {
                for with_timestamps in xread_move_data {
                    for (k, v) in with_timestamps {
                        if let (Ok(seq_no), Some(game_id), Some(game_state)) = (
                            XReadEntryId::from_str(k),
                            v.get("game_id").and_then(|g| Uuid::from_str(g).ok()),
                            v.get("data")
                                .and_then(|gs| GameState::from(gs.as_bytes()).ok()),
                        ) {
                            stream_data.insert(seq_no, StreamData::GS(GameId(game_id), game_state));
                        } else {
                            println!("Xread: Deser error around game states data")
                        }
                    }
                }
            } else {
                println!("Ignoring topic {}", &xread_topic[..])
            }
        }
    }

    stream_data
}

fn deser_game_ready_ev(
    xread_result: HashMap<String, String>,
) -> Result<GameReadyEvent, uuid::Error> {
    Ok(GameReadyEvent {
        game_id: GameId(Uuid::from_str(&xread_result["game_id"])?),
    })
}
