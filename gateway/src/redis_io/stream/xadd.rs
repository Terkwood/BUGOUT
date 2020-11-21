use crate::backend::commands::{
    ChooseColorPrefBackendCommand, CreateGameBackendCommand, FindPublicGameBackendCommand,
    JoinPrivateGameBackendCommand, ReqSyncBackendCommand, SessionDisconnected,
};
use crate::model::{Coord, MakeMoveCommand, ProvideHistoryCommand};
use crate::topics;
use bot_model::api::AttachBot;

use crate::backend::commands::IntoShared;
use log::error;
use std::sync::Arc;

pub trait XAddCommands {
    fn xadd_attach_bot(&self, attach_bot: AttachBot);
    fn xadd_make_move(&self, command: MakeMoveCommand);
    fn xadd_provide_history(&self, command: ProvideHistoryCommand);
    fn xadd_req_sync(&self, command: ReqSyncBackendCommand);
    fn xadd_join_private_game(&self, command: JoinPrivateGameBackendCommand);
    fn xadd_find_public_game(&self, command: FindPublicGameBackendCommand);
    fn xadd_create_game(&self, command: CreateGameBackendCommand);
    fn xadd_choose_color_pref(&self, command: ChooseColorPrefBackendCommand);
    fn xadd_session_disconnected(&self, command: SessionDisconnected);
}

pub struct RedisXAddCommands {
    pub client: Arc<redis::Client>,
}

impl XAddCommands for RedisXAddCommands {
    fn xadd_attach_bot(&self, attach_bot: AttachBot) {
        if let Ok(mut conn) = self.client.get_connection() {
            match bincode::serialize(&attach_bot) {
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
                    if let Err(e) = redis_cmd.query::<String>(&mut conn) {
                        error!("attach bot redis err {:?}", e)
                    }
                }
            }
        } else {
            error!("conn")
        }
    }
    fn xadd_make_move(&self, command: MakeMoveCommand) {
        if let Ok(mut conn) = self.client.get_connection() {
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
            if let Err(e) = redis_cmd.query::<String>(&mut conn) {
                error!("make move {:?}", e)
            }
        } else {
            error!("conn")
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

    fn xadd_req_sync(&self, command: ReqSyncBackendCommand) {
        self.xadd_classic(
            bincode::serialize(&command.into_shared()),
            topics::REQ_SYNC_TOPIC,
        )
    }

    fn xadd_choose_color_pref(&self, command: ChooseColorPrefBackendCommand) {
        self.xadd_classic(
            bincode::serialize(&command.into_shared()),
            topics::CHOOSE_COLOR_PREF_TOPIC,
        )
    }

    fn xadd_session_disconnected(&self, command: SessionDisconnected) {
        self.xadd_classic(
            bincode::serialize(&command.into_shared()),
            topics::SESSION_DISCONNECTED_TOPIC,
        )
    }
}

impl RedisXAddCommands {
    pub fn create(client: Arc<redis::Client>) -> Self {
        RedisXAddCommands { client }
    }

    fn xadd_classic(&self, bin: Result<Vec<u8>, Box<bincode::ErrorKind>>, topic: &str) {
        match self.client.get_connection() {
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
                    if let Err(e) = redis_cmd.query::<String>(&mut conn) {
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

    type BC = crate::backend::commands::BackendCommands;

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
        RSyn(ReqSyncBackendCommand),
        Join(JoinPrivateGameBackendCommand),
        Find(FindPublicGameBackendCommand),
        Create(CreateGameBackendCommand),
        ChCol(ChooseColorPrefBackendCommand),
        SessDisconn(SessionDisconnected),
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

        fn xadd_req_sync(&self, command: ReqSyncBackendCommand) {
            self.sssend(TestResult::RSyn(command))
        }

        fn xadd_choose_color_pref(&self, command: ChooseColorPrefBackendCommand) {
            self.sssend(TestResult::ChCol(command))
        }

        fn xadd_session_disconnected(&self, command: SessionDisconnected) {
            self.sssend(TestResult::SessDisconn(command))
        }
    }

    use bot_model::Bot;
    #[test]
    fn test_loop() {
        let (test_in, test_out): (Sender<TestResult>, Receiver<TestResult>) = unbounded();
        let (cmds_in, cmds_out): (Sender<BC>, Receiver<BC>) = unbounded();

        thread::spawn(move || {
            super::super::write::write_loop(cmds_out, &FakeXAddCmd { st: test_in })
        });

        cmds_in
            .send(BC::AttachBot(AttachBot {
                game_id: core_model::GameId(Uuid::nil()),
                board_size: Some(9),
                player: move_model::Player::WHITE,
                bot: Bot::KataGoOneStar,
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
