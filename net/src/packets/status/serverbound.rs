use crate::{
    connection::State,
    packets::deserialize::{Deserialize, Deserializer, Error, InvalidPacketIdSnafu},
};

#[derive(Debug)]
pub enum Packet {
    StatusRequest,
    PingRequest { timestamp: i64 },
}

impl<'de> Deserialize<'de> for Packet {
    fn deserialize(d: &mut Deserializer<'de>) -> Result<Self, Error> {
        match d.deserialize_varint()? {
            0x00 => Ok(Packet::StatusRequest),
            0x01 => Ok(Packet::PingRequest {
                timestamp: d.deserialize_long()?,
            }),
            packet_id => InvalidPacketIdSnafu {
                state: State::Status,
                packet_id,
            }
            .fail(),
        }
    }
}
