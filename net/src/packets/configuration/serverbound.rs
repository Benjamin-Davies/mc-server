use crate::{
    connection::State,
    packets::deserialize::{Deserialize, Deserializer, Error, InvalidPacketIdSnafu},
};

#[derive(Debug)]
pub enum Packet {
    ClientInformation {
        locale: String,
        view_distance: i8,
        chat_mode: i32,
        chat_colors: bool,
        displayed_skin_parts: u8,
        main_hand: i32,
        enable_text_filtering: bool,
        allow_server_listings: bool,
        particle_status: i32,
    },
    CustomPayload {
        channel: String,
        data: Vec<u8>,
    },
    FinishConfiguration,
    SelectKnownPacks {
        known_packs: Vec<(String, String, String)>,
    },
}

impl<'de> Deserialize<'de> for Packet {
    fn deserialize(d: &mut Deserializer<'de>) -> Result<Self, Error> {
        match d.deserialize_varint()? {
            0x00 => Ok(Packet::ClientInformation {
                locale: d.deserialize_string()?.to_owned(),
                view_distance: d.deserialize_byte()?,
                chat_mode: d.deserialize_varint()?,
                chat_colors: d.deserialize_boolean()?,
                displayed_skin_parts: d.deserialize_ubyte()?,
                main_hand: d.deserialize_varint()?,
                enable_text_filtering: d.deserialize_boolean()?,
                allow_server_listings: d.deserialize_boolean()?,
                particle_status: d.deserialize_varint()?,
            }),
            0x02 => Ok(Packet::CustomPayload {
                channel: d.deserialize_string()?.to_owned(),
                data: d.take_remaining().to_owned(),
            }),
            0x03 => Ok(Packet::FinishConfiguration),
            0x07 => Ok(Packet::SelectKnownPacks {
                known_packs: d.deserialize_prefixed_array(|d| {
                    Ok((
                        d.deserialize_string()?.to_owned(),
                        d.deserialize_string()?.to_owned(),
                        d.deserialize_string()?.to_owned(),
                    ))
                })?,
            }),
            // packet_id => anyhow::bail!("Invalid packet ID (configuration): 0x{packet_id:02x}"),
            packet_id => InvalidPacketIdSnafu {
                state: State::Configuration,
                packet_id,
            }
            .fail(),
        }
    }
}
