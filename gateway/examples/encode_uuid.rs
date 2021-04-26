extern crate gateway;
extern crate uuid;
use gateway::compact_ids::CompactId;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        panic!("You must specify a UUID as a command line argument")
    }

    let a = args.get(1).unwrap();

    if let Ok(u) = uuid::Uuid::parse_str(a) {
        println!("{}", CompactId::encode(u).0)
    } else {
        panic!("You must specify a UUID as a command line argument")
    }
}
