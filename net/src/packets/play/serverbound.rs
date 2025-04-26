use crate::packets::deserialize::{
    Deserialize,
    types::{byte, double, float, string, varint},
};

#[derive(Debug, Deserialize)]
#[packet(state = Play)]
pub enum Packet {
    #[packet(id = 0x00)]
    AcceptTeleportation { teleport_id: varint },
    #[packet(id = 0x09)]
    ChunkBatchReceived { chunks_per_tick: float },
    #[packet(id = 0x0B)]
    ClientTickEnd,
    #[packet(id = 0x14)]
    CustomPayload {
        channel: string,
        #[packet(deserialize_with = d.take_remaining().to_owned())]
        data: Vec<u8>,
    },
    #[packet(id = 0x1C)]
    MovePlayerPos {
        x: double,
        feet_y: double,
        z: double,
        flags: byte,
    },
    #[packet(id = 0x1D)]
    MovePlayerPosRot {
        x: double,
        feet_y: double,
        z: double,
        yaw: float,
        pitch: float,
        flags: byte,
    },
}
