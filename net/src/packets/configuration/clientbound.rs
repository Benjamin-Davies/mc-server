use crate::{
    nbt,
    packets::serialize::{Serialize, types},
};

#[derive(Debug, Serialize)]
pub enum Packet<'a> {
    #[packet(id = 0x02)]
    Disconnect {
        #[packet(serialize_with = s.serialize_nbt(*reason))]
        reason: &'a str,
    },
    #[packet(id = 0x03)]
    FinishConfiguration,
    #[packet(id = 0x07)]
    RegistryData {
        registry_id: types::string<'a>,
        entries: types::prefixed_array<'a, RegistryEntry<'a>>,
    },
    #[packet(id = 0x0E)]
    SelectKnownPacks {
        known_packs: types::prefixed_array<'a, KnownPack<'a>>,
    },
}

#[derive(Debug, Serialize)]
pub struct RegistryEntry<'a> {
    pub entry_id: types::string<'a>,
    pub entry_data: types::prefixed_optional<nbt::Tag>,
}

#[derive(Debug, Serialize)]
pub struct KnownPack<'a> {
    pub namespace: types::string<'a>,
    pub id: types::string<'a>,
    pub version: types::string<'a>,
}
