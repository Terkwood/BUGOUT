extern crate micro_changelog;

use micro_changelog::redis_conn_pool;
use micro_changelog::redis_conn_pool::{Pool, RedisHostUrl};
use micro_changelog::repo::redis_key::RedisKeyNamespace;
use micro_changelog::{r2d2, r2d2_redis, redis};
use redis::Commands;

const GAME_STATES_TOPIC: &str = "bugtest-game-states";
const MOVE_ACCEPTED_EV_TOPIC: &str = "bugtest-move-accepted-ev";
const MOVE_MADE_EV_TOPIC: &str = "bugtest-move-made-ev";

#[test]
fn test_process_move() {
    let keys_to_clean = vec![];
    let streams_to_clean = vec![];
    let pool = redis_pool();
    panic_cleanup(streams_to_clean, keys_to_clean, pool.clone());

    todo!("TEST");

    clean_streams(streams_to_clean, &pool);
    clean_keys(keys_to_clean, &pool);
}

fn panic_cleanup(stream_names: Vec<String>, keys: Vec<String>, pool: Pool) {
    std::panic::set_hook(Box::new(move |e| {
        println!("{:#?}", e);
        clean_streams(stream_names.clone(), &pool);
        clean_keys(keys.clone(), &pool);
    }));
}

fn redis_pool() -> r2d2::Pool<r2d2_redis::RedisConnectionManager> {
    redis_conn_pool::create(RedisHostUrl("redis://localhost".to_string()))
}
fn test_namespace() -> RedisKeyNamespace {
    RedisKeyNamespace("BUGTEST".to_string())
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
