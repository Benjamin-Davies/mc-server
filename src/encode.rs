use net::packets::serialize::{Serialize, Serializer};

pub trait Encode {
    fn encode(&self) -> anyhow::Result<Vec<u8>>;
}

fn varint(buf: &mut Vec<u8>, n: i32) {
    let mut n = n;
    loop {
        let mut byte = (n & 0x7F) as u8;
        n >>= 7;
        if n != 0 {
            byte |= 0x80;
        }
        buf.push(byte);
        if n == 0 {
            break;
        }
    }
}

pub(crate) fn prefixed_byte_array(buf: &mut Vec<u8>, data: &[u8]) {
    let len = data.len() as i32;
    varint(buf, len);
    buf.extend_from_slice(data);
}

impl<S: Serialize> Encode for S {
    fn encode(&self) -> anyhow::Result<Vec<u8>> {
        let mut s = Serializer::new();
        self.serialize(&mut s)?;
        Ok(s.finish())
    }
}
