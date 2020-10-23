mod data;
mod unacknowledged;
mod write;
mod xack;
pub mod xadd;
pub mod xread;

pub use data::*;
pub use unacknowledged::*;
pub use write::write_loop;

use crate::backend::events as be;
use crate::backend::events::BackendEvents;
use crate::model::{ColorsChosenEvent, HistoryProvidedEvent, MoveMadeEvent};
use crossbeam_channel::Sender;
use log::{error, info};
use xack::XAck;
use xread::XReader;

pub const GROUP_NAME: &str = "micro-gateway";

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
                    if let Err(e) = events_in.send(BackendEvents::from(data.clone())) {
                        error!("send backend event {:?}", e)
                    }
                    unacked.push(xid, data);
                    info!("🏞 OK {:?}", dc)
                }
            }
        }

        unacked.ack_all(opts.xack.as_ref())
    }
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
