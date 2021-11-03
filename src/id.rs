use bytes::{Buf, BufMut, Bytes, BytesMut};

pub trait PacketId {
    fn put_id(&self, buf: &mut BytesMut);
    fn get_id(buf: &mut Bytes) -> Option<Self>
    where
        Self: Sized;
}

impl PacketId for u8 {
    fn put_id(&self, buf: &mut BytesMut) {
        buf.put_u8(*self);
    }

    fn get_id(buf: &mut Bytes) -> Option<Self>
    where
        Self: Sized 
    {
        Some(buf.get_u8())
    }
}