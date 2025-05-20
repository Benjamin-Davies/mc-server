use std::f64;

use async_trait::async_trait;
use chrono::{Datelike, Timelike};
use net::{
    chunk::Subchunk,
    connection::Connection,
    nbt,
    packets::{
        play::{
            self,
            clientbound::{ChunkData, GameEvent, LightData, LoginData},
        },
        status::clientbound::{Players, TextComponent},
    },
    registries,
    server::{self, Error, Server},
};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Error> {
    Server::new(Callbacks).listen("0.0.0.0:25565").await?;
    Ok(())
}

struct Callbacks;

#[async_trait]
impl server::Callbacks for Callbacks {
    fn description(&self) -> TextComponent {
        TextComponent {
            text: chrono::Local::now().format("%H:%M:%S").to_string(),
        }
    }

    fn players(&self) -> Players {
        let now = chrono::Local::now();
        Players {
            max: now.month(),
            online: now.day(),
        }
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

        let chunk = Subchunk::demo();
        conn.send(play::clientbound::Packet::LevelChunkWithLight {
            chunk_x: 0,
            chunk_z: 0,
            data: ChunkData {
                heightmaps: nbt!({
                    WORLD_SURFACE: [-1i64; 22],
                    MOTION_BLOCKING: [-1i64; 22],
                }),
                data: chunk.chunk_data(),
            },
            light: LightData {},
        })
        .await?;

        let empty_chunk = Subchunk::empty();
        for chunk_x in -1..=1 {
            for chunk_z in -1..=1 {
                if (chunk_x, chunk_z) == (0, 0) {
                    continue;
                }
                conn.send(play::clientbound::Packet::LevelChunkWithLight {
                    chunk_x,
                    chunk_z,
                    data: ChunkData {
                        heightmaps: nbt!({
                            WORLD_SURFACE: [-1i64; 22],
                            MOTION_BLOCKING: [-1i64; 22],
                        }),
                        data: empty_chunk.chunk_data(),
                    },
                    light: LightData {},
                })
                .await?;
            }
        }

        for (i, (x, y, _pitch, _yaw)) in phantom_positions().enumerate() {
            conn.send(play::clientbound::Packet::AddEntity {
                entity_id: i as i32 + 10,
                entity_uuid: Uuid::new_v4(),
                entity_type: registries::entity_type("minecraft:phantom")
                    .unwrap()
                    .protocol_id,
                x: 8.0 - x,
                y: 7.75 + y,
                z: 15.5,
                pitch: 0,
                yaw: 0,
                head_yaw: 0,
                data: 0,
                velocity_x: 0,
                velocity_y: 0,
                velocity_z: 0,
            })
            .await?;
        }

        conn.send(play::clientbound::Packet::PlayerPosition {
            teleport_id: 0,
            x: 8.0,
            y: 6.38,
            z: 2.0,
            velocity_x: 0.0,
            velocity_y: 0.0,
            velocity_z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            flags: 0,
        })
        .await?;

        Ok(())
    }

    async fn on_tick(&self, conn: &mut Connection) -> Result<(), Error> {
        for (i, (x, y, pitch, yaw)) in phantom_positions().enumerate() {
            conn.send(play::clientbound::Packet::EntityPositionSync {
                entity_id: i as i32 + 10,
                x: 8.0 - x,
                y: 7.75 + y,
                z: 15.5,
                velocity_x: 0.0,
                velocity_y: 0.0,
                velocity_z: 0.0,
                yaw,
                pitch,
                on_ground: false,
            })
            .await?;
        }

        Ok(())
    }
}

fn phantom_positions() -> impl Iterator<Item = (f64, f64, f32, f32)> {
    let now = chrono::Local::now().time();
    let second_progress = now.num_seconds_from_midnight() as f64 / 60.0 % 1.0;
    let minute_progress = now.num_seconds_from_midnight() as f64 / 3_600.0 % 1.0;
    let hour_progress = now.num_seconds_from_midnight() as f64 / 12.0 / 3_600.0 % 1.0;
    [].into_iter()
        .chain(hand(5, 360.0 * second_progress))
        .chain(hand(4, 360.0 * minute_progress))
        .chain(hand(3, 360.0 * hour_progress))
}

fn hand(length: usize, angle: f64) -> impl Iterator<Item = (f64, f64, f32, f32)> {
    let sin = angle.to_radians().sin();
    let cos = angle.to_radians().cos();

    let pitch;
    let yaw;
    if angle < 180.0 {
        pitch = 90.0 - angle as f32;
        yaw = 90.0;
    } else {
        pitch = 90.0 - 360.0 + angle as f32;
        yaw = -90.0;
    }

    (1..=length).map(move |r| (r as f64 * sin, r as f64 * cos, pitch, yaw))
}
