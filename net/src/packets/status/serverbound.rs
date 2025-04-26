use crate::packets::deserialize::{Deserialize, types};

#[derive(Debug, Deserialize)]
#[packet(state = Status)]
pub enum Packet {
    #[packet(id = 0x00)]
    StatusRequest,
    #[packet(id = 0x01)]
    PingRequest { timestamp: types::long },
}
