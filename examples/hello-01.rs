use puppeteer::{Buf, BufMut, Bytes, BytesMut, Packet, Puppeteer};
use std::time::Instant;

#[derive(PartialEq, Debug)]
pub struct P1(String);
impl Packet for P1 {
    type Error = ();
    type IdType = u8;

    const ID: Self::IdType = 0;

    fn serialize(self, buf: &mut BytesMut) {
        buf.put_u16(self.0.len() as _);
        buf.put_slice(self.0.as_bytes());
    }

    fn deserialize(buf: &mut Bytes) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let len = buf.get_u16() as usize;
        Ok(Self(
            String::from_utf8_lossy(buf.slice(..len).as_ref()).to_string(),
        ))
    }
}

#[derive(PartialEq, Debug)]
pub struct P2(u32, f64);
impl Packet for P2 {
    type Error = ();
    type IdType = u8;

    const ID: Self::IdType = 1;

    fn serialize(self, buf: &mut BytesMut) {
        buf.put_u32(self.0);
        buf.put_f64(self.1);
    }

    fn deserialize(buf: &mut Bytes) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self(buf.get_u32(), buf.get_f64()))
    }
}

#[derive(PartialEq, Debug, Puppeteer)]
pub enum Packets {
    P1(P1),
    P2(P2)
}

fn main() {
    let mut buf = BytesMut::new();

    let start = Instant::now();
    Packets::serialize(P2(123, 3.553), &mut buf);
    println!("Serialization completed in: {:?}", Instant::now() - start);

    assert_eq!(
        Packets::from(P2(123, 3.553)),
        {
            let start = Instant::now();
            let out = Packets::deserialize(&mut buf.freeze())
                .unwrap() 
                // Result<T, IdType> 
                // * Ok(T) signals that there is a packet type for this 'PacketId'.
                .unwrap();
                // Result<T, Error>
                // * Ok(T) signals that deserialization was completed successfully.
            println!("Deserialization completed in: {:?}", Instant::now() - start);
            out
        }
    );
}

// Times
// v0.1.0 
// * Packets::serialize   = ~10.2 microseconds
// * Packets::deserialize = ~5.1  microseconds