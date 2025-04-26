use crate::packets::serialize::{Serialize, types};

#[derive(Debug, Serialize)]
pub enum Packet<'a> {
    #[packet(id = 0x00)]
    LoginDisconnect { reason: types::string<'a> },
    #[packet(id = 0x02)]
    LoginFinished {
        uuid: types::uuid,
        username: types::string<'a>,
        properties: types::prefixed_byte_array<'a>,
    },
}
