use crate::packets::serialize::{Serialize, types};

#[derive(Debug, Serialize)]
pub enum Packet<'a> {
    #[packet(id = 0x00)]
    StatusResponse {
        #[packet(serialize_with = s.serialize_string(&serde_json::to_string(status).unwrap().as_str()))]
        status: Status<'a>,
    },
    #[packet(id = 0x01)]
    PongResponse { timestamp: types::long },
}

#[derive(Debug, serde::Serialize)]
pub struct Status<'a> {
    pub version: Version<'a>,
    pub players: Players,
    pub description: TextComponent,
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
pub struct TextComponent {
    pub text: String,
}
