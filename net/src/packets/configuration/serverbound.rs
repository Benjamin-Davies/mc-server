use crate::packets::deserialize::{Deserialize, Deserializer};

#[derive(Debug)]
pub enum Packet<'a> {
    ClientInformation {
        locale: &'a str,
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
        channel: &'a str,
        data: &'a [u8],
    },
    FinishConfiguration,
    SelectKnownPacks {
        known_packs: Vec<(&'a str, &'a str, &'a str)>,
    },
}

impl<'de> Deserialize<'de> for Packet<'de> {
    fn deserialize(d: &mut Deserializer<'de>) -> anyhow::Result<Self> {
        match d.deserialize_varint()? {
            0x00 => Ok(Packet::ClientInformation {
                locale: d.deserialize_string()?,
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
                channel: d.deserialize_string()?,
                data: d.take_remaining(),
            }),
            0x03 => Ok(Packet::FinishConfiguration),
            0x07 => Ok(Packet::SelectKnownPacks {
                known_packs: d.deserialize_prefixed_array(|d| {
                    Ok((
                        d.deserialize_string()?,
                        d.deserialize_string()?,
                        d.deserialize_string()?,
                    ))
                })?,
            }),
            packet_id => anyhow::bail!("Invalid packet ID (configuration): 0x{packet_id:02x}"),
        }
    }
}
