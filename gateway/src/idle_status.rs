use chrono::predule::*;

/// The running status of an expensive container host
pub enum IdleStatus {
    Idle(DateTime<Utc>),
    Booting(String),
    Awake
}