use super::entry_id_repo::*;
use super::xread::XReader;
use crate::backend::events as be;
use crate::backend::events::BackendEvents;
use crate::model::{ColorsChosenEvent, HistoryProvidedEvent, MoveMadeEvent};
use color_model as color;
use crossbeam_channel::Sender;
use lobby_model as lobby;
use log::error;
use redis_streams::XReadEntryId;
use sync_model as sync;

#[derive(Clone, Debug)]
pub enum StreamData {
    BotAttached(micro_model_bot::gateway::BotAttached),
    MoveMade(micro_model_moves::MoveMade),
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
                        process_event(xid, data, &events_in, &opts)
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
    match data {
        StreamData::BotAttached(b) => {
            if let Err(e) = events_in.send(BackendEvents::BotAttached(b)) {
                error!("send err bot attached {:?}", e)
            } else if let Err(e) = opts.entry_id_repo.update(EntryIdType::BotAttached, xid) {
                error!("err tracking EID bot attached {:?}", e)
            }
        }
        StreamData::MoveMade(m) => {
            if let Err(e) = events_in.send(BackendEvents::from(StreamData::MoveMade(m))) {
                error!("send err move made {:?}", e)
            }

            if let Err(e) = opts.entry_id_repo.update(EntryIdType::MoveMade, xid) {
                error!("err tracking EID move made {:?}", e)
            }
        }
        StreamData::HistoryProvided(_) => todo!(),
        StreamData::SyncReply(_) => todo!(),
        StreamData::WaitForOpponent(_) => todo!(),
        StreamData::GameReady(_) => todo!(),
        StreamData::PrivGameRejected(_) => todo!(),
        StreamData::ColorsChosen(_) => todo!(),
    }
}

pub struct StreamOpts {
    pub entry_id_repo: Box<dyn EntryIdRepo>,
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
