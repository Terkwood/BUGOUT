use color_model as color;
use lobby_model as lobby;
use move_model as moves;
use sync_model as sync;

#[derive(Clone, Debug)]
pub enum StreamData {
    BotAttached(micro_model_bot::gateway::BotAttached),
    MoveMade(moves::MoveMade),
    HistoryProvided(sync::api::HistoryProvided),
    SyncReply(sync::api::SyncReply),
    WaitForOpponent(lobby::api::WaitForOpponent),
    GameReady(lobby::api::GameReady),
    PrivGameRejected(lobby::api::PrivateGameRejected),
    ColorsChosen(color::api::ColorsChosen),
}

impl From<sync_model::api::HistoryProvided> for StreamData {
    fn from(a: sync_model::api::HistoryProvided) -> Self {
        StreamData::HistoryProvided(a)
    }
}
impl From<sync_model::api::SyncReply> for StreamData {
    fn from(h: sync_model::api::SyncReply) -> Self {
        StreamData::SyncReply(h)
    }
}
impl From<lobby_model::api::WaitForOpponent> for StreamData {
    fn from(w: lobby_model::api::WaitForOpponent) -> Self {
        StreamData::WaitForOpponent(w)
    }
}
impl From<lobby_model::api::GameReady> for StreamData {
    fn from(w: lobby_model::api::GameReady) -> Self {
        StreamData::GameReady(w)
    }
}
impl From<lobby_model::api::PrivateGameRejected> for StreamData {
    fn from(w: lobby_model::api::PrivateGameRejected) -> Self {
        StreamData::PrivGameRejected(w)
    }
}
impl From<color_model::api::ColorsChosen> for StreamData {
    fn from(w: color_model::api::ColorsChosen) -> Self {
        StreamData::ColorsChosen(w)
    }
}
