extern crate chrono;
extern crate r2d2_redis;
extern crate redis;
extern crate rusoto_ec2;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate uuid;

pub mod ec2_startup;
pub mod subscriber;

use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

pub type ClientId = Uuid;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct WakeUpEvent {
    pub client_id: ClientId,
}
