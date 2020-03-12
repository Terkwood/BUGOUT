extern crate tinybrain;
use serde_json;
use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::Duration;
use tinybrain::*;

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

static COMMAND: &'static str =
"{\"id\":\"big_08\",\"initialStones\":[],\"moves\":[   [\"B\",\"D3\"],    [\"W\",\"Q4\"],   [\"B\",\"Q10\"],    [\"W\",\"Q16\"],   [\"B\",\"K16\"],   [\"W\",\"D17\"],   [\"B\",\"C7\"], [\"W\",\"C14\"],   [\"B\",\"E13\"], [\"W\",\"D4\"],   [\"B\",\"C4\"], [\"W\",\"E3\"],   [\"B\",\"D2\"], [\"W\",\"C5\"],   [\"B\",\"B5\"], [\"W\",\"B4\"],   [\"B\",\"C3\"]], \"rules\":\"tromp-taylor\",\"komi\":7.5,\"boardXSize\":19,\"boardYSize\":19}\n";

fn main() {
    println!("🔢 {:<8} {}", NAME, VERSION);

    let mut process = match Command::new("./katago")
        .arg("analysis")
        .arg("-model")
        .arg("g170e-b20c256x2-s2430231552-d525879064.bin.gz")
        .arg("-config")
        .arg("analysis.cfg")
        .arg("-analysis-threads")
        .arg("2")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Err(why) => panic!("couldn't spawn katago: {}", why.description()),
        Ok(process) => process,
    };

    let child_in = process.stdin.as_mut().unwrap();
    let mut child_out = BufReader::new(process.stdout.as_mut().unwrap());
    let mut s = String::new();

    std::thread::sleep(Duration::from_secs(5));

    let commands = vec![KataGoQuery {
        id: Id("foo".to_string()),
        ..KataGoQuery::default()
    }];
    for c in commands {
        match child_in.write(&serde_json::to_vec(&c).unwrap()) {
            Err(why) => panic!("couldn't write to   stdin: {}", why.description()),
            Ok(_) => println!("> sent command"),
        }

        match child_out.read_line(&mut s) {
            Err(why) => panic!("couldn't read   stdout: {}", why.description()),
            Ok(_) => print!("< katago respond:\n{}", s),
        }
    }
}
