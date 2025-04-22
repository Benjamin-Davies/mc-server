use crate::packets::deserialize::{Deserialize, Deserializer};

#[derive(Debug)]
pub enum Packet {
    StatusRequest,
    PingRequest { timestamp: i64 },
}

impl<'de> Deserialize<'de> for Packet {
    fn deserialize(d: &mut Deserializer<'de>) -> anyhow::Result<Self> {
        match d.deserialize_varint()? {
            0x00 => Ok(Packet::StatusRequest),
            0x01 => Ok(Packet::PingRequest {
                timestamp: d.deserialize_long()?,
            }),
            packet_id => anyhow::bail!("Invalid packet ID (status): 0x{packet_id:02x}"),
        }
    }
}
