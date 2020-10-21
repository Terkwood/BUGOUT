use std::env;

const ENV_AUTHORIZATION: &str = "AUTHORIZATION"; // username:password
const ENV_ADDRESS: &str = "ADDRESS";

const DEFAULT_ADDRESS: &str = "0.0.0.0:3012";
lazy_static! {
    pub static ref AUTHORIZATION: Option<String> = env::var(ENV_AUTHORIZATION).ok();
    pub static ref ADDRESS: String = env::var(ENV_ADDRESS).unwrap_or(DEFAULT_ADDRESS.to_string());
}

pub fn init() {
    dotenv::dotenv().ok();
}
