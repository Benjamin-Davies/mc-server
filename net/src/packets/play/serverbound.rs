use crate::{
    connection::State,
    packets::deserialize::{Deserialize, Deserializer, Error, InvalidPacketIdSnafu},
};

#[derive(Debug)]
pub enum Packet {
    AcceptTeleportation {
        teleport_id: i32,
    },
    ChunkBatchReceived {
        chunks_per_tick: f32,
    },
    ClientTickEnd,
    CustomPayload {
        channel: String,
        data: Vec<u8>,
    },
    MovePlayerPosRot {
        x: f64,
        feet_y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
        flags: i8,
    },
    MovePlayerPos {
        x: f64,
        feet_y: f64,
        z: f64,
        flags: i8,
    },
}

impl<'de> Deserialize<'de> for Packet {
    fn deserialize(d: &mut Deserializer<'de>) -> Result<Self, Error> {
        match d.deserialize_varint()? {
            0x00 => Ok(Packet::AcceptTeleportation {
                teleport_id: d.deserialize_varint()?,
            }),
            0x09 => Ok(Packet::ChunkBatchReceived {
                chunks_per_tick: d.deserialize_float()?,
            }),
            0x0B => Ok(Packet::ClientTickEnd),
            0x14 => Ok(Packet::CustomPayload {
                channel: d.deserialize_string()?.to_owned(),
                data: d.take_remaining().to_owned(),
            }),
            0x1C => Ok(Packet::MovePlayerPos {
                x: d.deserialize_double()?,
                feet_y: d.deserialize_double()?,
                z: d.deserialize_double()?,
                flags: d.deserialize_byte()?,
            }),
            0x1D => Ok(Packet::MovePlayerPosRot {
                x: d.deserialize_double()?,
                feet_y: d.deserialize_double()?,
                z: d.deserialize_double()?,
                yaw: d.deserialize_float()?,
                pitch: d.deserialize_float()?,
                flags: d.deserialize_byte()?,
            }),
            packet_id => InvalidPacketIdSnafu {
                state: State::Play,
                packet_id,
            }
            .fail(),
        }
    }
}
