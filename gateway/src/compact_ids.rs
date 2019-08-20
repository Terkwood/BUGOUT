use harsh::{Harsh, HarshBuilder};
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

// you can pass a hash_salt option to a .env file,
// if desired.  it'll be picked up by this static
// variable
use crate::env::HASH_SALT;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CompactId(pub String);

lazy_static! {
    static ref HARSH: Harsh = HarshBuilder::new()
        .salt(HASH_SALT.to_string())
        .init()
        .unwrap();
}

impl CompactId {
    pub fn encode(uuid: Uuid) -> CompactId {
        let bytes: &[u8; 16] = uuid.as_bytes();
        let chunk_one = &bytes[0..8];
        let chunk_two = &bytes[8..16];
        let x = u64::from_be_bytes(array(chunk_one));
        let y = u64::from_be_bytes(array(chunk_two));
        CompactId(HARSH.encode(&[x, y]).unwrap())
    }

    pub fn decode(self) -> Option<Uuid> {
        HARSH.decode(self.0).map(|xy| {
            let x = xy[0];
            let y = xy[1];
            let chunk_one = u64::to_be_bytes(x);
            let chunk_two = u64::to_be_bytes(y);
            let mut bytes: [u8; 16] = [0; 16];
            let mut i = 0;
            for b in chunk_one.iter() {
                bytes[i] = *b;
                i += 1;
            }
            for b in chunk_two.iter() {
                bytes[i] = *b;
                i += 1;
            }

            Uuid::from_bytes(bytes)
        })
    }
}

fn array(s: &[u8]) -> [u8; 8] {
    let mut a: [u8; 8] = Default::default();
    a.copy_from_slice(s);
    a
}

#[cfg(test)]
mod tests {
    use super::CompactId;
    use uuid::Uuid;

    #[test]
    fn encode() {
        let u = Uuid::new_v4();
        let compact = CompactId::encode(u);

        println!("{}\t{:?}\t{}", u, compact.0, compact.0.len());

        assert_eq!(compact.0.len() > 0, true);
    }

    #[test]
    fn encode_decode() {
        let u = Uuid::new_v4();
        let compact = CompactId::encode(u);
        let cc = compact.clone();

        let d = compact.decode();

        println!("{} {}\t{:?}", u, cc.0, d);

        assert_eq!(Some(u), d);
    }

    const LOTS: u32 = 10000;

    #[test]
    fn lots_of_encode() {
        let mut lowest: usize = 1000;
        let mut highest: usize = 0;

        for _ in 0..LOTS {
            let u = Uuid::new_v4();
            let compact = CompactId::encode(u);

            let len = compact.0.len();
            assert_eq!(len > 0, true);

            if len < lowest {
                lowest = len;
            }

            if len > highest {
                highest = len;
            }
        }

        println!("highest\t{}", highest);
        println!("lowest\t{}", lowest);
    }

    #[test]
    fn lots_of_encode_decode() {
        for _ in 0..LOTS {
            let u = Uuid::new_v4();
            let compact = CompactId::encode(u);

            let d = compact.decode();

            assert_eq!(Some(u), d);
        }
    }
}
