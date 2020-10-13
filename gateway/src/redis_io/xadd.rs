use crate::backend::commands::BackendCommands as BC;
use crate::backend::commands::{
    CreateGameBackendCommand, FindPublicGameBackendCommand, JoinPrivateGameBackendCommand,
};
use crate::model::{Coord, MakeMoveCommand, ProvideHistoryCommand};
use crate::redis_io::RedisPool;
use crate::topics;
use micro_model_bot::gateway::AttachBot;

use crate::backend::commands::IntoShared;
use crossbeam_channel::{select, Receiver};
use log::error;
use r2d2_redis::redis;
use std::sync::Arc;

pub trait XAddCommands {
    fn xadd_attach_bot(&self, attach_bot: AttachBot);
    fn xadd_make_move(&self, command: MakeMoveCommand);
    fn xadd_provide_history(&self, command: ProvideHistoryCommand);
    fn xadd_join_private_game(&self, command: JoinPrivateGameBackendCommand);
    fn xadd_find_public_game(&self, command: FindPublicGameBackendCommand);
    fn xadd_create_game(&self, command: CreateGameBackendCommand);
}

pub fn start(commands_out: Receiver<BC>, cmds: &dyn XAddCommands) {
    loop {
        select! {
            recv(commands_out) -> backend_command_msg => match backend_command_msg {
                Err(e) => error!("backend command xadd {:?}",e),
                Ok(command) => match command {
                    BC::AttachBot(attach_bot) => {
                        cmds.xadd_attach_bot(attach_bot)
                    }
                    BC::MakeMove(c) => cmds.xadd_make_move(c),
                    BC::ClientHeartbeat(_) => (),
                    BC::ProvideHistory(ph) => cmds.xadd_provide_history(ph),
                    BC::ReqSync(_) => todo!(),
                    BC::JoinPrivateGame(jpg) => cmds.xadd_join_private_game(jpg),
                    BC::FindPublicGame(_) => todo!(),
                    BC::CreateGame(_) => todo!(),
                    BC::ChooseColorPref(_) => todo!(),
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
                    .arg(topics::ATTACH_BOT_TOPIC)
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
            .arg(topics::MAKE_MOVE_TOPIC)
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

    fn xadd_provide_history(&self, command: ProvideHistoryCommand) {
        self.xadd_classic(
            bincode::serialize(&command.into_shared()),
            topics::PROVIDE_HISTORY_TOPIC,
        )
    }

    fn xadd_join_private_game(&self, command: JoinPrivateGameBackendCommand) {
        self.xadd_classic(
            bincode::serialize(&command.into_shared()),
            topics::JOIN_PRIVATE_GAME_TOPIC,
        )
    }

    fn xadd_find_public_game(&self, command: FindPublicGameBackendCommand) {
        self.xadd_classic(
            bincode::serialize(&command.into_shared()),
            topics::FIND_PUBLIC_GAME_TOPIC,
        )
    }

    fn xadd_create_game(&self, command: CreateGameBackendCommand) {
        self.xadd_classic(
            bincode::serialize(&command.into_shared()),
            topics::CREATE_GAME_TOPIC,
        )
    }
}

impl RedisXAddCommands {
    pub fn create(pool: Arc<RedisPool>) -> Self {
        RedisXAddCommands { pool }
    }

    fn xadd_classic(&self, bin: Result<Vec<u8>, Box<bincode::ErrorKind>>, topic: &str) {
        match self.pool.get() {
            Err(e) => error!("xadd {}: cannot get conn {:?}", topic, e),
            Ok(mut conn) => {
                if let Ok(b) = bin {
                    let mut redis_cmd = redis::cmd("XADD");
                    redis_cmd
                        .arg(topic)
                        .arg("MAXLEN")
                        .arg("~")
                        .arg("1000")
                        .arg("*")
                        .arg("data")
                        .arg(b);
                    if let Err(e) = redis_cmd.query::<String>(&mut *conn) {
                        error!("xadd {}: redis execution err. {:?}", topic, e)
                    }
                } else {
                    error!("xadd {}: serialization error", topic)
                }
            }
        }
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
        Hist(ProvideHistoryCommand),
        Join(JoinPrivateGameBackendCommand),
        Find(FindPublicGameBackendCommand),
        Create(CreateGameBackendCommand),
    }
    impl FakeXAddCmd {
        fn sssend(&self, tr: TestResult) {
            self.st.send(tr).expect("send")
        }
    }
    impl XAddCommands for FakeXAddCmd {
        fn xadd_attach_bot(&self, attach_bot: AttachBot) {
            self.sssend(TestResult::Bot(attach_bot))
        }
        fn xadd_make_move(&self, command: MakeMoveCommand) {
            self.sssend(TestResult::Move(command))
        }

        fn xadd_provide_history(&self, command: ProvideHistoryCommand) {
            self.sssend(TestResult::Hist(command))
        }

        fn xadd_join_private_game(&self, command: JoinPrivateGameBackendCommand) {
            self.sssend(TestResult::Join(command))
        }

        fn xadd_find_public_game(&self, command: FindPublicGameBackendCommand) {
            self.sssend(TestResult::Find(command))
        }

        fn xadd_create_game(&self, command: CreateGameBackendCommand) {
            self.sssend(TestResult::Create(command))
        }
    }

    #[test]
    fn test_loop() {
        let (test_in, test_out): (Sender<TestResult>, Receiver<TestResult>) = unbounded();
        let (cmds_in, cmds_out): (Sender<BC>, Receiver<BC>) = unbounded();

        thread::spawn(move || start(cmds_out, &FakeXAddCmd { st: test_in }));

        cmds_in
            .send(BC::AttachBot(AttachBot {
                game_id: micro_model_moves::GameId(Uuid::nil()),
                board_size: Some(9),
                player: micro_model_moves::Player::WHITE,
            }))
            .expect("send test");

        cmds_in
            .send(BC::MakeMove(MakeMoveCommand {
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
