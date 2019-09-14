use std::default::Default;

use crate::env::INSTANCE_NAME;
use rusoto_core::Region;
use rusoto_ec2::{Ec2, Ec2Client, StopInstancesRequest};

pub fn shutdown() {
    let client = Ec2Client::new(Region::UsEast1);
    let request: StopInstancesRequest = StopInstancesRequest {
        instance_ids: vec![INSTANCE_NAME.to_string()],
        ..Default::default()
    };
    client.stop_instances(request);

    println!("Shutting down instance {}...", INSTANCE_NAME.to_string())
}
