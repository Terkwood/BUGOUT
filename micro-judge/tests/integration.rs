/// CAUTION!  YOU MUST RUN THIS TEST FILE ON A SINGLE THREAD!
/// ```sh
/// cargo test -- --test-threads=1
/// ```
///
/// ...or...

/// ```sh
/// cargo watch -x "test -- --test-threads=1"
/// ```
extern crate micro_judge;
extern crate r2d2_redis;

use conn_pool::Pool;
use conn_pool::RedisHostUrl;
use micro_judge::io::xread::XReadEntryId;
use micro_judge::io::{conn_pool, redis_keys, stream, topics};
use micro_judge::model::*;
use micro_judge::repo::entry_id::{AllEntryIds, EntryIdRepo, EntryIdType};
use micro_judge::repo::game_states::GameStatesRepo;
use r2d2_redis::{r2d2, redis};
use redis::Commands;
use redis_keys::RedisKeyNamespace;
use std::collections::HashMap;
use std::panic;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::Duration;
use topics::StreamTopics;

const USAGE: &str = "CAUTION!  YOU MUST RUN tests/integration.rs ON A SINGLE THREAD!
Use one of the following:

    cargo test -- --test-threads=1

    cargo watch -x \"test -- --test-threads=1\"
";

const TEST_GAME_STATES_TOPIC: &str = "bugtest-game-states";
const TEST_MAKE_MOVE_CMD_TOPIC: &str = "bugtest-make-move-cmd";
const TEST_MOVE_ACCEPTED_EV_TOPIC: &str = "bugtest-move-accepted-ev";

static FIRST_TEST_COMPLETE: AtomicBool = AtomicBool::new(false);

fn redis_pool() -> r2d2::Pool<r2d2_redis::RedisConnectionManager> {
    conn_pool::create(RedisHostUrl("redis://localhost".to_string()))
}

fn test_namespace() -> RedisKeyNamespace {
    RedisKeyNamespace("BUGTEST".to_string())
}

fn test_opts() -> stream::ProcessOpts {
    stream::ProcessOpts {
        topics: StreamTopics {
            make_move_cmd: TEST_MAKE_MOVE_CMD_TOPIC.to_string(),
            game_states_changelog: TEST_GAME_STATES_TOPIC.to_string(),
            move_accepted_ev: TEST_MOVE_ACCEPTED_EV_TOPIC.to_string(),
        },
        entry_id_repo: EntryIdRepo {
            namespace: test_namespace(),
            pool: redis_pool(),
        },
        game_states_repo: GameStatesRepo {
            namespace: test_namespace(),
            pool: redis_pool(),
        },
    }
}

fn panic_cleanup(stream_names: Vec<String>, keys: Vec<String>, pool: Pool) {
    panic::set_hook(Box::new(move |e| {
        println!("{}", USAGE);
        println!("{:#?}", e);
        clean_streams(stream_names.clone(), &pool);
        clean_keys(keys.clone(), &pool);
        FIRST_TEST_COMPLETE.swap(true, std::sync::atomic::Ordering::Relaxed);
    }));
}

#[test]
fn test_emitted_game_states() {
    let pool = redis_pool();
    let streams_to_clean = vec![TEST_GAME_STATES_TOPIC.to_string()];

    let game_id = GameId(uuid::Uuid::new_v4());
    let data_key = redis_keys::game_states_key(&test_namespace(), &game_id);
    let keys_to_clean = vec![
        data_key.clone(),
        redis_keys::entry_ids_hash_key(&test_namespace()),
    ];
    panic_cleanup(
        streams_to_clean.clone(),
        keys_to_clean.clone(),
        pool.clone(),
    );

    let p2 = pool.clone();
    let stream_opts = test_opts();
    let eid_repo = stream_opts.clone().entry_id_repo;
    thread::spawn(move || stream::process(stream_opts, &p2));

    let mut conn = pool.get().unwrap();

    // Check precondition
    assert_eq!(eid_repo.fetch_all().unwrap().game_states_eid.millis_time, 0);

    let expected_game_state = GameState::default();
    redis::cmd("XADD")
        .arg(TEST_GAME_STATES_TOPIC)
        .arg("MAXLEN")
        .arg("~")
        .arg("1000")
        .arg("*")
        .arg("game_id")
        .arg(game_id.0.to_string())
        .arg("data")
        .arg(expected_game_state.serialize().unwrap())
        .query::<String>(&mut *conn)
        .unwrap();

    const WAIT_MS: u64 = 100;
    let mut local_game_state: Option<Vec<u8>> = None;
    const INIT_RETRIES: u8 = 100;
    let mut retries = INIT_RETRIES;
    while retries > 0 {
        let game_states_data: Result<Option<Vec<u8>>, _> = conn.get(&data_key);
        if let Ok(Some(h)) = game_states_data {
            local_game_state = Some(h.clone());
            break;
        } else {
            thread::sleep(Duration::from_millis(WAIT_MS));
            retries -= 1;
        }
    }
    assert!(local_game_state.is_some());
    let f = local_game_state.unwrap();
    assert!(f.len() > 0);
    let actual_game_state = GameState::from(&f);
    assert_eq!(expected_game_state, actual_game_state.unwrap());
    retries = INIT_RETRIES;
    let mut time_updated = false;
    while retries > 0 {
        if let Ok(entry_ids) = eid_repo.fetch_all() {
            time_updated = entry_ids.game_states_eid.millis_time > 0;
            if time_updated {
                break;
            }
        }
        thread::sleep(Duration::from_millis(WAIT_MS));
        retries -= 1;
    }
    assert!(time_updated);

    clean_streams(streams_to_clean, &pool);
    clean_keys(keys_to_clean, &pool);

    FIRST_TEST_COMPLETE.swap(true, std::sync::atomic::Ordering::Relaxed);
}

#[test]
fn test_moves_processed() {
    while !FIRST_TEST_COMPLETE.load(std::sync::atomic::Ordering::Relaxed) {
        thread::sleep(Duration::from_secs(1))
    }
    let pool = redis_pool();
    let streams_to_clean = vec![
        TEST_GAME_STATES_TOPIC.to_string(),
        TEST_MAKE_MOVE_CMD_TOPIC.to_string(),
        TEST_MOVE_ACCEPTED_EV_TOPIC.to_string(),
    ];

    let game_id = GameId(uuid::Uuid::new_v4());
    let game_states_data_key = redis_keys::game_states_key(&test_namespace(), &game_id);
    let keys_to_clean = vec![
        game_states_data_key.clone(),
        redis_keys::entry_ids_hash_key(&test_namespace()),
    ];
    panic_cleanup(
        streams_to_clean.clone(),
        keys_to_clean.clone(),
        pool.clone(),
    );

    let p2 = pool.clone();
    let stream_opts = test_opts();
    let eid_repo = stream_opts.clone().entry_id_repo;
    thread::spawn(move || stream::process(stream_opts, &p2));

    let mut conn = pool.get().unwrap();

    // Clear entry ids in redis
    eid_repo
        .update(EntryIdType::GameStateChangelog, XReadEntryId::default())
        .unwrap();
    eid_repo
        .update(EntryIdType::MakeMoveCommand, XReadEntryId::default())
        .unwrap();

    let initial_game_state = GameState::default();
    redis::cmd("XADD")
        .arg(TEST_GAME_STATES_TOPIC)
        .arg("MAXLEN")
        .arg("~")
        .arg("1000")
        .arg("*")
        .arg("game_id")
        .arg(game_id.0.to_string())
        .arg("data")
        .arg(initial_game_state.serialize().unwrap())
        .query::<String>(&mut *conn)
        .unwrap();

    let moves = vec![
        (Coord::of(10, 10), Player::BLACK),
        (Coord::of(9, 10), Player::WHITE),
        (Coord::of(0, 0), Player::BLACK),
        (Coord::of(11, 10), Player::WHITE),
        (Coord::of(0, 1), Player::BLACK),
        (Coord::of(10, 9), Player::WHITE),
        (Coord::of(0, 2), Player::BLACK),
        (Coord::of(10, 11), Player::WHITE), // captures 10,10
    ];

    let mut current_game_state = initial_game_state;
    for (move_coord, move_player) in moves.iter() {
        let move_req_id = uuid::Uuid::new_v4();
        // client makes a move
        redis::cmd("XADD")
            .arg(TEST_MAKE_MOVE_CMD_TOPIC)
            .arg("MAXLEN")
            .arg("~")
            .arg("1000")
            .arg("*")
            .arg("game_id")
            .arg(game_id.clone().0.to_string())
            .arg("player") //  req_id,
            .arg(move_player.to_string())
            .arg("coord_x")
            .arg(move_coord.x.to_string())
            .arg("coord_y")
            .arg(move_coord.y.to_string())
            .arg("req_id")
            .arg(move_req_id.to_string())
            .query::<String>(&mut *conn)
            .unwrap();

        thread::sleep(Duration::from_millis(10));
        let old_game_state = current_game_state.clone();
        current_game_state
            .board
            .pieces
            .insert(*move_coord, *move_player);
        current_game_state.player_up = match *move_player {
            Player::BLACK => Player::WHITE,
            _ => Player::BLACK,
        };
        current_game_state.moves.push(MoveMade {
            captured: micro_judge::game::captures_for(
                *move_player,
                *move_coord,
                &old_game_state.board,
            )
            .iter()
            .cloned()
            .collect(),
            coord: Some(*move_coord),
            player: *move_player,
            event_id: EventId(uuid::Uuid::nil()),
            game_id: game_id.clone(),
            reply_to: ReqId(move_req_id),
        });

        redis::cmd("XADD")
            .arg(TEST_GAME_STATES_TOPIC)
            .arg("MAXLEN")
            .arg("~")
            .arg("1000")
            .arg("*")
            .arg("game_id")
            .arg(game_id.clone().0.to_string())
            .arg("data")
            .arg(current_game_state.serialize().unwrap())
            .query::<String>(&mut *conn)
            .unwrap();
    }

    // Above moves should all be ACCEPTED by judge.  Check
    // that there is data on `bugtest-move-accepted-ev`
    // to confirm this.
    let mut conn = pool.get().unwrap();
    let move_accepted_data = redis::cmd("XREAD")
        .arg("BLOCK")
        .arg(5000)
        .arg("STREAMS")
        .arg(TEST_MOVE_ACCEPTED_EV_TOPIC)
        .arg("0-0")
        .query::<redis::Value>(&mut *conn)
        .unwrap();

    // It's in there somewhere...
    let mut out = vec![];
    match move_accepted_data {
        redis::Value::Bulk(bs) => match &bs[0] {
            redis::Value::Bulk(cs) => {
                for c in cs {
                    match c {
                        redis::Value::Bulk(ds) => match &ds[0] {
                            redis::Value::Bulk(es) => match &es[1] {
                                redis::Value::Bulk(fs) => match &fs[3] {
                                    redis::Value::Data(bin) => out.push(bin.clone()),
                                    _ => (),
                                },
                                _ => (),
                            },
                            _ => (),
                        },
                        _ => (),
                    }
                }
            }
            _ => (),
        },
        _ => (),
    };
    let accepted: MoveMade = bincode::deserialize(&out[0]).unwrap();
    let (first_coord, first_player) = moves[0];
    assert_eq!(accepted.coord, Some(first_coord));
    assert_eq!(accepted.player, first_player);

    clean_streams(streams_to_clean, &pool);
    clean_keys(keys_to_clean, &pool);
}

fn clean_keys(keys: Vec<String>, pool: &Pool) {
    let mut conn = pool.get().unwrap();
    for k in keys {
        conn.del(k.clone()).unwrap()
    }
}

fn clean_streams(stream_names: Vec<String>, pool: &Pool) {
    let mut conn = pool.get().unwrap();
    for sn in stream_names {
        match redis::cmd("XTRIM")
            .arg(&sn)
            .arg("MAXLEN")
            .arg("0")
            .query::<u32>(&mut *conn)
        {
            Err(e) => println!("Error in cleanup {}", e),
            Ok(count) => println!("Cleaned {} in {}", count, sn),
        }
    }
}
