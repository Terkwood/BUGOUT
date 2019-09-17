use dotenv::dotenv;
use std::env;

const ENV_TAG_NAME: &str = "INSTANCE_TAG_NAME";
const ENV_AWS_REGION: &str = "AWS_REGION";
const DEFAULT_REGION: &str = "us-east-1";

lazy_static! {
    pub static ref INSTANCE_TAG_NAME: String = if let Ok(i) = env::var(ENV_TAG_NAME) {
        i
    } else {
        panic!("You must specify INSTANCE_TAG_NAME in .env file")
    };
    pub static ref AWS_REGION: String =
        env::var(ENV_AWS_REGION).unwrap_or(DEFAULT_REGION.to_string());
}

pub fn init() {
    dotenv().ok();
}
