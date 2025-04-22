use std::{mem, str};

use uuid::Uuid;

pub trait Deserialize<'de>: Sized {
    fn deserialize(d: &mut Deserializer<'de>) -> anyhow::Result<Self>;
}

pub struct Deserializer<'de> {
    bytes: &'de [u8],
}

macro_rules! deserialize_primitive {
    ($name:ident($ty:ty)) => {
        pub fn $name(&mut self) -> anyhow::Result<$ty> {
            anyhow::ensure!(self.bytes.len() >= mem::size_of::<$ty>(), "Input too short");
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

    pub fn finish(&mut self) -> anyhow::Result<()> {
        anyhow::ensure!(self.bytes.is_empty(), "Unread bytes remaining");
        Ok(())
    }

    pub fn take_remaining(&mut self) -> &'de [u8] {
        let bytes = self.bytes;
        self.bytes = &[];
        bytes
    }

    pub fn deserialize_boolean(&mut self) -> anyhow::Result<bool> {
        self.deserialize_ubyte().map(|byte| byte != 0)
    }

    pub fn deserialize_byte(&mut self) -> anyhow::Result<i8> {
        self.deserialize_ubyte().map(|byte| byte as i8)
    }

    pub fn deserialize_ubyte(&mut self) -> anyhow::Result<u8> {
        anyhow::ensure!(!self.bytes.is_empty(), "No bytes remaining");
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

    pub fn deserialize_string(&mut self) -> anyhow::Result<&'de str> {
        self.deserialize_prefixed_byte_array()
            .and_then(|bytes| str::from_utf8(bytes).map_err(anyhow::Error::from))
    }

    pub fn deserialize_uuid(&mut self) -> anyhow::Result<Uuid> {
        anyhow::ensure!(
            self.bytes.len() >= mem::size_of::<Uuid>(),
            "Input too short"
        );
        let value = Uuid::from_bytes(self.bytes[..mem::size_of::<Uuid>()].try_into().unwrap());
        self.bytes = &self.bytes[mem::size_of::<Uuid>()..];
        Ok(value)
    }

    pub fn deserialize_varint(&mut self) -> anyhow::Result<i32> {
        let mut result = 0;
        let mut shift = 0;
        loop {
            let byte = self.bytes[0];
            self.bytes = &self.bytes[1..];
            result |= ((byte & 0x7F) as i32) << shift;
            if byte & 0x80 == 0 {
                return Ok(result);
            }
            shift += 7;
            anyhow::ensure!(shift < 32, "Varint is too long");
        }
    }

    pub fn deserialize_varlong(&mut self) -> anyhow::Result<i64> {
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
            anyhow::ensure!(shift < 64, "Varlong is too long");
        }
    }

    pub fn deserialize_prefixed_array<T>(
        &mut self,
        f: impl Fn(&mut Deserializer<'de>) -> anyhow::Result<T>,
    ) -> anyhow::Result<Vec<T>> {
        let length = self.deserialize_varint()?;
        let mut result = Vec::with_capacity(length as usize);
        for _ in 0..length {
            result.push(f(self)?);
        }
        Ok(result)
    }

    pub fn deserialize_prefixed_byte_array(&mut self) -> anyhow::Result<&'de [u8]> {
        let length = self.deserialize_varint()?;
        anyhow::ensure!(self.bytes.len() >= length as usize, "Input to short");
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
