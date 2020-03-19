use std::env;

const ENV_AUTHORIZATION: &str = "AUTHORIZATION";
const ENV_ADDRESS: &str = "ADDRESS";

const DEFAULT_ADDRESS: &str = "127.0.0.1:3012";
lazy_static! {
    pub static ref AUTHORIZATION: Option<String> = env::var(ENV_AUTHORIZATION).ok();
    pub static ref ADDRESS: String = env::var(ENV_ADDRESS)
        .unwrap_or(DEFAULT_ADDRESS.to_string())
        .to_string();
}

pub fn init() {
    dotenv::dotenv().ok();
}
