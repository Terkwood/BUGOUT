use dotenv::dotenv;
use std::env;
const ENV_AUTHORIZATION: &str = "AUTHORIZATION";
const ENV_ROBOCALL_URL: &str = "ROBOCALL_URL";

const DEFAULT_ROBOCALL_URL: &str = "ws://127.0.0.1:3012";

lazy_static! {
    pub static ref AUTHORIZATION: Option<String> = env::var(ENV_AUTHORIZATION).ok();
    pub static ref ROBOCALL_URL: String =
        env::var(ENV_ROBOCALL_URL).unwrap_or(DEFAULT_ROBOCALL_URL.to_string());
}

pub fn init() {
    dotenv().ok();
}
