use crate::packets::deserialize::{Deserialize, Deserializer};

#[derive(Debug)]
pub enum Packet<'a> {
    Intention {
        protocol_version: i32,
        server_address: &'a str,
        server_port: u16,
        next_state: NextState,
    },
}

#[derive(Debug)]
pub enum NextState {
    Status,
    Login,
    Transfer,
}

impl<'de> Deserialize<'de> for Packet<'de> {
    fn deserialize(d: &mut Deserializer<'de>) -> anyhow::Result<Self> {
        match d.deserialize_varint()? {
            0x00 => Ok(Packet::Intention {
                protocol_version: d.deserialize_varint()?,
                server_address: d.deserialize_string()?,
                server_port: d.deserialize_ushort()?,
                next_state: match d.deserialize_varint()? {
                    1 => NextState::Status,
                    2 => NextState::Login,
                    3 => NextState::Transfer,
                    state => anyhow::bail!("Invalid next state: 0x{state:02x}"),
                },
            }),
            packet_id => anyhow::bail!("Invalid packet ID (handshake): 0x{packet_id:02x}"),
        }
    }
}
