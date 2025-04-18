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
    Status { json_response: &'a str },
    Pong { timestamp: i64 },
}
