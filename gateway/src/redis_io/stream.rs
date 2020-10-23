use super::xread::XReader;
use crate::backend::events as be;
use crate::backend::events::BackendEvents;
use crate::model::{ColorsChosenEvent, HistoryProvidedEvent, MoveMadeEvent};
use color_model as color;
use crossbeam_channel::Sender;
use lobby_model as lobby;
use log::{error, info};
use move_model as moves;
use redis_streams::XReadEntryId;
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

pub fn process(events_in: Sender<BackendEvents>, opts: StreamOpts) {
    loop {
        match opts.xreader.xread_sorted() {
            Err(e) => error!("cannot xread {:?}", e),
            Ok(xrr) => {
                for (xid, data) in xrr {
                    match &data {
                        StreamData::HistoryProvided(_) => info!("📥 Stream HistoryProvided"),
                        StreamData::SyncReply(_) => info!("📥 Stream SyncReply"),
                        _ => info!("📥 Stream: {:?}", &data),
                    }

                    let dc = data.clone();
                    process_event(xid, data, &events_in, &opts);
                    info!("🏞 OK {:?}", dc)
                }
            }
        }

        todo!("ack")
    }
}

fn process_event(
    xid: XReadEntryId,
    data: StreamData,
    events_in: &Sender<BackendEvents>,
    opts: &StreamOpts,
) {
    if let Err(e) = events_in.send(BackendEvents::from(data)) {
        error!("send backend event {:?}", e)
    }

    todo!("think about ack")
}

pub struct StreamOpts {
    pub xreader: Box<dyn XReader>,
}

impl From<StreamData> for BackendEvents {
    fn from(stream_data: StreamData) -> Self {
        match stream_data {
            StreamData::MoveMade(m) => BackendEvents::MoveMade(MoveMadeEvent::from(m)),
            StreamData::BotAttached(b) => BackendEvents::BotAttached(b),
            StreamData::HistoryProvided(h) => {
                BackendEvents::HistoryProvided(HistoryProvidedEvent::from(h))
            }
            StreamData::SyncReply(s) => {
                BackendEvents::SyncReply(be::SyncReplyBackendEvent::from(s))
            }
            StreamData::WaitForOpponent(w) => {
                be::BackendEvents::WaitForOpponent(be::WaitForOpponentBackendEvent::from(w))
            }
            StreamData::GameReady(g) => {
                be::BackendEvents::GameReady(be::GameReadyBackendEvent::from(g))
            }
            StreamData::PrivGameRejected(p) => {
                be::BackendEvents::PrivateGameRejected(be::PrivateGameRejectedBackendEvent::from(p))
            }
            StreamData::ColorsChosen(c) => {
                be::BackendEvents::ColorsChosen(ColorsChosenEvent::from(c))
            }
        }
    }
}
