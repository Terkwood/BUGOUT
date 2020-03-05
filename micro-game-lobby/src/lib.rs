pub mod stream;

use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct StreamTopics {}
#[derive(Debug, Clone)]
pub struct Components {}
#[derive(Debug, Clone)]
pub struct GameId(pub Uuid);
#[derive(Debug, Clone)]
pub struct ClientId(pub Uuid);
#[derive(Debug, Clone)]
pub struct SessionId(pub Uuid);
#[derive(Debug, Clone)]
pub struct EventId(pub Uuid);
impl EventId {
    pub fn new() -> Self {
        EventId(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Visibility {
    Public,
    Private,
}
