extern crate gateway;
extern crate uuid;
use gateway::compact_ids::CompactId;

const FAIL_MSG: &str = "You must specify a UUID as a command line argument";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        panic!(FAIL_MSG)
    }

    let a = args.get(1).unwrap();

    if let Ok(u) = uuid::Uuid::parse_str(a) {
        println!("{}", CompactId::encode(u).0)
    } else {
        panic!(FAIL_MSG)
    }
}
