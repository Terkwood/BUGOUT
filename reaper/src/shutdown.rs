use std::default::Default;

use rusoto_core::Region;
use rusoto_ec2::StopInstancesRequest;

use crate::env::INSTANCE_NAME;

pub fn shutdown() {
    println!("Shutting down instance {}...", INSTANCE_NAME.to_string())
}
