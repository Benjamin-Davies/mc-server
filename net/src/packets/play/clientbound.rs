use crate::packets::serialize::{Serialize, Serializer};

#[derive(Debug)]
pub enum Packet {
    GameEvent {
        event: GameEvent,
        value: f32,
    },
    KeepAlive {
        keep_alive_id: i64,
    },
    Login {
        entity_id: i32,
        enforces_secure_chat: bool,
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum GameEvent {
    StartChunks = 13,
}

impl Serialize for Packet {
    fn serialize(&self, s: &mut Serializer) -> anyhow::Result<()> {
        match self {
            Self::GameEvent { event, value } => {
                s.serialize_varint(0x23)?;
                s.serialize_ubyte(*event as u8)?;
                s.serialize_float(*value)?;
            }
            Self::KeepAlive { keep_alive_id } => {
                s.serialize_varint(0x27)?;
                s.serialize_long(*keep_alive_id)?;
            }
            Self::Login {
                entity_id,
                enforces_secure_chat,
            } => {
                s.serialize_varint(0x2C)?;
                s.serialize_int(*entity_id)?;
                s.serialize_boolean(false)?;
                s.serialize_prefixed_array(&["overworld"], |s, item| s.serialize_string(item))?;
                s.serialize_varint(1)?;
                s.serialize_varint(2)?;
                s.serialize_varint(2)?;
                s.serialize_boolean(false)?;
                s.serialize_boolean(false)?;
                s.serialize_boolean(false)?;
                s.serialize_varint(0)?;
                s.serialize_string("overworld")?;
                s.serialize_long(0)?;
                s.serialize_ubyte(3)?;
                s.serialize_byte(-1)?;
                s.serialize_boolean(false)?;
                s.serialize_boolean(false)?;
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
        }
        Ok(())
    }
}
