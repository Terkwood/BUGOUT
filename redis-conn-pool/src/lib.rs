pub extern crate redis;

pub const DEFAULT_HOST_URL: &str = "redis://redis";

pub struct RedisHostUrl(pub String);
impl Default for RedisHostUrl {
    fn default() -> Self {
        RedisHostUrl(DEFAULT_HOST_URL.to_string())
    }
}

pub fn create(host_url: RedisHostUrl) {
    // TODOlet manager = RedisConnectionManager::new(host_url.0).unwrap();
    todo!("Delete everything") //r2d2::Pool::builder().build(manager).unwrap()
}
