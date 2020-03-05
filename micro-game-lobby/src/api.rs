#[derive(Debug, Clone)]
pub struct FindPublicGame {
    pub client_id: ClientId,
    pub session_id: SessionId,
}

/// game_id is empty in case gateway doesn't send one
#[derive(Debug, Clone)]
pub struct CreateGame {
    pub client_id: ClientId,
    pub visibility: Visibility,
    pub game_id: Option<GameId>,
    pub session_id: SessionId,
    pub board_size: u8,
}
#[derive(Debug, Clone)]
pub struct JoinPrivateGame {
    pub game_id: GameId,
    pub client_id: ClientId,
    pub session_id: SessionId,
}

/// This event is issued when someone has created
/// a game and is waiting for their opponent to join.
#[derive(Debug, Clone)]
pub struct WaitForOpponent {
    pub game_id: GameId,
    pub session_id: SessionId,
    pub event_id: EventId,
}
#[derive(Debug, Clone)]
pub struct GameReady {
    pub game_id: GameId,
    pub sessions: (SessionId, SessionId),
    pub event_id: EventId,
    pub board_size: u8,
}

#[derive(Debug, Clone)]
pub struct PrivateGameRejected {
    pub game_id: GameId,
    pub client_id: ClientId,
    pub event_id: EventId,
    pub session_id: SessionId,
}

/// This event is emitted from gateway whenever a session disconnects.
/// It drives the cleanup of abandoned games in the lobby
#[derive(Debug, Clone)]
pub struct SessionDisconnected {
    pub session_id: SessionId,
}
