use crate::packets::deserialize::{
    Deserialize,
    types::{boolean, byte, prefixed_array, string, ubyte, varint},
};

#[derive(Debug, Deserialize)]
#[packet(state = Configuration)]
pub enum Packet {
    #[packet(id = 0x00)]
    ClientInformation {
        locale: string,
        view_distance: byte,
        chat_mode: varint,
        chat_colors: boolean,
        displayed_skin_parts: ubyte,
        main_hand: varint,
        enable_text_filtering: boolean,
        allow_server_listings: boolean,
        particle_status: varint,
    },
    #[packet(id = 0x02)]
    CustomPayload {
        channel: string,
        #[packet(deserialize_with = d.take_remaining().to_owned())]
        data: Vec<u8>,
    },
    #[packet(id = 0x03)]
    FinishConfiguration,
    #[packet(id = 0x07)]
    SelectKnownPacks {
        known_packs: prefixed_array<KnownPack>,
    },
}

#[derive(Debug, Deserialize)]
pub struct KnownPack {
    pub x: string,
    pub y: string,
    pub z: string,
}
