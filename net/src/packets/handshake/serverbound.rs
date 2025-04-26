use crate::packets::deserialize::{
    Deserialize,
    types::{string, ushort, varint},
};

#[derive(Debug, Deserialize)]
#[packet(state = Handshake)]
pub enum Packet {
    #[packet(id = 0)]
    Intention {
        protocol_version: varint,
        server_address: string,
        server_port: ushort,
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
