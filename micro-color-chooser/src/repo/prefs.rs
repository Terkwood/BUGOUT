use super::*;
use core_model::SessionId;
use log::trace;

pub trait PrefsRepo {
    fn get(&self, session_id: &SessionId) -> Result<Option<SessionColorPref>, FetchErr>;
    fn put(&self, scp: &SessionColorPref) -> Result<(), WriteErr>;
}

impl PrefsRepo for Rc<Client> {
    fn get(&self, session_id: &SessionId) -> Result<Option<SessionColorPref>, FetchErr> {
        trace!("get {:?}", &session_id);
        if let Ok(mut conn) = self.get_connection() {
            let key = redis_key(session_id);
            let data: Result<Option<Vec<u8>>, _> = conn.get(&key).map_err(|_| FetchErr);

            match data {
                Ok(Some(bytes)) => {
                    touch_ttl(&mut conn, &key);
                    trace!("try to deser {:?}", &bytes);
                    bincode::deserialize(&bytes).map_err(|_| FetchErr)
                }
                Ok(None) => Ok(None),
                Err(_) => Err(FetchErr),
            }
        } else {
            Err(FetchErr)
        }
    }

    fn put(&self, scp: &SessionColorPref) -> Result<(), WriteErr> {
        let c = self.get_connection();
        let s = bincode::serialize(&scp);
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
