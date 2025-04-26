use crate::packets::deserialize::{Deserialize, types};

#[derive(Debug, Deserialize)]
#[packet(state = Play)]
pub enum Packet {
    #[packet(id = 0x00)]
    AcceptTeleportation { teleport_id: types::varint },
    #[packet(id = 0x09)]
    ChunkBatchReceived { chunks_per_tick: types::float },
    #[packet(id = 0x0B)]
    ClientTickEnd,
    #[packet(id = 0x14)]
    CustomPayload {
        channel: types::string,
        #[packet(deserialize_with = d.take_remaining().to_owned())]
        data: Vec<u8>,
    },
    #[packet(id = 0x1C)]
    MovePlayerPos {
        x: types::double,
        feet_y: types::double,
        z: types::double,
        flags: types::byte,
    },
    #[packet(id = 0x1D)]
    MovePlayerPosRot {
        x: types::double,
        feet_y: types::double,
        z: types::double,
        yaw: types::float,
        pitch: types::float,
        flags: types::byte,
    },
}
