use crate::packets::serialize::{Serialize, Serializer};

#[derive(Debug)]
pub enum Packet<'a> {
    StatusResponse { status: Status<'a> },
    PongResponse { timestamp: i64 },
}

#[derive(Debug, serde::Serialize)]
pub struct Status<'a> {
    pub version: Version<'a>,
    pub players: Players,
    pub description: TextComponent<'a>,
}

#[derive(Debug, serde::Serialize)]
pub struct Version<'a> {
    pub name: &'a str,
    pub protocol: i32,
}

#[derive(Debug, serde::Serialize)]
pub struct Players {
    pub max: u32,
    pub online: u32,
}

#[derive(Debug, serde::Serialize)]
pub struct TextComponent<'a> {
    pub text: &'a str,
}

impl Serialize for Packet<'_> {
    fn serialize(&self, s: &mut Serializer) {
        match self {
            Packet::StatusResponse { status } => {
                s.serialize_varint(0x00);
                s.serialize_string(&serde_json::to_string(status).unwrap());
            }
            Packet::PongResponse { timestamp } => {
                s.serialize_varint(0x01);
                s.serialize_long(*timestamp);
            }
        }
    }
}
