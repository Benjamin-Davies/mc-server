use std::str;

use anyhow::Context;

use crate::types::{HandshakeRequest, HandshakeRequestNextState, StatusRequest};

pub trait Decode<'a>: Sized {
    fn decode(bytes: &'a [u8]) -> anyhow::Result<Self>;
}

pub trait Parse {
    fn parse<'a, T: Decode<'a>>(&'a self) -> anyhow::Result<T>;
}

fn ushort(bytes: &[u8]) -> anyhow::Result<(u16, &[u8])> {
    let (buf, rest) = exact_byte_array(bytes).context("Invalid ushort")?;
    let n = u16::from_be_bytes(buf);
    Ok((n, rest))
}

fn long(bytes: &[u8]) -> anyhow::Result<(i64, &[u8])> {
    let (buf, rest) = exact_byte_array(bytes).context("Invalid long")?;
    let n = i64::from_be_bytes(buf);
    Ok((n, rest))
}

fn varint(bytes: &[u8]) -> anyhow::Result<(u32, &[u8])> {
    let mut n = 0;
    let mut shift = 0;
    let mut i = 0;

    while i < 5 {
        let byte = bytes.get(i).context("Invalid varint")?;
        n |= ((byte & 0x7F) as u32) << shift;
        shift += 7;
        i += 1;

        if byte & 0x80 == 0 {
            break;
        }
    }

    Ok((n, &bytes[i..]))
}

fn string(bytes: &[u8]) -> anyhow::Result<(&str, &[u8])> {
    let (len, rest) = varint(bytes)?;
    let s = str::from_utf8(&rest[..len as usize])?;
    Ok((s, &rest[len as usize..]))
}

fn exact_byte_array<const N: usize>(bytes: &[u8]) -> Option<([u8; N], &[u8])> {
    if bytes.len() >= N {
        let (head, tail) = bytes.split_at(N);
        Some((head.try_into().unwrap(), tail))
    } else {
        None
    }
}

impl<'a> Decode<'a> for HandshakeRequest<'a> {
    fn decode(bytes: &'a [u8]) -> anyhow::Result<Self> {
        let (packet_id, rest) = varint(bytes)?;

        match packet_id {
            0x00 => {
                let (protocol_version, rest) = varint(rest)?;
                let (server_address, rest) = string(rest)?;
                let (server_port, rest) = ushort(rest)?;
                let (next_state, _) = varint(rest)?;

                let next_state = match next_state {
                    1 => HandshakeRequestNextState::Status,
                    2 => HandshakeRequestNextState::Login,
                    3 => HandshakeRequestNextState::Transfer,
                    _ => anyhow::bail!("Invalid next state: {next_state}"),
                };

                Ok(HandshakeRequest::Handshake {
                    protocol_version,
                    server_address,
                    server_port,
                    next_state,
                })
            }
            _ => anyhow::bail!("Invalid packet ID (handshake)"),
        }
    }
}

impl<'a> Decode<'a> for StatusRequest {
    fn decode(bytes: &[u8]) -> anyhow::Result<Self> {
        let (packet_id, rest) = varint(bytes)?;

        match packet_id {
            0x00 => Ok(StatusRequest::Status),
            0x01 => {
                let (timestamp, _rest) = long(rest)?;

                Ok(StatusRequest::Ping { timestamp })
            }
            _ => anyhow::bail!("Invalid packet ID (status)"),
        }
    }
}

impl Parse for [u8] {
    fn parse<'a, T: Decode<'a>>(&'a self) -> anyhow::Result<T> {
        T::decode(self)
    }
}
