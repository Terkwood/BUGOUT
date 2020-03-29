use crate::backend_events::BackendEvents;
use crossbeam_channel::Sender;

pub fn process(_events_in: Sender<BackendEvents>) {
    todo!()
}
