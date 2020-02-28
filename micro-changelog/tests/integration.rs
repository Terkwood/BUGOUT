extern crate micro_changelog;

use micro_changelog::redis_conn_pool;
use micro_changelog::redis_conn_pool::RedisHostUrl;
use micro_changelog::{r2d2, r2d2_redis, redis};

const GAME_STATES_TOPIC: &str = "bugtest-game-states";
const MOVE_ACCEPTED_EV_TOPIC: &str = "bugtest-move-accepted-ev";
const MOVE_MADE_EV_TOPIC: &str = "bugtest-move-made-ev";

fn redis_pool() -> r2d2::Pool<r2d2_redis::RedisConnectionManager> {
    redis_conn_pool::create(RedisHostUrl("redis://localhost".to_string()))
}

#[test]
fn test_process_move() {}
