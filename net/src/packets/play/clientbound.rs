use uuid::Uuid;

use crate::{
    nbt,
    packets::serialize::{Serialize, Serializer},
};

#[derive(Debug)]
pub enum Packet {
    AddEntity {
        entity_id: i32,
        entity_uuid: Uuid,
        entity_type: i32,
        x: f64,
        y: f64,
        z: f64,
        pitch: u8,
        yaw: u8,
        head_yaw: u8,
        data: i32,
        velocity_x: i16,
        velocity_y: i16,
        velocity_z: i16,
    },
    ChunkBatchFinished {
        batch_size: i32,
    },
    ChunkBatchStart,
    GameEvent {
        event: GameEvent,
        value: f32,
    },
    KeepAlive {
        keep_alive_id: i64,
    },
    LevelChunkWithLight {
        chunk_x: i32,
        chunk_z: i32,
        data: ChunkData,
        light: LightData,
    },
    Login {
        entity_id: i32,
        game_mode: u8,
        enforces_secure_chat: bool,
        is_flat: bool,
    },
    PlayerPosition {
        teleport_id: i32,
        x: f64,
        y: f64,
        z: f64,
        velocity_x: f64,
        velocity_y: f64,
        velocity_z: f64,
        yaw: f32,
        pitch: f32,
    },
    SetChunkCacheCenter {
        chunk_x: i32,
        chunk_z: i32,
    },
}

#[derive(Debug)]
pub struct ChunkData {
    pub heightmaps: nbt::Tag,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct LightData {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum GameEvent {
    StartChunks = 13,
}

impl Serialize for Packet {
    fn serialize(&self, s: &mut Serializer) -> anyhow::Result<()> {
        match self {
            Self::AddEntity {
                entity_id,
                entity_uuid,
                entity_type,
                x,
                y,
                z,
                pitch,
                yaw,
                head_yaw,
                data,
                velocity_x,
                velocity_y,
                velocity_z,
            } => {
                s.serialize_varint(0x01)?;
                s.serialize_varint(*entity_id)?;
                s.serialize_uuid(*entity_uuid)?;
                s.serialize_varint(*entity_type)?;
                s.serialize_double(*x)?;
                s.serialize_double(*y)?;
                s.serialize_double(*z)?;
                s.serialize_ubyte(*pitch)?;
                s.serialize_ubyte(*yaw)?;
                s.serialize_ubyte(*head_yaw)?;
                s.serialize_varint(*data)?;
                s.serialize_short(*velocity_x)?;
                s.serialize_short(*velocity_y)?;
                s.serialize_short(*velocity_z)?;
            }
            Self::ChunkBatchFinished { batch_size } => {
                s.serialize_varint(0x0C)?;
                s.serialize_varint(*batch_size)?;
            }
            Self::ChunkBatchStart => {
                s.serialize_varint(0x0D)?;
            }
            Self::GameEvent { event, value } => {
                s.serialize_varint(0x23)?;
                s.serialize_ubyte(*event as u8)?;
                s.serialize_float(*value)?;
            }
            Self::KeepAlive { keep_alive_id } => {
                s.serialize_varint(0x27)?;
                s.serialize_long(*keep_alive_id)?;
            }
            Self::LevelChunkWithLight {
                chunk_x,
                chunk_z,
                data,
                light,
            } => {
                s.serialize_varint(0x28)?;
                s.serialize_int(*chunk_x)?;
                s.serialize_int(*chunk_z)?;
                data.serialize(s)?;
                light.serialize(s)?;
            }
            Self::Login {
                entity_id,
                game_mode,
                is_flat,
                enforces_secure_chat,
            } => {
                s.serialize_varint(0x2C)?;
                s.serialize_int(*entity_id)?;
                s.serialize_boolean(false)?;
                s.serialize_prefixed_array(&["overworld"], |s, item| s.serialize_string(item))?;
                s.serialize_varint(1)?;
                s.serialize_varint(8)?;
                s.serialize_varint(8)?;
                s.serialize_boolean(false)?;
                s.serialize_boolean(false)?;
                s.serialize_boolean(false)?;
                s.serialize_varint(0)?;
                s.serialize_string("overworld")?;
                s.serialize_long(0)?;
                s.serialize_ubyte(*game_mode)?;
                s.serialize_byte(-1)?;
                s.serialize_boolean(false)?;
                s.serialize_boolean(*is_flat)?;
                s.serialize_boolean(false)?;
                s.serialize_varint(0)?;
                s.serialize_varint(0)?;
                s.serialize_boolean(*enforces_secure_chat)?;
            }
            Self::PlayerPosition {
                teleport_id,
                x,
                y,
                z,
                velocity_x,
                velocity_y,
                velocity_z,
                yaw,
                pitch,
            } => {
                s.serialize_varint(0x42)?;
                s.serialize_varint(*teleport_id)?;
                s.serialize_double(*x)?;
                s.serialize_double(*y)?;
                s.serialize_double(*z)?;
                s.serialize_double(*velocity_x)?;
                s.serialize_double(*velocity_y)?;
                s.serialize_double(*velocity_z)?;
                s.serialize_float(*yaw)?;
                s.serialize_float(*pitch)?;
                s.serialize_int(0)?;
            }
            Packet::SetChunkCacheCenter { chunk_x, chunk_z } => {
                s.serialize_varint(0x58)?;
                s.serialize_varint(*chunk_x)?;
                s.serialize_varint(*chunk_z)?;
            }
        }
        Ok(())
    }
}

impl Serialize for ChunkData {
    fn serialize(&self, s: &mut Serializer) -> anyhow::Result<()> {
        s.serialize_nbt(&self.heightmaps)?;
        s.serialize_prefixed_byte_array(&self.data)?;
        s.serialize_varint(0)?;
        Ok(())
    }
}

impl Serialize for LightData {
    fn serialize(&self, s: &mut Serializer) -> anyhow::Result<()> {
        s.serialize_prefixed_array(&[0], |s, l| s.serialize_long(*l))?;
        s.serialize_prefixed_array(&[0], |s, l| s.serialize_long(*l))?;
        s.serialize_prefixed_array(&[0], |s, l| s.serialize_long(*l))?;
        s.serialize_prefixed_array(&[0], |s, l| s.serialize_long(*l))?;
        s.serialize_varint(0)?;
        s.serialize_varint(0)?;
        Ok(())
    }
}
