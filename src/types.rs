#![allow(dead_code)]

use serde::Serialize;
use uuid::Uuid;

use crate::nbt;

#[derive(Debug)]
pub enum HandshakeRequest<'a> {
    Handshake {
        protocol_version: i32,
        server_address: &'a str,
        server_port: u16,
        next_state: HandshakeRequestNextState,
    },
}

#[derive(Debug)]
pub enum HandshakeRequestNextState {
    Status,
    Login,
    Transfer,
}

#[derive(Debug)]
pub enum StatusRequest {
    Status,
    Ping { timestamp: i64 },
}

#[derive(Debug)]
pub enum StatusResponse<'a> {
    Status { status: Status<'a> },
    Pong { timestamp: i64 },
}

#[derive(Debug, Serialize)]
pub struct Status<'a> {
    pub version: Version<'a>,
    pub players: Players,
    pub description: TextComponent<'a>,
}

#[derive(Debug, Serialize)]
pub struct Version<'a> {
    pub name: &'a str,
    pub protocol: i32,
}

#[derive(Debug, Serialize)]
pub struct Players {
    pub max: u32,
    pub online: u32,
}

#[derive(Debug, Serialize)]
pub struct TextComponent<'a> {
    pub text: &'a str,
}

#[derive(Debug)]
pub enum LoginRequest<'a> {
    LoginStart { name: &'a str, player_uuid: Uuid },
    LoginAcknowledged,
}

#[derive(Debug)]
pub enum LoginResponse<'a> {
    Disconnect { reason: &'a str },
    LoginSuccess { uuid: Uuid, username: &'a str },
}

#[derive(Debug)]
pub enum ConfigurationRequest<'a> {
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
    PluginMessage {
        message: ServerboundPluginMessage<'a>,
    },
    AcknowledgeFinishConfiguration,
    KnownPacks {
        known_packs: Vec<(&'a str, &'a str, &'a str)>,
    },
}

#[derive(Debug)]
pub enum ConfigurationResponse<'a> {
    FinishConfiguration,
    RegistryData {
        registry_id: &'a str,
        entries: &'a [(&'a str, Option<nbt::Tag>)],
    },
    KnownPacks {
        known_packs: &'a [(&'a str, &'a str, &'a str)],
    },
}

#[derive(Debug)]
pub enum PlayRequest<'a> {
    ConfirmTeleport {
        teleport_id: i32,
    },
    TickEnd,
    PluginMessage {
        message: ServerboundPluginMessage<'a>,
    },
    SetPlayerPositionAndRotation {
        x: f64,
        feet_y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
        flags: i8,
    },
    SetPlayerPosition {
        x: f64,
        feet_y: f64,
        z: f64,
        flags: i8,
    },
    Unknown,
}

#[derive(Debug)]
pub enum PlayResponse {
    GameEvent {
        event: GameEvent,
        value: f32,
    },
    Login {
        entity_id: i32,
        enforces_secure_chat: bool,
    },
    SynchronizePlayerPosition {
        teleport_id: i32,
        x: f64,
        y: f64,
        z: f64,
        velocity_x: f64,
        velocity_y: f64,
        velocity_z: f64,
        yaw: f32,
        pitch: f32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum GameEvent {
    StartChunks = 13,
}

#[derive(Debug)]
pub enum ServerboundPluginMessage<'a> {
    Brand { brand: &'a str },
    Unknown { channel: &'a str, data: &'a [u8] },
}
