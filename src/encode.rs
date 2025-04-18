use crate::types::{ConfigurationResponse, LoginResponse, PlayResponse, StatusResponse};

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

fn prefixed_array<T>(buf: &mut Vec<u8>, data: &[T], mut f: impl FnMut(&mut Vec<u8>, &T)) {
    let len = data.len() as i32;
    varint(buf, len);
    for item in data {
        f(buf, item);
    }
}

pub(crate) fn prefixed_byte_array(buf: &mut Vec<u8>, data: &[u8]) {
    let len = data.len() as i32;
    varint(buf, len);
    buf.extend_from_slice(data);
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
            ConfigurationResponse::KnownPacks { known_packs } => {
                varint(&mut buf, 0x0E);
                prefixed_array(&mut buf, known_packs, |buf, (namespace, id, version)| {
                    string(buf, namespace);
                    string(buf, id);
                    string(buf, version);
                });
            }
        }

        Ok(buf)
    }
}

impl Encode for PlayResponse {
    fn encode(&self) -> anyhow::Result<Vec<u8>> {
        let mut buf = Vec::new();

        match self {
            PlayResponse::Login { entity_id } => {
                varint(&mut buf, 0x2C);
                int(&mut buf, *entity_id);
                boolean(&mut buf, false);
                prefixed_array(&mut buf, &["overworld"], |buf, s| string(buf, s));
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
                boolean(&mut buf, false);
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
