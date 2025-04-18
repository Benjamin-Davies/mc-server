use std::str;

use anyhow::Context;
use uuid::Uuid;

use crate::types::{
    ConfigurationRequest, HandshakeRequest, HandshakeRequestNextState, LoginRequest,
    ServerboundPluginMessage, StatusRequest,
};

pub trait Decode<'a>: Sized {
    fn decode(bytes: &'a [u8]) -> anyhow::Result<Self>;
}

pub trait Parse {
    fn parse<'a, T: Decode<'a>>(&'a self) -> anyhow::Result<T>;
}

fn boolean(bytes: &[u8]) -> anyhow::Result<(bool, &[u8])> {
    let (buf, rest) = exact_byte_array(bytes).context("Invalid boolean")?;
    let [n] = buf;
    Ok((n != 0, rest))
}

fn byte(bytes: &[u8]) -> anyhow::Result<(i8, &[u8])> {
    let (buf, rest) = exact_byte_array(bytes).context("Invalid byte")?;
    let [n] = buf;
    Ok((n as i8, rest))
}

fn ubyte(bytes: &[u8]) -> anyhow::Result<(u8, &[u8])> {
    let (buf, rest) = exact_byte_array(bytes).context("Invalid byte")?;
    let [n] = buf;
    Ok((n as u8, rest))
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

fn uuid(bytes: &[u8]) -> anyhow::Result<(Uuid, &[u8])> {
    let (uuid, rest) = exact_byte_array(bytes).context("Invalid UUID")?;
    let uuid = Uuid::from_bytes(uuid);
    Ok((uuid, rest))
}

fn prefixed_array<'a, T>(
    bytes: &'a [u8],
    mut f: impl FnMut(&'a [u8]) -> anyhow::Result<(T, &'a [u8])>,
) -> anyhow::Result<(Vec<T>, &'a [u8])> {
    let (len, mut rest) = varint(bytes)?;
    let mut result = Vec::with_capacity(len as usize);

    for _ in 0..len {
        let (item, new_rest) = f(rest)?;
        result.push(item);
        rest = new_rest;
    }

    Ok((result, rest))
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
            _ => anyhow::bail!("Invalid packet ID (handshake): {packet_id}"),
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
            _ => anyhow::bail!("Invalid packet ID (status): {packet_id}"),
        }
    }
}

impl<'a> Decode<'a> for LoginRequest<'a> {
    fn decode(bytes: &'a [u8]) -> anyhow::Result<Self> {
        let (packet_id, rest) = varint(bytes)?;

        match packet_id {
            0x00 => {
                let (name, rest) = string(rest)?;
                let (player_uuid, _rest) = uuid(rest)?;

                Ok(LoginRequest::LoginStart { name, player_uuid })
            }
            0x03 => Ok(LoginRequest::LoginAcknowledged),
            _ => anyhow::bail!("Invalid packet ID (login): {packet_id}"),
        }
    }
}

impl<'a> Decode<'a> for ConfigurationRequest<'a> {
    fn decode(bytes: &'a [u8]) -> anyhow::Result<Self> {
        let (packet_id, rest) = varint(bytes)?;

        match packet_id {
            0x00 => {
                let (locale, rest) = string(rest)?;
                let (view_distance, rest) = byte(rest)?;
                let (chat_mode, rest) = varint(rest)?;
                let (chat_colors, rest) = boolean(rest)?;
                let (displayed_skin_parts, rest) = ubyte(rest)?;
                let (main_hand, rest) = varint(rest)?;
                let (enable_text_filtering, rest) = boolean(rest)?;
                let (allow_server_listings, rest) = boolean(rest)?;
                let (particle_status, _rest) = varint(rest)?;

                Ok(ConfigurationRequest::ClientInformation {
                    locale,
                    view_distance,
                    chat_mode,
                    chat_colors,
                    displayed_skin_parts,
                    main_hand,
                    enable_text_filtering,
                    allow_server_listings,
                    particle_status,
                })
            }
            0x02 => {
                let (channel, rest) = string(rest)?;
                let message = match channel {
                    "minecraft:brand" => {
                        let (brand, _rest) = string(rest)?;
                        ServerboundPluginMessage::Brand { brand }
                    }
                    _ => ServerboundPluginMessage::Unknown {
                        channel,
                        data: rest,
                    },
                };

                Ok(ConfigurationRequest::PluginMessage { message })
            }
            0x03 => Ok(ConfigurationRequest::AcknowledgeFinishConfiguration),
            0x07 => {
                let (known_packs, _rest) = prefixed_array(rest, |rest| {
                    let (namespace, rest) = string(rest)?;
                    let (id, rest) = string(rest)?;
                    let (version, rest) = string(rest)?;

                    Ok(((namespace, id, version), rest))
                })?;

                Ok(ConfigurationRequest::KnownPacks { known_packs })
            }
            _ => anyhow::bail!("Invalid packet ID (configuration): {packet_id}"),
        }
    }
}

impl Parse for [u8] {
    fn parse<'a, T: Decode<'a>>(&'a self) -> anyhow::Result<T> {
        T::decode(self)
    }
}
