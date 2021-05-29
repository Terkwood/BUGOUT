use crate::repo::GameLobbyRepo;
use crate::stream::XAdd;
use redis_streams::{RedisSortedStreams, SortedStreams};
use std::rc::Rc;

pub struct Components {
    pub game_lobby_repo: Box<dyn GameLobbyRepo>,
    pub xadd: Box<dyn XAdd>,
    pub sorted_streams: Box<dyn SortedStreams>,
}

const REDIS_URL: &str = "redis://redis/";

pub fn redis_client() -> Rc<redis::Client> {
    Rc::new(redis::Client::open(REDIS_URL).expect("redis client"))
}

impl Components {
    pub fn new(client: Rc<redis::Client>) -> Self {
        let mut conn = client.get_connection().expect("redis conn");
        let stream_handlers: Vec<(
            &str,
            Box<
                FnMut(
                    redis_streams::XId,
                    &redis_streams::Message,
                ) -> Result<(), redis_streams::anyhow::Error>,
            >,
        )> = vec![
            ("some-stream", todo!()),
            ("another-stream", todo!()),
            ("fix-the-names", todo!()),
        ];
        let sorted_streams =
            RedisSortedStreams::xgroup_create_mkstreams(stream_handlers, todo!("opts"), &mut conn)
                .expect("stream creation");
        Components {
            game_lobby_repo: Box::new(client.clone()),
            xadd: Box::new(client),
            sorted_streams: Box::new(sorted_streams),
        }
    }
}
