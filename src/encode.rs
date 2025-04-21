use crate::{
    nbt,
    types::{ConfigurationResponse, LoginResponse, PlayResponse, StatusResponse},
};

pub trait Encode {
    fn encode(&self) -> anyhow::Result<Vec<u8>>;
}

fn boolean(buf: &mut Vec<u8>, b: bool) {
    buf.push(b as u8);
}

fn byte(buf: &mut Vec<u8>, n: i8) {
    buf.push(n as u8);
}

fn ubyte(buf: &mut Vec<u8>, n: u8) {
    buf.push(n);
}

fn short(buf: &mut Vec<u8>, n: i16) {
    let bytes = n.to_be_bytes();
    buf.extend_from_slice(&bytes);
}

fn ushort(buf: &mut Vec<u8>, n: u16) {
    let bytes = n.to_be_bytes();
    buf.extend_from_slice(&bytes);
}

fn int(buf: &mut Vec<u8>, n: i32) {
    let bytes = n.to_be_bytes();
    buf.extend_from_slice(&bytes);
}

fn long(buf: &mut Vec<u8>, n: i64) {
    let bytes = n.to_be_bytes();
    buf.extend_from_slice(&bytes);
}

fn varint(buf: &mut Vec<u8>, n: i32) {
    let mut n = n;
    loop {
        let mut byte = (n & 0x7F) as u8;
        n >>= 7;
        if n != 0 {
            byte |= 0x80;
        }
        buf.push(byte);
        if n == 0 {
            break;
        }
    }
}

fn float(buf: &mut Vec<u8>, n: f32) {
    let bytes = n.to_be_bytes();
    buf.extend_from_slice(&bytes);
}

fn double(buf: &mut Vec<u8>, n: f64) {
    let bytes = n.to_be_bytes();
    buf.extend_from_slice(&bytes);
}

fn string(buf: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    prefixed_byte_array(buf, bytes);
}

fn uuid(buf: &mut Vec<u8>, uuid: uuid::Uuid) {
    buf.extend_from_slice(uuid.as_bytes());
}

fn prefixed_optional<T>(
    buf: &mut Vec<u8>,
    data: &Option<T>,
    mut f: impl FnMut(&mut Vec<u8>, &T) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    let present = data.is_some();
    boolean(buf, present);
    if let Some(data) = data {
        f(buf, data)?;
    }
    Ok(())
}

fn prefixed_array<T>(
    buf: &mut Vec<u8>,
    data: &[T],
    mut f: impl FnMut(&mut Vec<u8>, &T) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    let len = data.len() as i32;
    varint(buf, len);
    for item in data {
        f(buf, item)?;
    }
    Ok(())
}

pub(crate) fn prefixed_byte_array(buf: &mut Vec<u8>, data: &[u8]) {
    let len = data.len() as i32;
    varint(buf, len);
    buf.extend_from_slice(data);
}

fn nbt(buf: &mut Vec<u8>, tag: &nbt::Tag) -> anyhow::Result<()> {
    ubyte(buf, tag.kind() as u8);
    nbt_body(buf, tag)?;
    Ok(())
}

fn nbt_named(buf: &mut Vec<u8>, name: &str, tag: &nbt::Tag) -> anyhow::Result<()> {
    ubyte(buf, tag.kind() as u8);
    ushort(buf, name.len() as u16);
    buf.extend_from_slice(name.as_bytes());
    nbt_body(buf, tag)?;
    Ok(())
}

fn nbt_body(buf: &mut Vec<u8>, tag: &nbt::Tag) -> anyhow::Result<()> {
    match tag {
        nbt::Tag::End => {}
        nbt::Tag::Byte(n) => byte(buf, *n),
        nbt::Tag::Short(n) => short(buf, *n),
        nbt::Tag::Int(n) => int(buf, *n),
        nbt::Tag::Long(n) => long(buf, *n),
        nbt::Tag::Float(n) => float(buf, *n),
        nbt::Tag::Double(n) => double(buf, *n),
        nbt::Tag::ByteArray(_) => todo!(),
        nbt::Tag::String(s) => {
            ushort(buf, s.len() as u16);
            buf.extend_from_slice(s.as_bytes());
        }
        nbt::Tag::List(_) => todo!(),
        nbt::Tag::Compound(items) => {
            for (key, value) in items {
                nbt_named(buf, key, value)?;
            }
            nbt(buf, &nbt::Tag::End)?;
        }
        nbt::Tag::IntArray(_) => todo!(),
        nbt::Tag::LongArray(items) => {
            int(buf, items.len() as i32);
            for item in items {
                long(buf, *item);
            }
        }
    }
    Ok(())
}

impl Encode for StatusResponse<'_> {
    fn encode(&self) -> anyhow::Result<Vec<u8>> {
        let mut buf = Vec::new();

        match self {
            StatusResponse::Status { status } => {
                varint(&mut buf, 0x00);
                string(&mut buf, &serde_json::to_string(status)?);
            }
            StatusResponse::Pong { timestamp } => {
                varint(&mut buf, 0x01);
                long(&mut buf, *timestamp);
            }
        }

        Ok(buf)
    }
}

impl Encode for LoginResponse<'_> {
    fn encode(&self) -> anyhow::Result<Vec<u8>> {
        let mut buf = Vec::new();

        match self {
            LoginResponse::Disconnect { reason } => {
                varint(&mut buf, 0x00);
                string(&mut buf, &serde_json::to_string(reason)?);
            }
            LoginResponse::LoginSuccess {
                uuid: player_uuid,
                username,
            } => {
                varint(&mut buf, 0x02);
                uuid(&mut buf, *player_uuid);
                string(&mut buf, username);
                prefixed_byte_array(&mut buf, &[]);
            }
        }

        Ok(buf)
    }
}

impl Encode for ConfigurationResponse<'_> {
    fn encode(&self) -> anyhow::Result<Vec<u8>> {
        let mut buf = Vec::new();

        match self {
            ConfigurationResponse::FinishConfiguration => {
                varint(&mut buf, 0x03);
            }
            ConfigurationResponse::RegistryData {
                registry_id,
                entries,
            } => {
                varint(&mut buf, 0x07);
                string(&mut buf, registry_id);
                prefixed_array(&mut buf, entries, |buf, (entry_id, entry_data)| {
                    string(buf, entry_id);
                    prefixed_optional(buf, entry_data, |buf, data| nbt(buf, data))?;
                    Ok(())
                })?;
            }
            ConfigurationResponse::KnownPacks { known_packs } => {
                varint(&mut buf, 0x0E);
                prefixed_array(&mut buf, known_packs, |buf, (namespace, id, version)| {
                    string(buf, namespace);
                    string(buf, id);
                    string(buf, version);
                    Ok(())
                })?;
            }
        }

        Ok(buf)
    }
}

impl Encode for PlayResponse {
    fn encode(&self) -> anyhow::Result<Vec<u8>> {
        let mut buf = Vec::new();

        match self {
            PlayResponse::GameEvent { event, value } => {
                varint(&mut buf, 0x23);
                ubyte(&mut buf, *event as u8);
                float(&mut buf, *value);
                dbg!(&buf);
            }
            PlayResponse::Login {
                entity_id,
                enforces_secure_chat,
            } => {
                varint(&mut buf, 0x2C);
                int(&mut buf, *entity_id);
                boolean(&mut buf, false);
                prefixed_array(&mut buf, &["overworld"], |buf, s| {
                    string(buf, s);
                    Ok(())
                })?;
                varint(&mut buf, 1);
                varint(&mut buf, 2);
                varint(&mut buf, 2);
                boolean(&mut buf, false);
                boolean(&mut buf, false);
                boolean(&mut buf, false);
                varint(&mut buf, 0);
                string(&mut buf, "overworld");
                long(&mut buf, 0);
                ubyte(&mut buf, 3);
                byte(&mut buf, -1);
                boolean(&mut buf, false);
                boolean(&mut buf, false);
                boolean(&mut buf, false);
                varint(&mut buf, 0);
                varint(&mut buf, 0);
                boolean(&mut buf, *enforces_secure_chat);
            }
            PlayResponse::SynchronizePlayerPosition {
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
                varint(&mut buf, 0x42);
                varint(&mut buf, *teleport_id);
                double(&mut buf, *x);
                double(&mut buf, *y);
                double(&mut buf, *z);
                double(&mut buf, *velocity_x);
                double(&mut buf, *velocity_y);
                double(&mut buf, *velocity_z);
                float(&mut buf, *yaw);
                float(&mut buf, *pitch);
                buf.extend_from_slice(&[0; 4]);
            }
        }

        Ok(buf)
    }
}
