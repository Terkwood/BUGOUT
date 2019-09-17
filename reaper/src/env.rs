use dotenv::dotenv;
use std::env;

const ENV_TAG_NAME: &str = "INSTANCE_TAG_NAME";
const ENV_AWS_ROLE_ARN: &str = "AWS_ROLE_ARN";
const ENV_AWS_REGION: &str = "AWS_REGION";

lazy_static! {
    pub static ref INSTANCE_TAG_NAME: String = env_var(ENV_TAG_NAME);
    // TODO DEAD CODE
    pub static ref AWS_ROLE_ARN: String = env_var(ENV_AWS_ROLE_ARN);
    pub static ref AWS_REGION: String = env_var(ENV_AWS_REGION);
}

pub fn init() {
    dotenv().ok();
}

fn env_var(v: &str) -> String {
    if let Ok(i) = env::var(v) {
        i
    } else {
        panic!("You must specify {} in .env file", v)
    }
}
