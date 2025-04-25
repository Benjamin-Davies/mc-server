use uuid::Uuid;

use crate::packets::serialize::{Serialize, Serializer};

#[derive(Debug)]
pub enum Packet<'a> {
    LoginDisconnect { reason: &'a str },
    LoginFinished { uuid: Uuid, username: &'a str },
}

impl Serialize for Packet<'_> {
    fn serialize(&self, s: &mut Serializer) {
        match self {
            Packet::LoginDisconnect { reason } => {
                s.serialize_varint(0x00);
                s.serialize_string(reason);
            }
            Packet::LoginFinished { uuid, username } => {
                s.serialize_varint(0x02);
                s.serialize_uuid(*uuid);
                s.serialize_string(username);
                s.serialize_prefixed_byte_array(&[]);
            }
        }
    }
}
