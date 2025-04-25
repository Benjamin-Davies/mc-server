use std::{mem, str};

use snafu::prelude::*;
use uuid::Uuid;

use crate::connection::State;

pub trait Deserialize<'de>: Sized {
    fn deserialize(d: &mut Deserializer<'de>) -> Result<Self, Error>;
}

pub struct Deserializer<'de> {
    bytes: &'de [u8],
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Unexpected end of packet"))]
    EndOfPacket,
    #[snafu(display("Unread bytes remaining"))]
    BytesRemaining,
    #[snafu(display("Varint is too long"))]
    VarintTooLong,
    #[snafu(display("Varlong is too long"))]
    VarlongTooLong,
    #[snafu(
        visibility(pub),
        display("Invalid packet id ({state:?}): 0x{packet_id:02X}")
    )]
    InvalidPacketId { state: State, packet_id: i32 },
    #[snafu(
        visibility(pub),
        display("Invalid enum variant ({enum_name:?}): {value}")
    )]
    InvalidEnumVariant { enum_name: &'static str, value: i32 },
    #[snafu(transparent)]
    Utf8Error { source: std::str::Utf8Error },
}

macro_rules! deserialize_primitive {
    ($name:ident($ty:ty)) => {
        pub fn $name(&mut self) -> Result<$ty, Error> {
            ensure!(self.bytes.len() >= mem::size_of::<$ty>(), EndOfPacketSnafu);
            let value =
                <$ty>::from_be_bytes(self.bytes[..mem::size_of::<$ty>()].try_into().unwrap());
            self.bytes = &self.bytes[mem::size_of::<$ty>()..];
            Ok(value)
        }
    };
}

impl<'de> Deserializer<'de> {
    pub fn new(bytes: &'de [u8]) -> Self {
        Deserializer { bytes }
    }

    pub fn finish(&mut self) -> Result<(), Error> {
        ensure!(self.bytes.is_empty(), BytesRemainingSnafu);
        Ok(())
    }

    #[must_use]
    pub fn take_remaining(&mut self) -> &'de [u8] {
        let bytes = self.bytes;
        self.bytes = &[];
        bytes
    }

    pub fn deserialize_boolean(&mut self) -> Result<bool, Error> {
        self.deserialize_ubyte().map(|byte| byte != 0)
    }

    pub fn deserialize_byte(&mut self) -> Result<i8, Error> {
        self.deserialize_ubyte().map(|byte| byte as i8)
    }

    pub fn deserialize_ubyte(&mut self) -> Result<u8, Error> {
        ensure!(!self.bytes.is_empty(), EndOfPacketSnafu);
        let byte = self.bytes[0];
        self.bytes = &self.bytes[1..];
        Ok(byte)
    }

    deserialize_primitive!(deserialize_short(i16));
    deserialize_primitive!(deserialize_ushort(u16));
    deserialize_primitive!(deserialize_int(i32));
    deserialize_primitive!(deserialize_uint(u32));
    deserialize_primitive!(deserialize_long(i64));
    deserialize_primitive!(deserialize_ulong(u64));
    deserialize_primitive!(deserialize_float(f32));
    deserialize_primitive!(deserialize_double(f64));

    pub fn deserialize_string(&mut self) -> Result<&'de str, Error> {
        self.deserialize_prefixed_byte_array()
            .and_then(|bytes| str::from_utf8(bytes).map_err(Error::from))
    }

    pub fn deserialize_uuid(&mut self) -> Result<Uuid, Error> {
        ensure!(self.bytes.len() >= mem::size_of::<Uuid>(), EndOfPacketSnafu);
        let value = Uuid::from_bytes(self.bytes[..mem::size_of::<Uuid>()].try_into().unwrap());
        self.bytes = &self.bytes[mem::size_of::<Uuid>()..];
        Ok(value)
    }

    pub fn deserialize_varint(&mut self) -> Result<i32, Error> {
        let mut result = 0;
        let mut shift = 0;
        loop {
            ensure!(!self.bytes.is_empty(), EndOfPacketSnafu);
            let byte = self.bytes[0];
            self.bytes = &self.bytes[1..];
            result |= ((byte & 0x7F) as i32) << shift;
            if byte & 0x80 == 0 {
                return Ok(result);
            }
            shift += 7;
            ensure!(shift < 32, VarintTooLongSnafu);
        }
    }

    pub fn deserialize_varlong(&mut self) -> Result<i64, Error> {
        let mut result = 0;
        let mut shift = 0;
        loop {
            let byte = self.bytes[0];
            self.bytes = &self.bytes[1..];
            result |= ((byte & 0x7F) as i64) << shift;
            if byte & 0x80 == 0 {
                return Ok(result);
            }
            shift += 7;
            ensure!(shift < 64, VarlongTooLongSnafu);
        }
    }

    pub fn deserialize_prefixed_array<T>(
        &mut self,
        f: impl Fn(&mut Deserializer<'de>) -> Result<T, Error>,
    ) -> Result<Vec<T>, Error> {
        let length = self.deserialize_varint()?;
        let mut result = Vec::with_capacity(length as usize);
        for _ in 0..length {
            result.push(f(self)?);
        }
        Ok(result)
    }

    pub fn deserialize_prefixed_byte_array(&mut self) -> Result<&'de [u8], Error> {
        let length = self.deserialize_varint()?;
        ensure!(self.bytes.len() >= length as usize, EndOfPacketSnafu);
        let bytes = &self.bytes[..length as usize];
        self.bytes = &self.bytes[length as usize..];
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use crate::packets::deserialize::Deserializer;

    #[test]
    fn test_deserialize_int() {
        fn test(n: i32, expected: Vec<u8>) {
            let mut deserializer = Deserializer::new(&expected);
            let result = deserializer.deserialize_int().unwrap();
            deserializer.finish().unwrap();
            assert_eq!(result, n);
        }

        test(0, vec![0x00, 0x00, 0x00, 0x00]);
        test(1, vec![0x00, 0x00, 0x00, 0x01]);
        test(-1, vec![0xff, 0xff, 0xff, 0xff]);
        test(2147483647, vec![0x7f, 0xff, 0xff, 0xff]);
        test(-2147483648, vec![0x80, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_deserialize_varint() {
        fn test(n: i32, expected: Vec<u8>) {
            let mut deserializer = Deserializer::new(&expected);
            let result = deserializer.deserialize_varint().unwrap();
            deserializer.finish().unwrap();
            assert_eq!(result, n);
        }

        test(0, vec![0x00]);
        test(1, vec![0x01]);
        test(2, vec![0x02]);
        test(127, vec![0x7f]);
        test(128, vec![0x80, 0x01]);
        test(255, vec![0xff, 0x01]);
        test(25565, vec![0xdd, 0xc7, 0x01]);
        test(2097151, vec![0xff, 0xff, 0x7f]);
        test(2147483647, vec![0xff, 0xff, 0xff, 0xff, 0x07]);
        test(-1, vec![0xff, 0xff, 0xff, 0xff, 0x0f]);
        test(-2147483648, vec![0x80, 0x80, 0x80, 0x80, 0x08]);
    }

    #[test]
    fn test_deserialize_varlong() {
        fn test(n: i64, expected: Vec<u8>) {
            let mut deserializer = Deserializer::new(&expected);
            let result = deserializer.deserialize_varlong().unwrap();
            deserializer.finish().unwrap();
            assert_eq!(result, n);
        }

        test(0, vec![0x00]);
        test(1, vec![0x01]);
        test(2, vec![0x02]);
        test(127, vec![0x7f]);
        test(128, vec![0x80, 0x01]);
        test(255, vec![0xff, 0x01]);
        test(2147483647, vec![0xff, 0xff, 0xff, 0xff, 0x07]);
        test(
            9223372036854775807,
            vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f],
        );
        test(
            -1,
            vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01],
        );
        test(
            -2147483648,
            vec![0x80, 0x80, 0x80, 0x80, 0xf8, 0xff, 0xff, 0xff, 0xff, 0x01],
        );
        test(
            -9223372036854775808,
            vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        );
    }
}
