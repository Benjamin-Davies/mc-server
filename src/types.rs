use serde::Serialize;

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
pub enum LoginResponse<'a> {
    Disconnect { reason: TextComponent<'a> },
}
