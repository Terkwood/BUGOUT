use crate::*;
use crossbeam_channel::{select, Receiver, Sender};
use json::*;
use log::{error, info};
use std::convert::TryFrom;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::thread;

pub mod json;

const PROGRAM: &str = "./katago";

lazy_static! {
    pub static ref ARGS: Vec<String> = vec![
        "analysis".to_string(),
        "-model".to_string(),
        env::MODEL_FILE.to_string(),
        "-config".to_string(),
        "analysis.cfg".to_string(),
        "-analysis-threads".to_string(),
        "2".to_string(),
    ];
}

pub fn start(move_computed_in: Sender<MoveComputed>, compute_move_out: Receiver<ComputeMove>) {
    let mut process = launch_child().expect("failed to start katago");

    let mut child_in = process.stdin.take().expect("no handle to stdin");
    thread::spawn(move || loop {
        select! {
                recv(compute_move_out) -> request =>
                    match request {
                        Ok(r) =>{
                            if let Ok(query) = KataGoQuery::from(&r.game_id, &r.game_state) {
                                match query.to_json() {
                                    Ok(qj) => match child_in.write(&qj) {
                                        Err(why) => panic!("couldn't write to stdin: {:?}", why),
                                        Ok(_) => info!("> requested compute for {:?}",query),
                                    },
                                    Err(e) => error!("failed query ser {:?}",e)
                                }
                            } else {
                                error!("ERR Bad coord in game state")
                            }
                        }
                        Err(_) => error!("Error receiving compute move in katago select")
                    },
        }
    });

    let mut child_out = BufReader::new(process.stdout.take().expect("no handle to stdout"));

    loop {
        let mut s = String::new();

        match child_out.read_line(&mut s) {
            Err(why) => panic!("couldn't read stdout: {:?}", why),
            Ok(_) => {
                info!("< katago respond:\n{}", s);
                let deser: Result<KataGoResponse, _> = serde_json::from_str(&s.trim());
                match deser {
                    Err(e) => error!("Deser error in katago response: {:?}\nraw: {}", e, s),
                    Ok(kgr) => {
                        if let Err(e) = move_computed_in
                            .send(MoveComputed::try_from(kgr).expect("couldnt make a movecomputed"))
                        {
                            error!("failed to send move_computed {:?}", e)
                        }
                    }
                }
            }
        }
    }
}

const PASS: &str = "PASS";
impl TryFrom<KataGoResponse> for MoveComputed {
    type Error = crate::err::KataGoParseErr;
    fn try_from(response: KataGoResponse) -> Result<Self, Self::Error> {
        let game_id = response.game_id()?;
        let player = response.player()?;
        let alpha_num_or_pass = &response.move_infos[0].r#move;

        let alphanum_coord = if alpha_num_or_pass.to_ascii_uppercase().trim() == PASS {
            None
        } else {
            let ans: Vec<char> = alpha_num_or_pass.chars().collect();
            let left: char = ans[0];
            Some(AlphaNumCoord(
                left,
                alpha_num_or_pass[1..].parse::<u16>().expect("alphanum"),
            ))
        };

        Ok(MoveComputed {
            game_id,
            player,
            alphanum_coord,
        })
    }
}

fn launch_child() -> Result<Child, std::io::Error> {
    Command::new(PROGRAM)
        .arg(&ARGS[0])
        .arg(&ARGS[1])
        .arg(&ARGS[2])
        .arg(&ARGS[3])
        .arg(&ARGS[4])
        .arg(&ARGS[5])
        .arg(&ARGS[6])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
}

#[cfg(test)]
mod tests {
    use super::*;
    use json::KataGoResponse;
    use micro_model_moves::*;
    use uuid::Uuid;
    #[test]
    fn move_computed_from_play() {
        let actual = MoveComputed::try_from(KataGoResponse {
            id: Id(format!("{}_1_WHITE", Uuid::nil().to_string())),
            turn_number: 1,
            move_infos: vec![MoveInfo {
                r#move: "B3".to_string(),
                order: 0,
            }],
        })
        .expect("fail");
        let expected = MoveComputed {
            game_id: GameId(Uuid::nil()),
            alphanum_coord: Some(AlphaNumCoord('B', 3)),
            player: Player::WHITE,
        };
        assert_eq!(actual, expected)
    }

    #[test]
    fn y_coord_not_truncated() {
        let actual = MoveComputed::try_from(KataGoResponse {
            id: Id(format!("{}_1_WHITE", Uuid::nil().to_string())),
            turn_number: 1,
            move_infos: vec![MoveInfo {
                r#move: "D10".to_string(),
                order: 0,
            }],
        })
        .expect("fail");
        let expected = MoveComputed {
            game_id: GameId(Uuid::nil()),
            alphanum_coord: Some(AlphaNumCoord('D', 10)),
            player: Player::WHITE,
        };
        assert_eq!(actual, expected)
    }

    #[test]
    fn move_computed_from_pass() {
        let actual = MoveComputed::try_from(KataGoResponse {
            id: Id(format!("{}_1_BLACK", Uuid::nil().to_string())),
            turn_number: 1,
            move_infos: vec![MoveInfo {
                r#move: "pass".to_string(),
                order: 0,
            }],
        })
        .expect("fail");
        let expected = MoveComputed {
            game_id: GameId(Uuid::nil()),
            alphanum_coord: None,
            player: Player::BLACK,
        };
        assert_eq!(actual, expected)
    }
}
