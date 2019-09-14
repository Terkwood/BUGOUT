use crate::env::INSTANCE_NAME;

pub fn shutdown() {
    println!("Shutting down instance {}...", INSTANCE_NAME.to_string())
}
