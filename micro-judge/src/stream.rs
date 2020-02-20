use crate::conn_pool::Pool;
use crate::model::*;
use crate::topics;
use redis;
use redis::{Commands, Value};
use std::collections::HashMap;

const BLOCK_MSEC: u32 = 5000;

pub fn process(pool: Pool) {
    loop {
        if let Ok(make_move_cmd) = xread(&pool).and_then(|p| Ok(MakeMoveCommand::from(p))) {
            todo!("cmd {:#?}", make_move_cmd)
        } else {
            println!("timeout")
        }
    }
}

fn xread(pool: &Pool) -> Result<HashMap<String, String>, redis::RedisError> {
    let mut conn = pool.get().unwrap();
    // XREAD BLOCK 1000 STREAMS mystream 1526999626221-0

    /*println!(
        "found {:#?}",
        // TODO
        cmd.query::<Vec<HashMap<String, HashMap<String, String>>>>(&mut *conn)
    );*/

    if let Ok(s) = conn.get::<String, String>("SOME".to_string()) {
        println!("got some {}", s)
    };

    let args = &[
        "BLOCK",
        &BLOCK_MSEC.to_string(),
        "STREAMS",
        topics::MAKE_MOVE_CMD,
        "0-0",
        "0-0",
    ];
    println!("args for XREAD {:#?}", args);
    /*let mut iter: redis::Iter<String> = redis::cmd("XREAD")
        .arg(args[0])
        .arg(args[1])
        .arg(args[2])
        .arg(args[3])
        .arg(args[4])
        .arg(args[5])
        .cursor_arg(0)
        .clone()
        .iter(&mut *conn)
        .unwrap();

    for x in iter {
        println!("..{:#?}", x)
    }*/

    type T = (String, Vec<(String, Vec<String>)>);
    type U = redis::Value;
    let found = redis::cmd("XREAD")
        .arg(args[0])
        .arg(args[1])
        .arg(args[2])
        .arg(args[3])
        .arg(args[4])
        .query::<U>(&mut *conn)
        .unwrap();
    println!("found {:#?}", found);
    /*
    bulk(
        bulk(
            string-data('"bugout-make-move-cmd"'),
            bulk(
                bulk(
                    string-data('"1582060872100-0"'),
                    bulk(
                        string-data('"game_id"'),
                        string-data('"abcd"')
                    )
                ),
                bulk(
                    string-data('"1582061401598-0"'),
                    bulk(
                        string-data('"game_id"'),
                        string-data('"abce"')
                    )
                )
            )
        )
    )
    */

    if let Value::Bulk(bs) = found {
        println!("ok then {:#?}", bs);
        todo!()
    } else {
        todo!("no bulky")
    }
}
