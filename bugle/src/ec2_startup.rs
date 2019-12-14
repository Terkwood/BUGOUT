use chrono::prelude::*;
use chrono::{DateTime, Utc};
use r2d2_redis::redis;
use redis::Commands;

use rusoto_core::Region;
use rusoto_ec2::{DescribeInstancesRequest, Ec2, Ec2Client, StopInstancesRequest, Tag};

const EC2_TAG_KEY: &str = "Name";

type RedisPool = r2d2_redis::r2d2::Pool<r2d2_redis::RedisConnectionManager>;

const DELAY_SECS: u32 = 60;

const REDIS_LAST_STARTUP_KEY: &str = "bugle/last_startup";

fn get_last_startup(pool: &RedisPool) -> Option<DateTime<Utc>> {
    let mut conn = pool.get().unwrap();
    if let Ok(epoch) = conn.get(REDIS_LAST_STARTUP_KEY) {
        println!("Got val {}", epoch);

        Some(Utc.timestamp(epoch, 0))
    } else {
        None
    }
}

fn go(pool: &RedisPool) {
    ec2_init_instance();
    if let Err(e) = set_last_startup(pool) {
        println!("err {}", e)
    }
}
fn ec2_init_instance() {
    println!("hiya")
}
fn set_last_startup(pool: &RedisPool) -> Result<(), redis::RedisError> {
    let mut conn = pool.get().unwrap();
    let ts = Utc::now().timestamp();
    conn.set(REDIS_LAST_STARTUP_KEY, ts)
}

pub fn startup(pool: &RedisPool) {
    match get_last_startup(&pool) {
        None => go(pool),
        Some(t) if Utc::now().timestamp() - t.timestamp() > DELAY_SECS as i64 => go(pool),
        Some(_) => println!("pass"),
    }
}
