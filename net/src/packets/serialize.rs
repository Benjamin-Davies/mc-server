use uuid::Uuid;

use crate::nbt;

pub trait Serialize {
    fn serialize(&self, s: &mut Serializer);
}

pub struct Serializer {
    buf: Vec<u8>,
}

macro_rules! serialize_primitive {
    ($name:ident($ty:ty)) => {
        pub fn $name(&mut self, value: $ty) {
            self.buf.extend(value.to_be_bytes());
        }
    };
}

impl Serializer {
    pub fn new() -> Self {
        Serializer { buf: Vec::new() }
    }

    pub fn finish(self) -> Vec<u8> {
        self.buf
    }

    pub fn serialize_boolean(&mut self, value: bool) {
        self.buf.push(value as u8);
    }

    pub fn serialize_byte(&mut self, value: i8) {
        self.buf.push(value as u8);
    }

    pub fn serialize_ubyte(&mut self, value: u8) {
        self.buf.push(value);
    }

    serialize_primitive!(serialize_short(i16));
    serialize_primitive!(serialize_ushort(u16));
    serialize_primitive!(serialize_int(i32));
    serialize_primitive!(serialize_uint(u32));
    serialize_primitive!(serialize_long(i64));
    serialize_primitive!(serialize_ulong(u64));
    serialize_primitive!(serialize_float(f32));
    serialize_primitive!(serialize_double(f64));

    pub fn serialize_varint(&mut self, value: i32) {
        let mut value = value as u32;
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            self.buf.push(byte);
            if value == 0 {
                break;
            }
        }
    }

    pub fn serialize_varlong(&mut self, value: i64) {
        let mut value = value as u64;
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            self.buf.push(byte);
            if value == 0 {
                break;
            }
        }
    }

    pub fn serialize_string(&mut self, value: &str) {
        self.serialize_prefixed_byte_array(value.as_bytes())
    }

    pub fn serialize_nbt(&mut self, value: &nbt::Tag) {
        value.serialize(self);
    }

    pub fn serialize_uuid(&mut self, value: Uuid) {
        self.buf.extend_from_slice(value.as_bytes());
    }

    pub fn serialize_prefixed_optional<T>(
        &mut self,
        value: &Option<T>,
        mut f: impl FnMut(&mut Self, &T),
    ) {
        self.serialize_boolean(value.is_some());
        if let Some(value) = value {
            f(self, &value);
        }
    }

    pub fn serialize_array<T>(&mut self, array: &[T], mut f: impl FnMut(&mut Self, &T)) {
        for item in array {
            f(self, item);
        }
    }

    pub fn serialize_byte_array(&mut self, array: &[u8]) {
        self.buf.extend_from_slice(array);
    }

    pub fn serialize_prefixed_array<T>(&mut self, array: &[T], f: impl FnMut(&mut Self, &T)) {
        self.serialize_varint(array.len() as i32);
        self.serialize_array(array, f);
    }

    pub fn serialize_prefixed_byte_array(&mut self, array: &[u8]) {
        self.serialize_varint(array.len() as i32);
        self.serialize_byte_array(array);
    }
}

#[cfg(test)]
mod tests {
    use crate::packets::serialize::Serializer;

    #[test]
    fn test_serialize_int() {
        fn test(n: i32, expected: Vec<u8>) {
            let mut serializer = Serializer::new();
            serializer.serialize_int(n);
            assert_eq!(serializer.finish(), expected);
        }

        test(0, vec![0x00, 0x00, 0x00, 0x00]);
        test(1, vec![0x00, 0x00, 0x00, 0x01]);
        test(-1, vec![0xff, 0xff, 0xff, 0xff]);
        test(2147483647, vec![0x7f, 0xff, 0xff, 0xff]);
        test(-2147483648, vec![0x80, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_serialize_varint() {
        fn test(n: i32, expected: Vec<u8>) {
            let mut serializer = Serializer::new();
            serializer.serialize_varint(n);
            assert_eq!(serializer.finish(), expected);
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
    fn test_serialize_varlong() {
        fn test(n: i64, expected: Vec<u8>) {
            let mut serializer = Serializer::new();
            serializer.serialize_varlong(n);
            assert_eq!(serializer.finish(), expected);
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
