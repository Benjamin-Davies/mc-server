use uuid::Uuid;

use crate::{
    connection::State,
    packets::deserialize::{Deserialize, Deserializer, Error, InvalidPacketIdSnafu},
};

#[derive(Debug)]
pub enum Packet {
    Hello { name: String, player_uuid: Uuid },
    LoginAcknowledged,
}

impl<'de> Deserialize<'de> for Packet {
    fn deserialize(d: &mut Deserializer<'de>) -> Result<Self, Error> {
        match d.deserialize_varint()? {
            0x00 => Ok(Packet::Hello {
                name: d.deserialize_string()?.to_owned(),
                player_uuid: d.deserialize_uuid()?,
            }),
            0x03 => Ok(Packet::LoginAcknowledged),
            packet_id => InvalidPacketIdSnafu {
                state: State::Login,
                packet_id,
            }
            .fail(),
        }
    }
}
