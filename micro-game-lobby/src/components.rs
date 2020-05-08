use crate::repo::EntryIdRepo;
use crate::stream::XReader;
pub struct Components {
    pub entry_id_repo: Box<dyn EntryIdRepo>,
    pub xreader: Box<dyn XReader>,
}
