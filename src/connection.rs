use std::io::{Read, Write};

use crate::encode::{self, Encode};

pub fn read_packet(stream: &mut impl Read) -> anyhow::Result<Vec<u8>> {
    let len = read_varint(stream)?;

    let mut buf = vec![0; len as usize];
    stream.read_exact(&mut buf)?;

    Ok(buf)
}

pub fn write_packet(stream: &mut impl Write, packet: impl Encode) -> anyhow::Result<()> {
    let bytes = packet.encode();

    let mut buf = Vec::new();
    encode::varint(&mut buf, bytes.len() as u32);
    buf.extend_from_slice(&bytes);

    stream.write_all(&buf)?;

    Ok(())
}

fn read_varint(stream: &mut impl Read) -> anyhow::Result<u32> {
    let mut n = 0;
    let mut shift = 0;

    for _ in 0..5 {
        let mut buf = [0; 1];
        stream.read_exact(&mut buf)?;

        let [byte] = buf;
        n |= ((byte & 0x7F) as u32) << shift;
        shift += 7;

        if byte & 0x80 == 0 {
            break;
        }
    }

    Ok(n)
}
