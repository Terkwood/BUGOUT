use std::default::Default;
use std::str::FromStr;

use crossbeam_channel::select;

use rusoto_core::Region;
use rusoto_ec2::{DescribeInstancesRequest, Ec2, Ec2Client, StopInstancesRequest, Tag};

use crate::env::*;
use crate::model::ShutdownCommand;

const TAG_KEY: &str = "Name";

pub fn listen(shutdown_out: crossbeam::Receiver<ShutdownCommand>) {
    let client = Ec2Client::new(region());

    loop {
        select! {
            recv(shutdown_out) -> command =>
                match command {
                    Ok(_) => shutdown(&client),
                    Err(e) => println!("Failed to select shutdown_out {}", e)
                }
        }
    }
}

fn shutdown(client: &Ec2Client) {
    let instance_id: Option<String> = big_box_instance_id(&client);

    if let Some(id) = instance_id {
        let stop_request: StopInstancesRequest = StopInstancesRequest {
            instance_ids: vec![id.to_string()],
            ..Default::default()
        };

        match client.stop_instances(stop_request).sync() {
            Ok(output) => println!("OK: {:?}", output),
            Err(error) => println!("Error: {:?}", error),
        }

        println!("Shutting down instance {}...", id.to_string())
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
            } if tag_key == TAG_KEY && v == INSTANCE_TAG_NAME.to_string() => return true,
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
