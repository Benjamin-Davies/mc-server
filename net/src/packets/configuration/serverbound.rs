use crate::packets::deserialize::{Deserialize, types};

#[derive(Debug, Deserialize)]
#[packet(state = Configuration)]
pub enum Packet {
    #[packet(id = 0x00)]
    ClientInformation {
        locale: types::string,
        view_distance: types::byte,
        chat_mode: types::varint,
        chat_colors: types::boolean,
        displayed_skin_parts: types::ubyte,
        main_hand: types::varint,
        enable_text_filtering: types::boolean,
        allow_server_listings: types::boolean,
        particle_status: types::varint,
    },
    #[packet(id = 0x02)]
    CustomPayload {
        channel: types::string,
        #[packet(deserialize_with = d.take_remaining().to_owned())]
        data: Vec<u8>,
    },
    #[packet(id = 0x03)]
    FinishConfiguration,
    #[packet(id = 0x07)]
    SelectKnownPacks {
        known_packs: types::prefixed_array<KnownPack>,
    },
}

#[derive(Debug, Deserialize)]
pub struct KnownPack {
    pub namespace: types::string,
    pub id: types::string,
    pub version: types::string,
}
