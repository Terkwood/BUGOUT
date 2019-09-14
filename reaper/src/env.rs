use dotenv::dotenv;
use std::env;

const ENV_TAG_NAME: &str = "INSTANCE_TAG_NAME";

lazy_static! {
    pub static ref TAG_NAME: String = tag_name();
}

pub fn init() {
    dotenv().ok();
}

fn tag_name() -> String {
    if let Ok(i) = env::var(ENV_TAG_NAME) {
        i
    } else {
        panic!("Specify desired instance nametag value in env")
    }
}
