use crate::{
    connection::State,
    packets::deserialize::{
        Deserialize, Deserializer, Error, InvalidEnumVariantSnafu, InvalidPacketIdSnafu,
    },
};

#[derive(Debug)]
pub enum Packet {
    Intention {
        protocol_version: i32,
        server_address: String,
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

impl<'de> Deserialize<'de> for Packet {
    fn deserialize(d: &mut Deserializer<'de>) -> Result<Self, Error> {
        match d.deserialize_varint()? {
            0x00 => Ok(Packet::Intention {
                protocol_version: d.deserialize_varint()?,
                server_address: d.deserialize_string()?.to_owned(),
                server_port: d.deserialize_ushort()?,
                next_state: match d.deserialize_varint()? {
                    1 => NextState::Status,
                    2 => NextState::Login,
                    3 => NextState::Transfer,
                    value => InvalidEnumVariantSnafu {
                        enum_name: "Next state",
                        value,
                    }
                    .fail()?,
                },
            }),
            packet_id => InvalidPacketIdSnafu {
                state: State::Handshake,
                packet_id,
            }
            .fail(),
        }
    }
}
