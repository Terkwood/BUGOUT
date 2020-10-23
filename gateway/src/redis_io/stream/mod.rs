mod unacknowledged;
mod write;
mod xack;
pub mod xadd;
pub mod xread;

pub use unacknowledged::*;
pub use write::write_loop;

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
use xack::XAck;
use xread::XReader;

pub const GROUP_NAME: &str = "micro-gateway";

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

pub struct StreamOpts {
    pub xread: Box<dyn XReader>,
    pub xack: Box<dyn XAck>,
}

pub fn read_loop(events_in: Sender<BackendEvents>, opts: StreamOpts) {
    let mut unacked = Unacknowledged::default();
    loop {
        match opts.xread.xread_sorted() {
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

        unacked.ack_all(opts.xack.as_ref())
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
