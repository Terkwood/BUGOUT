use chrono::predule::*;

/// The running status of an expensive container host
/// 
/// - Idle (since when)
/// - Booting (an optional message indicating what's going on)
/// - Awake (you may proceed to have fun)
pub enum IdleStatus {
    Idle(DateTime<Utc>),
    Booting(Option<String>),
    Awake
}