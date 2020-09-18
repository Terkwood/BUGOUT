use dotenv::dotenv;
use std::env;
const ENV_AUTHORIZATION: &str = "AUTHORIZATION";
const ENV_BOTLINK_URL: &str = "BOTLINK_URL";
const ENV_MODEL_FILE: &str = "MODEL_FILE";

const DEFAULT_BOTLINK_URL: &str = "ws://127.0.0.1:3012";
const DEFAULT_MODEL_FILE: &str = "g170e-b20c256x2-s2430231552-d525879064.bin.gz";

lazy_static! {
    pub static ref AUTHORIZATION: Option<String> = env::var(ENV_AUTHORIZATION).ok();
    pub static ref BOTLINK_URL: String =
        env::var(ENV_BOTLINK_URL).unwrap_or_else(|_| DEFAULT_BOTLINK_URL.to_string());
    pub static ref MODEL_FILE: String =
        env::var(ENV_MODEL_FILE).unwrap_or_else(|_| DEFAULT_MODEL_FILE.to_string());
}

pub fn init() {
    dotenv().ok();
}
