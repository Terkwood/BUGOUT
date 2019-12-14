extern crate chrono;
extern crate dotenv;
#[macro_use]
extern crate lazy_static;
extern crate r2d2_redis;
extern crate redis;
extern crate rusoto_ec2;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate uuid;

pub mod env;
pub mod subscriber;
pub mod wakeup;

use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

pub type ClientId = Uuid;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct WakeUpEvent {
    pub client_id: ClientId,
}
