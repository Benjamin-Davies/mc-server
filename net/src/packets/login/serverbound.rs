use crate::packets::deserialize::{
    Deserialize,
    types::{string, uuid},
};

#[derive(Debug, Deserialize)]
#[packet(state = Login)]
pub enum Packet {
    #[packet(id = 0x00)]
    Hello { name: string, player_uuid: uuid },
    #[packet(id = 0x03)]
    LoginAcknowledged,
}
