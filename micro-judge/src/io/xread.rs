use super::conn_pool::Pool;
use super::redis;
use super::topics::*;
use crate::model::*;
use crate::repo::entry_id::AllEntryIds;
use redis_streams::*;
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

const BLOCK_MSEC: u32 = 5000;
pub type XReadResult = Vec<HashMap<String, Vec<HashMap<String, HashMap<String, String>>>>>;

pub fn xread_sort(
    entry_ids: AllEntryIds,
    topics: &StreamTopics,
    pool: &Pool,
) -> Result<Vec<(XReadEntryId, StreamData)>, redis::RedisError> {
    let mut conn = pool.get().unwrap();
    let ser = redis::cmd("XREAD")
        .arg("BLOCK")
        .arg(&BLOCK_MSEC.to_string())
        .arg("STREAMS")
        .arg(&topics.make_move_cmd)
        .arg(&topics.game_states_changelog)
        .arg(entry_ids.make_moves_eid.to_string())
        .arg(entry_ids.game_states_eid.to_string())
        .query::<XReadResult>(&mut *conn)?;

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

#[derive(Clone)]
pub enum StreamData {
    MM(MakeMoveCommand),
    GS(GameId, GameState),
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
                            println!("Deser error around make move cmd")
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
                            println!("Deser error around make move cmd")
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
    xread_result: HashMap<String, String>,
) -> Result<MakeMoveCommand, uuid::Error> {
    let mx: Option<u16> = xread_result
        .get("coord_x")
        .and_then(|s| s.parse::<u16>().ok());
    let my: Option<u16> = xread_result
        .get("coord_y")
        .and_then(|s| s.parse::<u16>().ok());
    let coord = match (mx, my) {
        (Some(x), Some(y)) => Some(Coord { x, y }),
        _ => None,
    };
    Ok(MakeMoveCommand {
        game_id: GameId(Uuid::from_str(&xread_result["game_id"])?),
        req_id: ReqId(Uuid::from_str(&xread_result["req_id"])?),
        player: Player::from_str(&xread_result["player"]),
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
