use crate::{
    nbt,
    packets::serialize::{Serialize, Serializer, types},
};

#[derive(Debug, Serialize)]
pub enum Packet<'a> {
    #[packet(id = 0x01)]
    AddEntity {
        entity_id: types::varint,
        entity_uuid: types::uuid,
        entity_type: types::varint,
        x: types::double,
        y: types::double,
        z: types::double,
        pitch: types::ubyte,
        yaw: types::ubyte,
        head_yaw: types::ubyte,
        data: types::varint,
        velocity_x: types::short,
        velocity_y: types::short,
        velocity_z: types::short,
    },
    #[packet(id = 0x0C)]
    ChunkBatchFinished { batch_size: types::varint },
    #[packet(id = 0x0D)]
    ChunkBatchStart,
    #[packet(id = 0x1D)]
    Disconnect {
        #[packet(serialize_with = nbt::Tag::from(*reason).serialize(s))]
        reason: &'a str,
    },
    #[packet(id = 0x20)]
    EntityPositionSync {
        entity_id: types::varint,
        x: types::double,
        y: types::double,
        z: types::double,
        velocity_x: types::double,
        velocity_y: types::double,
        velocity_z: types::double,
        yaw: types::float,
        pitch: types::float,
        on_ground: types::boolean,
    },
    #[packet(id = 0x23)]
    GameEvent {
        #[packet(serialize_with = s.serialize_ubyte(*event as u8))]
        event: GameEvent,
        value: types::float,
    },
    #[packet(id = 0x27)]
    KeepAlive { keep_alive_id: types::long },
    #[packet(id = 0x28)]
    LevelChunkWithLight {
        chunk_x: types::int,
        chunk_z: types::int,
        data: ChunkData,
        light: LightData,
    },
    #[packet(id = 0x2C)]
    Login {
        entity_id: types::int,
        data: LoginData,
    },
    #[packet(id = 0x42)]
    PlayerPosition {
        teleport_id: types::varint,
        x: types::double,
        y: types::double,
        z: types::double,
        velocity_x: types::double,
        velocity_y: types::double,
        velocity_z: types::double,
        yaw: types::float,
        pitch: types::float,
        flags: types::int,
    },
    #[packet(id = 0x58)]
    SetChunkCacheCenter {
        chunk_x: types::varint,
        chunk_z: types::varint,
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

#[derive(Debug)]
pub struct LoginData {
    pub game_mode: u8,
    pub enforces_secure_chat: bool,
    pub is_flat: bool,
}

impl Serialize for ChunkData {
    fn serialize(&self, s: &mut Serializer) {
        self.heightmaps.serialize(s);
        s.serialize_prefixed_byte_array(&self.data);
        s.serialize_varint(0);
    }
}

impl Serialize for LightData {
    fn serialize(&self, s: &mut Serializer) {
        s.serialize_prefixed_array_with(&[0], |s, item| s.serialize_long(*item));
        s.serialize_prefixed_array_with(&[0], |s, item| s.serialize_long(*item));
        s.serialize_prefixed_array_with(&[0], |s, item| s.serialize_long(*item));
        s.serialize_prefixed_array_with(&[0], |s, item| s.serialize_long(*item));
        s.serialize_varint(0);
        s.serialize_varint(0);
    }
}

impl Serialize for LoginData {
    fn serialize(&self, s: &mut Serializer) {
        let LoginData {
            game_mode,
            enforces_secure_chat,
            is_flat,
        } = self;
        s.serialize_boolean(false);
        s.serialize_prefixed_array_with(&["overworld"], |s, item| s.serialize_string(item));
        s.serialize_varint(1);
        s.serialize_varint(8);
        s.serialize_varint(8);
        s.serialize_boolean(false);
        s.serialize_boolean(false);
        s.serialize_boolean(false);
        s.serialize_varint(0);
        s.serialize_string("overworld");
        s.serialize_long(0);
        s.serialize_ubyte(*game_mode);
        s.serialize_byte(-1);
        s.serialize_boolean(false);
        s.serialize_boolean(*is_flat);
        s.serialize_boolean(false);
        s.serialize_varint(0);
        s.serialize_varint(0);
        s.serialize_boolean(*enforces_secure_chat);
    }
}
