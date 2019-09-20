use dotenv::dotenv;
use std::env;

const ENV_ALLOWED_IDLE_SECS: &str = "ALLOWED_IDLE_SECS";
const ENV_TAG_NAME: &str = "INSTANCE_TAG_NAME";
const ENV_AWS_REGION: &str = "AWS_REGION";
const ENV_DISABLED: &str = "DISABLED";

const DEFAULT_ALLOWED_IDLE_SECS: i64 = 300;
const DEFAULT_REGION: &str = "us-east-1";
const DEFAULT_DISABLED: bool = false;

lazy_static! {
    pub static ref ALLOWED_IDLE_SECS: i64 = env::var(ENV_ALLOWED_IDLE_SECS)
        .map(|s| s.parse::<i64>().unwrap_or(DEFAULT_ALLOWED_IDLE_SECS))
        .unwrap_or(DEFAULT_ALLOWED_IDLE_SECS);
    pub static ref INSTANCE_TAG_NAME: String = if let Ok(i) = env::var(ENV_TAG_NAME) {
        i
    } else {
        panic!("You must specify INSTANCE_TAG_NAME in .env file")
    };
    pub static ref AWS_REGION: String =
        env::var(ENV_AWS_REGION).unwrap_or(DEFAULT_REGION.to_string());
    pub static ref DISABLED: bool = env::var(ENV_DISABLED)
        .map(|s| s.parse::<bool>().unwrap_or(DEFAULT_DISABLED))
        .unwrap_or(DEFAULT_DISABLED);
}

pub fn init() {
    dotenv().ok();
}
