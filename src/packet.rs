use bytes::{Bytes, BytesMut};

use crate::id::PacketId;

pub trait Packet {
    type Error;
    type IdType: PacketId;

    const ID: Self::IdType;

    fn serialize(self, buf: &mut BytesMut);
    fn deserialize(buf: &mut Bytes) -> Result<Self, Self::Error>
    where
        Self: Sized;
}
