extern crate tinybrain;
use std::error::Error;
use std::io::prelude::*;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

static COMMAND: &'static str =
"{\"id\":\"big_08\",\"initialStones\":[],\"moves\":[   [\"B\",\"D3\"],    [\"W\",\"Q4\"],   [\"B\",\"Q10\"],    [\"W\",\"Q16\"],   [\"B\",\"K16\"],   [\"W\",\"D17\"],   [\"B\",\"C7\"], [\"W\",\"C14\"],   [\"B\",\"E13\"], [\"W\",\"D4\"],   [\"B\",\"C4\"], [\"W\",\"E3\"],   [\"B\",\"D2\"], [\"W\",\"C5\"],   [\"B\",\"B5\"], [\"W\",\"B4\"],   [\"B\",\"C3\"]], \"rules\":\"tromp-taylor\",\"komi\":7.5,\"boardXSize\":19,\"boardYSize\":19}";

fn main() {
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

    loop {
        match child_in.write(COMMAND.as_bytes()) {
            Err(why) => panic!("couldn't write to   stdin: {}", why.description()),
            Ok(_) => println!("sent command"),
        }

        match child_out.read_line(&mut s) {
            Err(why) => panic!("couldn't read   stdout: {}", why.description()),
            Ok(_) => print!("  responded with:\n{}", s),
        }

        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}
