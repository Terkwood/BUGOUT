extern crate r2d2_redis;
extern crate redis;
extern crate uuid;

pub mod conn_pool;
mod model;
pub mod stream;
mod topics;

use uuid::Uuid;
