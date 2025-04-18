use serde::Serialize;
use uuid::Uuid;

#[derive(Debug)]
pub enum HandshakeRequest<'a> {
    Handshake {
        protocol_version: u32,
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
    pub protocol: u32,
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
    #[allow(dead_code)]
    Disconnect {
        reason: &'a str,
    },
    LoginSuccess {
        uuid: Uuid,
        username: &'a str,
    },
}

#[derive(Debug)]
pub enum ConfigurationRequest<'a> {
    #[allow(dead_code)]
    ClientInformation {
        locale: &'a str,
        view_distance: i8,
        chat_mode: u32,
        chat_colors: bool,
        displayed_skin_parts: u8,
        main_hand: u32,
        enable_text_filtering: bool,
        allow_server_listings: bool,
        particle_status: u32,
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
    KnownPacks {
        known_packs: &'a [(&'a str, &'a str, &'a str)],
    },
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum ServerboundPluginMessage<'a> {
    Brand { brand: &'a str },
    Unknown { channel: &'a str, data: &'a [u8] },
}

#[derive(Debug)]
pub enum PlayResponse {
    Login {
        entity_id: i32,
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
