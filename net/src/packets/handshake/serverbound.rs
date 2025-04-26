use crate::packets::deserialize::{Deserialize, types};

#[derive(Debug, Deserialize)]
#[packet(state = Handshake)]
pub enum Packet {
    #[packet(id = 0)]
    Intention {
        protocol_version: types::varint,
        server_address: types::string,
        server_port: types::ushort,
        next_state: NextState,
    },
}

#[derive(Debug, Deserialize)]
#[packet(state = Handshake)]
pub enum NextState {
    #[packet(id = 1)]
    Status,
    #[packet(id = 2)]
    Login,
    #[packet(id = 3)]
    Transfer,
}
