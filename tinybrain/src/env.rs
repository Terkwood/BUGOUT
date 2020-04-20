use dotenv::dotenv;
use std::env;
const ENV_AUTHORIZATION: &str = "AUTHORIZATION";
const ENV_BOTLINK_URL: &str = "BOTLINK_URL";

const DEFAULT_BOTLINK_URL: &str = "ws://127.0.0.1:3012";

lazy_static! {
    pub static ref AUTHORIZATION: Option<String> = env::var(ENV_AUTHORIZATION).ok();
    pub static ref BOTLINK_URL: String =
        env::var(ENV_BOTLINK_URL).unwrap_or_else(|_| DEFAULT_BOTLINK_URL.to_string());
}

pub fn init() {
    dotenv().ok();
}
