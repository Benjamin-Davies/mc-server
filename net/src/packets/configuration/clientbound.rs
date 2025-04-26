use crate::{
    nbt,
    packets::serialize::{Serialize, Serializer},
};

#[derive(Debug)]
pub enum Packet<'a> {
    Disconnect {
        reason: &'a str,
    },
    FinishConfiguration,
    RegistryData {
        registry_id: &'a str,
        entries: &'a [RegistryEntry<'a>],
    },
    SelectKnownPacks {
        known_packs: &'a [KnownPack<'a>],
    },
}

#[derive(Debug)]
pub struct RegistryEntry<'a> {
    pub entry_id: &'a str,
    pub entry_data: Option<nbt::Tag>,
}

#[derive(Debug)]
pub struct KnownPack<'a> {
    pub namespace: &'a str,
    pub id: &'a str,
    pub version: &'a str,
}

impl Serialize for Packet<'_> {
    fn serialize(&self, s: &mut Serializer) {
        match self {
            &Packet::Disconnect { reason } => {
                s.serialize_varint(0x02);
                nbt::Tag::from(reason).serialize(s);
            }
            &Packet::FinishConfiguration => {
                s.serialize_varint(0x03);
            }
            &Packet::RegistryData {
                registry_id,
                entries,
            } => {
                s.serialize_varint(0x07);
                s.serialize_string(registry_id);
                s.serialize_prefixed_array(entries);
            }
            Packet::SelectKnownPacks { known_packs } => {
                s.serialize_varint(0x0E);
                s.serialize_prefixed_array(known_packs);
            }
        }
    }
}

impl Serialize for RegistryEntry<'_> {
    fn serialize(&self, s: &mut Serializer) {
        s.serialize_string(self.entry_id);
        s.serialize_prefixed_optional(&self.entry_data);
    }
}

impl Serialize for KnownPack<'_> {
    fn serialize(&self, s: &mut Serializer) {
        s.serialize_string(self.namespace);
        s.serialize_string(self.id);
        s.serialize_string(self.version);
    }
}
