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
use std::panic;
use std::thread;
use std::time::Duration;
use topics::StreamTopics;
use uuid::Uuid;

const USAGE: &str = "CAUTION!  YOU MUST RUN tests/integration.rs ON A SINGLE THREAD!
Use one of the following:

    cargo test -- --test-threads=1

    cargo watch -x \"test -- --test-threads=1\"
";
const TEST_GAME_STATES_TOPIC: &str = "bugtest-game-states";
const TEST_MAKE_MOVE_CMD_TOPIC: &str = "bugtest-make-move-cmd";

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
    }));
}

#[test]
fn test_redis_connect() {
    let pool = redis_pool();
    let mut conn = pool.get().unwrap();
    assert_eq!(
        redis::cmd("PING").query::<String>(&mut *conn).unwrap(),
        "PONG"
    );
}

#[test]
fn test_track_emitted_game_states() {
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

    let game_state = GameState::default();
    redis::cmd("XADD")
        .arg(TEST_GAME_STATES_TOPIC)
        .arg("MAXLEN")
        .arg("~")
        .arg("1000")
        .arg("*")
        .arg("game_id")
        .arg(game_id.0.to_string())
        .arg("data")
        .arg(game_state.serialize().unwrap())
        .query::<String>(&mut *conn)
        .unwrap();

    const WAIT_MS: u64 = 100;
    let mut found: Option<Vec<u8>> = None;
    const INIT_RETRIES: u8 = 100;
    let mut retries = INIT_RETRIES;
    while retries > 0 {
        let data: Result<Option<Vec<u8>>, _> = conn.get(&data_key);
        if let Ok(Some(h)) = data {
            found = Some(h.clone());
            break;
        } else {
            thread::sleep(Duration::from_millis(WAIT_MS));
            retries -= 1;
        }
    }
    assert!(found.is_some());
    let f = found.unwrap();
    assert!(f.len() > 0);
    let d = GameState::from(&f);
    assert_eq!(game_state, d.unwrap());
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
}

#[test]
fn test_moves_processed() {
    let pool = redis_pool();
    let streams_to_clean = vec![
        TEST_GAME_STATES_TOPIC.to_string(),
        TEST_MAKE_MOVE_CMD_TOPIC.to_string(),
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
    for move_to_make in moves {
        redis::cmd("XADD")
            .arg(TEST_MAKE_MOVE_CMD_TOPIC)
            .arg("MAXLEN")
            .arg("~")
            .arg("1000")
            .arg("*")
            .arg("game_id")
            .arg(game_id.0.to_string())
            .arg("data")
            .arg(current_game_state.serialize().unwrap())
            .query::<String>(&mut *conn)
            .unwrap();
    }

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
