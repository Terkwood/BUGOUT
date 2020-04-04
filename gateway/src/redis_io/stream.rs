use crate::backend_events::BackendEvents;
use crossbeam_channel::Sender;
use log::warn;

pub fn process(_events_in: Sender<BackendEvents>) {
    warn!("Stream processing unimplemented")
}
