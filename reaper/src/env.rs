use dotenv::dotenv;
use std::env;

const ENV_INSTANCE_ID: &str = "INSTANCE_ID";

lazy_static! {
    pub static ref INSTANCE_ID: String = instance_name();
}

pub fn init() {
    dotenv().ok();
}

fn instance_name() -> String {
    if let Ok(i) = env::var(ENV_INSTANCE_ID) {
        i
    } else {
        panic!("Specify instance ID in env")
    }
}
