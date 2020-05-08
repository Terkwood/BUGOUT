use crate::backend_commands::BackendCommands;
use crate::model::{Coord, MakeMoveCommand};
use crate::redis_io::RedisPool;
use crate::topics::{ATTACH_BOT_TOPIC, MAKE_MOVE_TOPIC};
use micro_model_bot::gateway::AttachBot;

use crossbeam_channel::{select, Receiver};
use log::error;
use r2d2_redis::redis;
use std::sync::Arc;

pub trait XAddCommands {
    fn xadd_attach_bot(&self, attach_bot: AttachBot);
    fn xadd_make_move(&self, command: MakeMoveCommand);
}

pub fn start(commands_out: Receiver<BackendCommands>, cmds: &dyn XAddCommands) {
    loop {
        select! {
            recv(commands_out) -> backend_command_msg => match backend_command_msg {
                Err(e) => error!("backend command xadd {:?}",e),
                Ok(command) => match command {
                    BackendCommands::AttachBot(attach_bot) => {
                        cmds.xadd_attach_bot(attach_bot)
                    }
                    BackendCommands::MakeMove(c) => cmds.xadd_make_move(c),
                    BackendCommands::ClientHeartbeat(_) => (),
                    _ => error!("cannot match backend command to xadd"),
                }
            }
        }
    }
}

pub struct RedisXAddCommands {
    pub pool: Arc<RedisPool>,
}

impl XAddCommands for RedisXAddCommands {
    fn xadd_attach_bot(&self, attach_bot: AttachBot) {
        let mut conn = self.pool.get().unwrap();

        match attach_bot.serialize() {
            Err(e) => error!("attach bot ser error {:?}", e),
            Ok(bin) => {
                let mut redis_cmd = redis::cmd("XADD");
                redis_cmd
                    .arg(ATTACH_BOT_TOPIC)
                    .arg("MAXLEN")
                    .arg("~")
                    .arg("1000")
                    .arg("*")
                    .arg("data")
                    .arg(bin);
                if let Err(e) = redis_cmd.query::<String>(&mut *conn) {
                    error!("attach bot redis err {:?}", e)
                }
            }
        }
    }
    fn xadd_make_move(&self, command: MakeMoveCommand) {
        let mut conn = self.pool.get().unwrap();

        let mut redis_cmd = redis::cmd("XADD");
        redis_cmd
            .arg(MAKE_MOVE_TOPIC)
            .arg("MAXLEN")
            .arg("~")
            .arg("1000")
            .arg("*")
            .arg("game_id")
            .arg(command.game_id.to_string())
            .arg("player")
            .arg(command.player.to_string())
            .arg("req_id")
            .arg(command.req_id.to_string());
        if let Some(Coord { x, y }) = command.coord {
            redis_cmd.arg("coord_x").arg(x).arg("coord_y").arg(y);
        }
        if let Err(e) = redis_cmd.query::<String>(&mut *conn) {
            error!("make move {:?}", e)
        }
    }
}

impl RedisXAddCommands {
    pub fn create(pool: Arc<RedisPool>) -> Self {
        RedisXAddCommands { pool }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    use crossbeam_channel::{select, unbounded, Receiver, Sender};
    use std::thread;
    use uuid::Uuid;

    struct FakeXAddCmd {
        st: Sender<TestResult>,
    }
    enum TestResult {
        Bot(AttachBot),
        Move(MakeMoveCommand),
    }
    impl XAddCommands for FakeXAddCmd {
        fn xadd_attach_bot(&self, attach_bot: AttachBot) {
            self.st.send(TestResult::Bot(attach_bot)).expect("send")
        }
        fn xadd_make_move(&self, command: MakeMoveCommand) {
            self.st.send(TestResult::Move(command)).expect("send")
        }
    }

    #[test]
    fn test_loop() {
        let (test_in, test_out): (Sender<TestResult>, Receiver<TestResult>) = unbounded();
        let (cmds_in, cmds_out): (Sender<BackendCommands>, Receiver<BackendCommands>) = unbounded();

        thread::spawn(move || start(cmds_out, &FakeXAddCmd { st: test_in }));

        cmds_in
            .send(BackendCommands::AttachBot(AttachBot {
                game_id: micro_model_moves::GameId(Uuid::nil()),
                board_size: Some(9),
                player: micro_model_moves::Player::WHITE,
            }))
            .expect("send test");

        cmds_in
            .send(BackendCommands::MakeMove(MakeMoveCommand {
                game_id: Uuid::nil(),
                req_id: Uuid::nil(),
                player: Player::BLACK,
                coord: Some(Coord { x: 0, y: 0 }),
            }))
            .expect("send move test");

        select! { recv(test_out) -> msg => match msg.expect("test out 0 ") {
            TestResult::Bot(_) => assert!(true),
            _ => assert!(false)
        } }
        select! { recv(test_out) -> msg => match msg.expect("test out 1 ") {
            TestResult::Move(_) => assert!(true),
            _ => assert!(false)
        } }
    }
}
