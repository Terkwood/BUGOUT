use envy;
use serde_derive::Deserialize;

pub fn init() {
    dotenv::dotenv().ok();
}

#[derive(Deserialize, Debug)]
struct Env {
    hash_salt: Option<String>,
    link_to: Option<String>,
}

lazy_static! {
    static ref ENV: Option<Env> = match envy::from_env::<Env>() {
        Ok(env) => Some(env),
        Err(_) => None,
    };
}

const DEFAULT_HASH_SALT: &str = "BUGOUT";
const DEFAULT_LINK_TO: &str = "http://localhost:8000";
lazy_static! {
    pub static ref HASH_SALT: String = ENV
        .as_ref()
        .and_then(|env| env.hash_salt.as_ref())
        .unwrap_or(&DEFAULT_HASH_SALT.to_string())
        .to_string();
    pub static ref LINK_TO: String = ENV
        .as_ref()
        .and_then(|env| env.link_to.as_ref())
        .unwrap_or(&DEFAULT_LINK_TO.to_string())
        .to_string();
}
