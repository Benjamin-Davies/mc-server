use std::collections::BTreeMap;

use async_trait::async_trait;
use net::{
    chunk::{Block, Chunk},
    connection::Connection,
    nbt,
    packets::{
        play::{
            self,
            clientbound::{ChunkData, GameEvent, LightData, LoginData},
        },
        status::clientbound::{Players, TextComponent},
    },
    server::{self, DimensionData, Error, Server},
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let bytes = include_bytes!("../../data/teapot_50.bin");
    let mut data = vec![0f32; 256 * 256 * 256];
    bytemuck::cast_slice_mut::<f32, u8>(&mut data)[..bytes.len()].copy_from_slice(bytes);

    let mut chunks = BTreeMap::new();
    for chunk_z in -1..17 {
        for chunk_x in -1..17 {
            let chunk = Chunk::from_fn(16, |x, y, z| {
                let x = x as i32 + 16 * chunk_x;
                let y = y as i32;
                let z = z as i32 + 16 * chunk_z;
                if !(0..178).contains(&x) || !(0..256).contains(&z) {
                    return Block::Air;
                }

                let index = ((256 - y) * 256 + z) * 178 + x;
                let value = data[index as usize];

                if value > 0.4 {
                    Block::GrayConcrete
                } else if value > 0.2 {
                    Block::GrayStainedGlass
                } else {
                    Block::Air
                }
            });
            chunks.insert((-chunk_x, chunk_z), chunk);
        }
    }

    Server::new(Callbacks { chunks })
        .listen("0.0.0.0:25565")
        .await?;

    Ok(())
}

struct Callbacks {
    chunks: BTreeMap<(i32, i32), Chunk>,
}

#[async_trait]
impl server::Callbacks for Callbacks {
    fn description(&self) -> TextComponent {
        TextComponent {
            text: "I'm a teapot".to_owned(),
        }
    }

    fn players(&self) -> Players {
        Players {
            max: 418,
            online: 0,
        }
    }

    fn dimension_data(&self) -> DimensionData {
        DimensionData { height: 256 }
    }

    async fn on_login(&self, conn: &mut Connection) -> Result<(), Error> {
        // https://minecraft.wiki/w/Java_Edition_protocol/FAQ#%E2%80%A6my_player_isn't_spawning!
        conn.send(play::clientbound::Packet::Login {
            entity_id: 1,
            data: LoginData {
                game_mode: 3,
                is_flat: true,
                enforces_secure_chat: true,
            },
        })
        .await?;
        conn.send(play::clientbound::Packet::GameEvent {
            event: GameEvent::StartChunks,
            value: 0.0,
        })
        .await?;
        conn.send(play::clientbound::Packet::SetChunkCacheCenter {
            chunk_x: -8,
            chunk_z: 8,
        })
        .await?;

        for (&(chunk_x, chunk_z), chunk) in &self.chunks {
            conn.send(play::clientbound::Packet::LevelChunkWithLight {
                chunk_x,
                chunk_z,
                data: ChunkData {
                    heightmaps: nbt!({
                        WORLD_SURFACE: [-1i64; 22],
                        MOTION_BLOCKING: [-1i64; 22],
                    }),
                    data: chunk.chunk_data(),
                },
                light: LightData { subchunk_count: 16 },
            })
            .await?;
        }

        conn.send(play::clientbound::Packet::PlayerPosition {
            teleport_id: 0,
            x: 0.0,
            y: 160.0,
            z: 220.0,
            velocity_x: 0.0,
            velocity_y: 0.0,
            velocity_z: 0.0,
            yaw: 120.0,
            pitch: 0.0,
            flags: 0,
        })
        .await?;
        conn.send(play::clientbound::Packet::PlayerAbilities {
            flags: 0xF,
            flying_speed: 0.2,
            fov_modifier: 0.1,
        })
        .await?;

        Ok(())
    }

    async fn on_tick(&self, _conn: &mut Connection) -> Result<(), Error> {
        Ok(())
    }
}
