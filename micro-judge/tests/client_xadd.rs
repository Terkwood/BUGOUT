extern crate micro_judge;
extern crate r2d2_redis;

use conn_pool::Pool;
use conn_pool::RedisHostUrl;
use micro_judge::io::entry_id::EntryIdRepo;
use micro_judge::io::{conn_pool, redis_keys, stream, topics};
use micro_judge::model::{GameId, GameState};
use r2d2_redis::{r2d2, redis};
use redis::Commands;
use redis_keys::Namespace;
use std::panic;
use std::thread;
use std::time::Duration;
use topics::StreamTopics;

const TEST_GAME_STATES_TOPIC: &str = "bugtest-game-states";
const TEST_MAKE_MOVE_CMD_TOPIC: &str = "bugtest-make-move-cmd";

fn redis_pool() -> r2d2::Pool<r2d2_redis::RedisConnectionManager> {
    conn_pool::create(RedisHostUrl("redis://localhost".to_string()))
}

fn test_opts() -> stream::ProcessOpts {
    let namespace = Namespace("BUGTEST".to_string());
    stream::ProcessOpts {
        topics: StreamTopics {
            make_move_cmd: TEST_MAKE_MOVE_CMD_TOPIC.to_string(),
            game_states_changelog: TEST_GAME_STATES_TOPIC.to_string(),
        },
        namespace: namespace.clone(),
        entry_id_repo: EntryIdRepo {
            namespace,
            pool: redis_pool(),
        },
    }
}

fn panic_cleanup(stream_names: Vec<String>, keys: Vec<String>, pool: Pool) {
    panic::set_hook(Box::new(move |e| {
        println!("{}", e);
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
    let data_key = redis_keys::game_states_key(&test_opts().namespace, &game_id);
    let keys_to_clean = vec![
        data_key.clone(),
        redis_keys::entry_ids_hash_key(&test_opts().namespace),
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

    const WAIT_MS: u64 = 300;
    let mut found: Option<Vec<u8>> = None;
    let mut retries = 10;
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
    let entry_ids = eid_repo.fetch_all().unwrap();
    assert!(entry_ids.game_states_eid.millis_time > 0);

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
