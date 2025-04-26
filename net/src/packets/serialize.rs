use paste::paste;

#[allow(non_camel_case_types)]
pub mod types {
    pub type boolean = bool;

    pub type byte = i8;
    pub type ubyte = u8;
    pub type short = i16;
    pub type ushort = u16;
    pub type int = i32;
    pub type uint = u32;
    pub type long = i64;
    pub type ulong = u64;
    pub type float = f32;
    pub type double = f64;

    pub type string<'a> = &'a str;
    pub type uuid = ::uuid::Uuid;
    pub type varint = i32;
    pub type varlong = i64;
    pub type prefixed_optional<T> = Option<T>;
    pub type array<'a, T> = &'a [T];
    pub type byte_array<'a> = &'a [u8];
    pub type prefixed_array<'a, T> = &'a [T];
    pub type prefixed_byte_array<'a> = &'a [u8];
}

pub trait Serialize {
    fn serialize(&self, s: &mut Serializer);
}

pub struct Serializer {
    buf: Vec<u8>,
}

macro_rules! serialize_primitive {
    ($ty:ident) => {
        paste! {
            pub fn [<serialize_ $ty>](&mut self, value: types::$ty) {
                self.buf.extend(value.to_be_bytes());
            }
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

    pub fn serialize_boolean(&mut self, value: types::boolean) {
        self.buf.push(value as u8);
    }

    pub fn serialize_byte(&mut self, value: types::byte) {
        self.buf.push(value as u8);
    }

    pub fn serialize_ubyte(&mut self, value: types::ubyte) {
        self.buf.push(value);
    }

    serialize_primitive!(short);
    serialize_primitive!(ushort);
    serialize_primitive!(int);
    serialize_primitive!(uint);
    serialize_primitive!(long);
    serialize_primitive!(ulong);
    serialize_primitive!(float);
    serialize_primitive!(double);

    pub fn serialize_varint(&mut self, value: types::varint) {
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

    pub fn serialize_varlong(&mut self, value: types::varlong) {
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

    pub fn serialize_string(&mut self, value: types::string) {
        self.serialize_prefixed_byte_array(value.as_bytes())
    }

    pub fn serialize_uuid(&mut self, value: types::uuid) {
        self.buf.extend_from_slice(value.as_bytes());
    }

    pub fn serialize_prefixed_optional<T: Serialize>(
        &mut self,
        value: &types::prefixed_optional<T>,
    ) {
        self.serialize_boolean(value.is_some());
        if let Some(value) = value {
            value.serialize(self);
        }
    }

    pub fn serialize_array<T: Serialize>(&mut self, array: types::array<T>) {
        self.serialize_array_with(array, |s, item| item.serialize(s));
    }

    pub fn serialize_array_with<T>(
        &mut self,
        array: types::array<T>,
        mut f: impl FnMut(&mut Self, &T),
    ) {
        for item in array {
            f(self, item);
        }
    }

    pub fn serialize_byte_array(&mut self, array: types::byte_array) {
        self.buf.extend_from_slice(array);
    }

    pub fn serialize_prefixed_array<T: Serialize>(&mut self, array: types::prefixed_array<T>) {
        self.serialize_varint(array.len() as i32);
        self.serialize_array(array);
    }

    pub fn serialize_prefixed_array_with<T>(
        &mut self,
        array: types::prefixed_array<T>,
        f: impl FnMut(&mut Self, &T),
    ) {
        self.serialize_varint(array.len() as i32);
        self.serialize_array_with(array, f);
    }

    pub fn serialize_prefixed_byte_array(&mut self, array: types::prefixed_byte_array) {
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
