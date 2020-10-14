use redis::{Client, Commands};
use redis_streams::XReadEntryId;

pub fn xack(
    key: &str,
    group: &str,
    ids: &[XReadEntryId],
    client: &Client,
) -> Result<(), redis::RedisError> {
    let c = client.get_connection();
    if let Ok(mut conn) = c {
        let idstrs: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
        let _: usize = conn.xack(key, group, &idstrs)?;
        Ok(())
    } else {
        c.map(|_| ())
    }
}
