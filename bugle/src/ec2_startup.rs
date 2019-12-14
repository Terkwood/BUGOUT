use std::time::SystemTime;

pub struct StartupManager {
    pub last_startup: Option<SystemTime>,
}

impl StartupManager {
    pub fn wake_up(&mut self) {
        match self.last_startup {
            None => unimplemented!(),
            Some(_t) => unimplemented!(),
        }
    }
}

impl Default for StartupManager {
    fn default() -> StartupManager {
        StartupManager { last_startup: None }
    }
}
