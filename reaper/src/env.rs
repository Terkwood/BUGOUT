use dotenv::dotenv;
use std::env;

const ENV_INSTANCE_NAME: &str = "INSTANCE_NAME";
const DEFAULT_INSTANCE_NAME: &str = "UNKNOWN";

lazy_static! {
    pub static ref INSTANCE_NAME: String = instance_name();
}

pub fn init() {
    dotenv().ok();
}

fn instance_name() -> String {
    if let Ok(i) = env::var(ENV_INSTANCE_NAME) {
        i
    } else {
        DEFAULT_INSTANCE_NAME.to_string()
    }
}
