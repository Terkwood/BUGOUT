use dotenv::dotenv;
use std::env;

const ENV_ALLOWED_IDLE_SECS: &str = "ALLOWED_IDLE_SECS";
const ENV_STARTUP_GRACE_SECS: &str = "STARTUP_GRACE_SECS";
const ENV_TAG_NAME: &str = "INSTANCE_TAG_NAME";
const ENV_AWS_REGION: &str = "AWS_REGION";
const ENV_DISABLED: &str = "DISABLED";

const DEFAULT_ALLOWED_IDLE_SECS: u64 = 300;
const DEFAULT_STARTUP_GRACE_SECS: u64 = 300;
const DEFAULT_INSTANCE_TAG_NAME: &str = "TOO_EXPENSIVE";
const DEFAULT_REGION: &str = "us-east-1";
const DEFAULT_DISABLED: bool = false;

lazy_static! {
    pub static ref ALLOWED_IDLE_SECS: u64 = env::var(ENV_ALLOWED_IDLE_SECS)
        .map(|s| s.parse::<u64>().unwrap_or(DEFAULT_ALLOWED_IDLE_SECS))
        .unwrap_or(DEFAULT_ALLOWED_IDLE_SECS);
    pub static ref STARTUP_GRACE_SECS: u64 = env::var(ENV_STARTUP_GRACE_SECS)
        .map(|s| s.parse::<u64>().unwrap_or(DEFAULT_STARTUP_GRACE_SECS))
        .unwrap_or(DEFAULT_STARTUP_GRACE_SECS);
    pub static ref INSTANCE_TAG_NAME: String =
        env::var(ENV_TAG_NAME).unwrap_or(DEFAULT_INSTANCE_TAG_NAME.to_string());
    pub static ref AWS_REGION: String =
        env::var(ENV_AWS_REGION).unwrap_or(DEFAULT_REGION.to_string());
    pub static ref DISABLED: bool = env::var(ENV_DISABLED)
        .map(|s| s.parse::<bool>().unwrap_or(DEFAULT_DISABLED))
        .unwrap_or(DEFAULT_DISABLED);
}

pub fn init() {
    dotenv().ok();

    println!("🏞 environment vars 🏞");
    println!("ALLOWED_IDLE_SECS {}", *ALLOWED_IDLE_SECS);
    println!("STARTUP_GRACE_SECS {}", *STARTUP_GRACE_SECS);
    println!("AWS_REGION {}", *AWS_REGION);
    println!("INSTANCE_TAG_NAME {}", *INSTANCE_TAG_NAME);
    println!("DISABLED {}", *DISABLED);
    println!("\n\n");
}
