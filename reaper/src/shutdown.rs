use std::default::Default;
use std::str::FromStr;

use futures::future::Future;

use crossbeam_channel::select;

use rusoto_core::{HttpClient, ProvideAwsCredentials, Region};
use rusoto_credential::ChainProvider;
use rusoto_ec2::{
    DescribeInstancesRequest, DescribeSpotInstanceRequestsRequest, Ec2, Ec2Client,
    StopInstancesRequest, Tag,
};

use rusoto_sts::{
    StsAssumeRoleSessionCredentialsProvider, StsClient, StsSessionCredentialsProvider,
};

use crate::env::*;
use crate::model::ShutdownCommand;

const TAG_KEY: &str = "Name";

pub fn listen(shutdown_out: crossbeam::Receiver<ShutdownCommand>) {
    /*let mut chain = ChainProvider::new();

    let client = Ec2Client::new_with(
        HttpClient::new().expect("failed to create request dispatcher"),
        chain,
        region(),
    );*/

    let client = ec2_client_with_role();

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

// TODO WASTED // TODO WASTED // TODO WASTED
/// Assume a role which can shut something down
fn ec2_client_with_role() -> Ec2Client {
    let sts = StsClient::new(region());

    let sts_creds_provider = StsSessionCredentialsProvider::new(sts.clone(), None, None);
    /*
    match sts_creds_provider.credentials().wait() {
        Err(e) => panic!("sts credentials provider error: {:?}", e),
        Ok(r) => println!("sts credentials provider result: {:?}", r),
    }*/

    println!("OH HEY THERE");

    let provider = StsAssumeRoleSessionCredentialsProvider::new(
        sts,
        AWS_ROLE_ARN.to_string(),
        "default".to_string(),
        None,
        None,
        None,
        None,
    );

    println!("AND HI");

    let client = Ec2Client::new_with(HttpClient::new().unwrap(), provider, region());

    client
}
