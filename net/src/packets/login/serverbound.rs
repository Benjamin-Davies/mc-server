use uuid::Uuid;

use crate::packets::deserialize::{Deserialize, Deserializer};

#[derive(Debug)]
pub enum Packet<'a> {
    Hello { name: &'a str, player_uuid: Uuid },
    LoginAcknowledged,
}

impl<'de> Deserialize<'de> for Packet<'de> {
    fn deserialize(d: &mut Deserializer<'de>) -> anyhow::Result<Self> {
        match d.deserialize_varint()? {
            0x00 => Ok(Packet::Hello {
                name: d.deserialize_string()?,
                player_uuid: d.deserialize_uuid()?,
            }),
            0x03 => Ok(Packet::LoginAcknowledged),
            _ => Err(anyhow::anyhow!("Invalid packet type")),
        }
    }
}
