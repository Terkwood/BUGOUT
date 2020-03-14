use crate::*;
use crossbeam_channel::{select, Receiver, Sender};
use json::*;
use micro_model_moves::*;
use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::thread;
use uuid::Uuid;

pub mod json;

const PROGRAM: &str = "./katago";
const ARGS: &[&str] = &[
    "analysis",
    "-model",
    "g170e-b20c256x2-s2430231552-d525879064.bin.gz",
    "-config",
    "analysis.cfg",
    "-analysis-threads",
    "2",
];

pub fn start(move_computed_in: Sender<MoveComputed>, compute_move_out: Receiver<ComputeMove>) {
    let mut process = launch_child().expect("failed to start katago");

    let mut child_in = process.stdin.take().expect("no handle to stdin");
    thread::spawn(move || loop {
        select! {
                recv(compute_move_out) -> request =>
                    match request {
                        Ok(r) =>{
                            if let Ok(query) = KataGoQuery::from(&r.game_id, &r.game_state) {
                                let qj = query.to_json().expect("failed query ser");
                                match child_in.write(&qj) {
                                    Err(why) => panic!("couldn't write to   stdin: {}", why.description()),
                                    Ok(_) => println!("> sent command {:?}",qj),
                                }
                            } else {
                                println!("ERR Bad coord in game state")
                            }
                        }
                        Err(_) => println!("Error receiving compute move in katago select")
                    },
        }
    });

    let mut child_out = BufReader::new(process.stdout.take().expect("no handle to stdout"));
    let mut s = String::new();

    loop {
        match child_out.read_line(&mut s) {
            Err(why) => panic!("couldn't read   stdout: {}", why.description()),
            Ok(_) => {
                print!("< katago respond:\n{}", s);

                let kgr: KataGoResponse = serde_json::from_str(&s).expect("couldnt deser response");
                if let Err(e) = move_computed_in
                    .send(MoveComputed::from(kgr).expect("couldnt make a movecomputed"))
                {
                    println!("failed to send move_computed {:?}", e)
                }
            }
        }
    }

    todo!("remove everything below");
    let game_id = GameId(Uuid::nil());
    let game_state = GameState {
        moves: vec![
            MoveMade {
                coord: Some(Coord::of(0, 0)),
                event_id: EventId::new(),
                game_id: game_id.clone(),
                reply_to: ReqId(Uuid::nil()),
                player: Player::BLACK,
                captured: vec![],
            },
            MoveMade {
                coord: Some(Coord::of(1, 1)),
                event_id: EventId::new(),
                game_id: game_id.clone(),
                reply_to: ReqId(Uuid::nil()),
                player: Player::WHITE,
                captured: vec![],
            },
        ],
        turn: 2,
        ..GameState::default()
    };
}

fn launch_child() -> Result<Child, std::io::Error> {
    Command::new(PROGRAM)
        .arg(ARGS[0])
        .arg(ARGS[1])
        .arg(ARGS[2])
        .arg(ARGS[3])
        .arg(ARGS[4])
        .arg(ARGS[5])
        .arg(ARGS[6])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
}
