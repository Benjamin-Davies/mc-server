use crate::{
    nbt,
    packets::serialize::{Serialize, Serializer},
};

#[derive(Debug)]
pub enum Packet<'a> {
    FinishConfiguration,
    RegistryData {
        registry_id: &'a str,
        entries: &'a [(&'a str, Option<nbt::Tag>)],
    },
    SelectKnownPacks {
        known_packs: &'a [(&'a str, &'a str, &'a str)],
    },
}

impl Serialize for Packet<'_> {
    fn serialize(&self, s: &mut Serializer) -> anyhow::Result<()> {
        match self {
            Packet::FinishConfiguration => {
                s.serialize_varint(0x03)?;
            }
            Packet::RegistryData {
                registry_id,
                entries,
            } => {
                s.serialize_varint(0x07)?;
                s.serialize_string(registry_id)?;
                s.serialize_prefixed_array(entries, |s, (entry_id, entry_data)| {
                    s.serialize_string(entry_id)?;
                    s.serialize_prefixed_optional(entry_data, |s, data| s.serialize_nbt(data))?;
                    Ok(())
                })?;
            }
            Packet::SelectKnownPacks { known_packs } => {
                s.serialize_varint(0x0E)?;
                s.serialize_prefixed_array(known_packs, |s, (namespace, id, version)| {
                    s.serialize_string(namespace)?;
                    s.serialize_string(id)?;
                    s.serialize_string(version)?;
                    Ok(())
                })?;
            }
        }
        Ok(())
    }
}
