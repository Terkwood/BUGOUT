use dotenv::dotenv;
use std::env;

const ENV_DELAY_SECS: &str = "DELAY_SECS";
const ENV_TAG_NAME: &str = "INSTANCE_TAG_NAME";
const ENV_AWS_REGION: &str = "AWS_REGION";

const DEFAULT_DELAY_SECS: u64 = 30;
const DEFAULT_INSTANCE_TAG_NAME: &str = "TOO_EXPENSIVE";
const DEFAULT_REGION: &str = "us-east-1";

lazy_static! {
    pub static ref DELAY_SECS: u64 = env::var(ENV_DELAY_SECS)
        .map(|s| s.parse::<u64>().unwrap_or(DEFAULT_DELAY_SECS))
        .unwrap_or(DEFAULT_DELAY_SECS);
    pub static ref INSTANCE_TAG_NAME: String =
        env::var(ENV_TAG_NAME).unwrap_or(DEFAULT_INSTANCE_TAG_NAME.to_string());
    pub static ref AWS_REGION: String =
        env::var(ENV_AWS_REGION).unwrap_or(DEFAULT_REGION.to_string());
}

pub fn init() {
    dotenv().ok();

    println!("🎪 environment vars 🎪");
    println!("DELAY_SECS {}", *DELAY_SECS);
    println!("AWS_REGION {}", *AWS_REGION);
    println!("INSTANCE_TAG_NAME {}", *INSTANCE_TAG_NAME);
    println!("\n\n");
}
