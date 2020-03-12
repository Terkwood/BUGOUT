extern crate tinybrain;
use serde_json;
use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::Duration;
use tinybrain::*;

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

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
        let command_out = &serde_json::to_string(&c).unwrap();
        match child_in.write(format!("{}\n", command_out).as_bytes()) {
            Err(why) => panic!("couldn't write to   stdin: {}", why.description()),
            Ok(_) => println!("> sent command"),
        }

        match child_out.read_line(&mut s) {
            Err(why) => panic!("couldn't read   stdout: {}", why.description()),
            Ok(_) => print!("< katago respond:\n{}", s),
        }
    }
}
