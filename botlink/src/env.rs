use std::env;

const ENV_AUTHORIZATION: &str = "AUTHORIZATION";

lazy_static! {
    pub static ref AUTHORIZATION: Option<String> = env::var(ENV_AUTHORIZATION).ok();
}

pub fn init() {
    dotenv::dotenv().ok();
}
