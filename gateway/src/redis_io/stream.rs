use super::entry_id_repo::*;
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
        match opts.entry_id_repo.fetch_all() {
            Err(e) => error!("cannot fetch entry id repo {:?}", e),
            Ok(entry_ids) => match opts.xreader.xread_sorted(entry_ids) {
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
            },
        }
    }
}

fn process_event(
    xid: XReadEntryId,
    data: StreamData,
    events_in: &Sender<BackendEvents>,
    opts: &StreamOpts,
) {
    let xid_type = EntryIdType::from(&data);
    if let Err(e) = events_in.send(BackendEvents::from(data)) {
        error!("send backend event {:?}", e)
    } else if let Err(e) = opts.entry_id_repo.update(xid_type, xid) {
        error!("err tracking XID {:?}", e)
    }
}

pub struct StreamOpts {
    pub entry_id_repo: Box<dyn EntryIdRepo>,
    pub xreader: Box<dyn XReader>,
}

impl From<&StreamData> for EntryIdType {
    fn from(data: &StreamData) -> Self {
        match data {
            StreamData::BotAttached(_) => EntryIdType::BotAttached,
            StreamData::MoveMade(_) => EntryIdType::MoveMade,
            StreamData::HistoryProvided(_) => EntryIdType::HistProv,
            StreamData::SyncReply(_) => EntryIdType::SyncReply,
            StreamData::WaitForOpponent(_) => EntryIdType::WaitOpponent,
            StreamData::GameReady(_) => EntryIdType::GameReady,
            StreamData::PrivGameRejected(_) => EntryIdType::PrivGameReject,
            StreamData::ColorsChosen(_) => EntryIdType::ColorsChosen,
        }
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
