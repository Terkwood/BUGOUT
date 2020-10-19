use super::stream::GROUP_NAME;
use super::topics::*;
use core_model::*;
use log::error;
use move_model::*;
use redis::streams::StreamReadOptions;
use redis::{Client, Commands};
use redis_streams::*;
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

const BLOCK_MS: usize = 5000;

pub type XReadResult = Vec<HashMap<String, Vec<HashMap<String, HashMap<String, Vec<u8>>>>>>;

#[derive(Clone)]
pub enum StreamData {
    MM(MakeMove),
    GS(GameState),
}

pub fn read_sorted(
    topics: &StreamTopics,
    client: &Client,
) -> Result<Vec<(XReadEntryId, StreamData)>, redis::RedisError> {
    let mut conn = client.get_connection().expect("redis conn");
    let opts = StreamReadOptions::default()
        .block(BLOCK_MS)
        .group(GROUP_NAME, "singleton");
    let ser = conn.xread_options(
        &[&topics.make_move_cmd, &topics.game_states_changelog],
        &[">", ">"],
        opts,
    )?;

    let unsorted = deser(ser, &topics);
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

pub fn ack(
    key: &str,
    group: &str,
    ids: &[XReadEntryId],
    client: &Client,
) -> Result<(), redis::RedisError> {
    let mut conn = client.get_connection().expect("conn");
    let idstrs: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
    let _: usize = conn.xack(key, group, &idstrs)?;
    Ok(())
}

fn deser(xread_result: XReadResult, topics: &StreamTopics) -> HashMap<XReadEntryId, StreamData> {
    let mut stream_data = HashMap::new();
    let make_move_topic = &topics.make_move_cmd;
    let game_states_topic = &topics.game_states_changelog;
    for hash in xread_result.iter() {
        for (xread_topic, xread_move_data) in hash.iter() {
            if &xread_topic[..] == make_move_topic {
                for with_timestamps in xread_move_data {
                    for (k, v) in with_timestamps {
                        if let (Ok(seq_no), Ok(m)) = (
                            XReadEntryId::from_str(k),
                            deser_make_move_command(v.clone()),
                        ) {
                            stream_data.insert(seq_no, StreamData::MM(m));
                        } else {
                            error!("Deser error around make move cmd")
                        }
                    }
                }
            } else if &xread_topic[..] == game_states_topic {
                for with_timestamps in xread_move_data {
                    for (k, v) in with_timestamps {
                        if let (Ok(seq_no), Some(game_state)) = (
                            XReadEntryId::from_str(k),
                            v.get("data").and_then(|gs| GameState::from(gs).ok()),
                        ) {
                            stream_data.insert(seq_no, StreamData::GS(game_state));
                        } else {
                            error!("Deser error around make move cmd")
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

fn deser_make_move_command(
    xread_result: HashMap<String, Vec<u8>>,
) -> Result<MakeMove, uuid::Error> {
    let values_as_strings: HashMap<String, String> = xread_result
        .iter()
        .map(|(k, v)| (k.clone(), String::from_utf8(v.clone()).expect("bytes")))
        .collect();
    let mx: Option<u16> = values_as_strings
        .get("coord_x")
        .and_then(|s| s.parse::<u16>().ok());
    let my: Option<u16> = values_as_strings
        .get("coord_y")
        .and_then(|s| s.parse::<u16>().ok());
    let coord = match (mx, my) {
        (Some(x), Some(y)) => Some(Coord { x, y }),
        _ => None,
    };
    Ok(MakeMove {
        game_id: GameId(Uuid::from_str(&values_as_strings["game_id"])?),
        req_id: ReqId(Uuid::from_str(&values_as_strings["req_id"])?),
        player: Player::from_str(&values_as_strings["player"]),
        coord,
    })
}

#[cfg(test)]
mod tests {
    use super::XReadEntryId;
    #[test]
    fn test_sort() {
        let foo = XReadEntryId {
            millis_time: 2,
            seq_no: 0,
        };
        let bar = XReadEntryId {
            millis_time: 1,
            seq_no: 10,
        };
        let baz = XReadEntryId {
            millis_time: 1,
            seq_no: 1,
        };
        let mut entries = vec![foo, bar, baz];

        entries.sort();
        assert_eq!(entries, vec![baz, bar, foo])
    }
}
