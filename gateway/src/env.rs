use envy;
use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
struct Env {
    hash_salt: Option<String>,
}

lazy_static! {
    static ref ENV: Option<Env> = match envy::from_env::<Env>() {
        Ok(env) => Some(env),
        Err(_) => None,
    };
}

const DEFAULT_HASH_SALT: &str = "BUGOUT";
lazy_static! {
    pub static ref HASH_SALT: String = ENV
        .as_ref()
        .and_then(|env| env.hash_salt.as_ref())
        .unwrap_or(&DEFAULT_HASH_SALT.to_string())
        .to_string();
}
