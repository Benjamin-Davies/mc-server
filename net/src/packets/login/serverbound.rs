use crate::packets::deserialize::{Deserialize, types};

#[derive(Debug, Deserialize)]
#[packet(state = Login)]
pub enum Packet {
    #[packet(id = 0x00)]
    Hello {
        name: types::string,
        player_uuid: types::uuid,
    },
    #[packet(id = 0x03)]
    LoginAcknowledged,
}
