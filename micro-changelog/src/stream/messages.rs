use super::StreamTopics;
use log::{error, warn};
use move_model::{GameState, MoveMade};
use redis::streams::StreamReadOptions;
use redis::Commands;
use redis_streams::XId;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub enum StreamData {
    MA(MoveMade),
    GS(GameState),
}

pub type XReadResult = Vec<HashMap<String, Vec<HashMap<String, (String, Option<Vec<u8>>)>>>>;

const BLOCK_MS: usize = 5000;

pub fn read_sorted(
    topics: &StreamTopics,
    client: &redis::Client,
) -> Result<Vec<(XId, StreamData)>, redis::RedisError> {
    let mut conn = client.get_connection().expect("conn");
    let opts = StreamReadOptions::default()
        .block(BLOCK_MS)
        .group("micro-changelog", "singleton");
    let ser = conn.xread_options(
        &[&topics.move_accepted_ev, &topics.game_states_changelog],
        &[">", ">"],
        opts,
    )?;

    let unsorted = deser(ser, &topics);
    let mut sorted_keys: Vec<XId> = unsorted.keys().map(|k| *k).collect();
    sorted_keys.sort();

    let mut answer = vec![];
    for sk in sorted_keys {
        if let Some(data) = unsorted.get(&sk) {
            answer.push((sk, data.clone()))
        }
    }
    Ok(answer)
}

pub fn ack(
    key: &str,
    group: &str,
    ids: &[XId],
    client: &redis::Client,
) -> Result<(), redis::RedisError> {
    let mut conn = client.get_connection().expect("conn");
    let idstrs: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
    let _: usize = conn.xack(key, group, &idstrs)?;
    Ok(())
}

fn deser(xread_result: XReadResult, topics: &StreamTopics) -> HashMap<XId, StreamData> {
    let mut stream_data = HashMap::new();
    let move_accepted_topic = &topics.move_accepted_ev;
    let game_states_topic = &topics.game_states_changelog;
    for hash in xread_result.iter() {
        for (xread_topic, xread_move_data) in hash.iter() {
            if &xread_topic[..] == move_accepted_topic {
                for with_timestamps in xread_move_data {
                    for (k, v) in with_timestamps {
                        if let (Ok(seq_no), Some(move_accepted)) = (
                            XId::from_str(k),
                            v.1.clone().and_then(|mm| {
                                let move_made_deser: Option<MoveMade> =
                                    bincode::deserialize(&mm).ok();
                                move_made_deser
                            }),
                        ) {
                            stream_data.insert(seq_no, StreamData::MA(move_accepted));
                        } else {
                            error!("Xread: Deser err in move accepted ")
                        }
                    }
                }
            } else if &xread_topic[..] == game_states_topic {
                for with_timestamps in xread_move_data {
                    for (k, v) in with_timestamps {
                        if let (Ok(seq_no), Some(game_state)) = (
                            XId::from_str(k),
                            v.1.clone().and_then(|bytes| GameState::from(&bytes).ok()),
                        ) {
                            stream_data.insert(seq_no, StreamData::GS(game_state));
                        } else {
                            error!("Xread: Deser error around game states data")
                        }
                    }
                }
            } else {
                warn!("Ignoring topic {}", &xread_topic[..])
            }
        }
    }

    stream_data
}
