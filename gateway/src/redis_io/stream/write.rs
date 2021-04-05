use super::xadd::XAddCommands;
use crate::backend::commands::BackendCommands as BC;
use crossbeam_channel::{select, Receiver};
use log::{error, info};

pub fn write_loop(commands_out: Receiver<BC>, cmds: &dyn XAddCommands) {
    loop {
        select! {
            recv(commands_out) -> backend_command_msg => match backend_command_msg {
                Err(e) => error!("backend command xadd {:?}",e),
                Ok(command) => {
                    match &command {
                        BC::ClientHeartbeat(_) => (), // no one cares
                        _ => info!("Send command: {:?}", &command)
                    }
                    match command {
                        BC::AttachBot(attach_bot) => cmds.xadd_attach_bot(attach_bot),
                        BC::MakeMove(c) => cmds.xadd_make_move(c),
                        BC::ClientHeartbeat(_) => (),
                        BC::ProvideHistory(ph) => cmds.xadd_provide_history(ph),
                        BC::ReqSync(rs) => cmds.xadd_req_sync(rs),
                        BC::JoinPrivateGame(jpg) => cmds.xadd_join_private_game(jpg),
                        BC::FindPublicGame(fpg) => cmds.xadd_find_public_game(fpg),
                        BC::CreateGame(cg) => cmds.xadd_create_game(cg),
                        BC::ChooseColorPref(cp) => cmds.xadd_choose_color_pref(cp),
                        BC::SessionDisconnected(sd) => cmds.xadd_session_disconnected(sd),
                        BC::UndoMove(ud) => cmds.xadd_undo_move(ud),
                        _ => error!("cannot match backend command to xadd"),
                    }
                }
            }
        }
    }
}
