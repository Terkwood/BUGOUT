use super::*;
use core_model::SessionId;
pub trait PrefsRepo {
    fn get(&self, session_id: &SessionId) -> Result<Option<SessionColorPref>, FetchErr>;
    fn put(&self, scp: &SessionColorPref) -> Result<(), WriteErr>;
}

impl PrefsRepo for Rc<Client> {
    fn get(&self, session_id: &SessionId) -> Result<Option<SessionColorPref>, FetchErr> {
        if let Ok(mut conn) = self.get_connection() {
            let key = redis_key(session_id);
            let data: Result<Vec<u8>, _> = conn.get(&key).map_err(|_| FetchErr);

            if let Ok(_) = data {
                touch_ttl(&mut conn, &key)
            }

            data.and_then(|bytes| bincode::deserialize(&bytes).map_err(|_| FetchErr))
        } else {
            Err(FetchErr)
        }
    }

    fn put(&self, scp: &SessionColorPref) -> Result<(), WriteErr> {
        let c = self.get_connection();
        let s = bincode::serialize(scp);
        if let (Ok(mut conn), Ok(bytes)) = (c, s) {
            let key = redis_key(&scp.session_id);

            let done: Result<(), _> = conn.set(&key, bytes.clone());
            if let Ok(_) = done {
                touch_ttl(&mut conn, &key)
            }

            done.map_err(|_| WriteErr)
        } else {
            Err(WriteErr)
        }
    }
}

fn redis_key(session_id: &SessionId) -> String {
    format!("/BUGOUT/micro_color_chooser/prefs/{}", session_id.0)
}
