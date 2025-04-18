use crate::types::{LoginResponse, StatusResponse};

pub trait Encode {
    fn encode(&self) -> anyhow::Result<Vec<u8>>;
}

fn long(buf: &mut Vec<u8>, n: i64) {
    let bytes = n.to_be_bytes();
    buf.extend_from_slice(&bytes);
}

pub(crate) fn varint(buf: &mut Vec<u8>, n: u32) {
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

fn string(buf: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    prefixed_byte_array(buf, bytes);
}

fn prefixed_byte_array(buf: &mut Vec<u8>, data: &[u8]) {
    let len = data.len() as u32;
    varint(buf, len);
    buf.extend_from_slice(data);
}

impl Encode for StatusResponse<'_> {
    fn encode(&self) -> anyhow::Result<Vec<u8>> {
        let mut buf = Vec::new();

        match self {
            StatusResponse::Status { status } => {
                varint(&mut buf, 0x00);
                string(&mut buf, &serde_json::to_string(status)?);
            }
            StatusResponse::Pong { timestamp } => {
                varint(&mut buf, 0x01);
                long(&mut buf, *timestamp);
            }
        }

        Ok(buf)
    }
}

impl Encode for LoginResponse<'_> {
    fn encode(&self) -> anyhow::Result<Vec<u8>> {
        let mut buf = Vec::new();

        match self {
            LoginResponse::Disconnect { reason } => {
                varint(&mut buf, 0x00);
                string(&mut buf, &serde_json::to_string(reason)?);
            }
        }

        Ok(buf)
    }
}
