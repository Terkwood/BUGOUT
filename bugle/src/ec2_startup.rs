use crate::env::*;

use chrono::prelude::*;
use chrono::{DateTime, Utc};
use r2d2_redis::redis;
use redis::Commands;
use std::str::FromStr;

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
    let client = Ec2Client::new(region());
    match get_last_startup(&pool) {
        None => go(pool),
        Some(t) if Utc::now().timestamp() - t.timestamp() > DELAY_SECS as i64 => go(pool),
        Some(_) => println!("pass"),
    }
}

fn big_box_instance_id(client: &Ec2Client) -> Option<String> {
    let desc_request: DescribeInstancesRequest = Default::default();

    let mut instance_id: Option<String> = None;

    match client.describe_instances(desc_request).sync() {
        Ok(d) => {
            if let Some(rs) = d.reservations {
                for r in rs {
                    if let Some(is) = r.instances {
                        for i in is {
                            let id = i.instance_id;
                            if let Some(tags) = i.tags {
                                if has_required_name(tags) {
                                    instance_id = id
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(error) => println!("Error: {:?}", error),
    }

    instance_id
}

fn has_required_name(tags: Vec<Tag>) -> bool {
    for tag in tags {
        match tag {
            Tag {
                key: Some(tag_key),
                value: Some(v),
            } if tag_key == EC2_TAG_KEY && v == INSTANCE_TAG_NAME.to_string() => return true,
            _ => (),
        }
    }

    return false;
}

fn region() -> Region {
    match Region::from_str(&AWS_REGION.to_string()) {
        Ok(r) => r,
        Err(_e) => panic!("Failed to set AWS region"),
    }
}
