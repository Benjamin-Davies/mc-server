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
            clientbound::{ChunkData, GameEvent, LightData},
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
            game_mode: 1,
            is_flat: true,
            enforces_secure_chat: true,
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

        let now = chrono::Local::now().time();
        let minute_angle = now.num_seconds_from_midnight() * 256 / 3_600 % 256;
        let abs_minute_angle;
        let mirror_minute;
        if minute_angle < 128 {
            abs_minute_angle = minute_angle;
            mirror_minute = false;
        } else {
            abs_minute_angle = 256 - minute_angle;
            mirror_minute = true;
        }
        let hour_angle = now.num_seconds_from_midnight() * 256 / 24 / 3_600 % 256;
        let abs_hour_angle;
        let mirror_hour;
        if hour_angle < 128 {
            abs_hour_angle = hour_angle;
            mirror_hour = false;
        } else {
            abs_hour_angle = 256 - hour_angle;
            mirror_hour = true;
        }
        for i in 1..=4 {
            conn.send(play::clientbound::Packet::AddEntity {
                entity_id: i + 10,
                entity_uuid: Uuid::new_v4(),
                entity_type: registries::entity_type("minecraft:phantom")
                    .unwrap()
                    .protocol_id,
                x: 8.0 - f64::sin(f64::consts::PI * minute_angle as f64 / 128.0) * i as f64,
                y: 8.0 + f64::cos(f64::consts::PI * minute_angle as f64 / 128.0) * i as f64,
                z: 15.5,
                pitch: ((256 + 64 - abs_minute_angle) % 256) as u8,
                yaw: if mirror_minute { 192 } else { 64 },
                head_yaw: 0,
                data: 0,
                velocity_x: 0,
                velocity_y: 0,
                velocity_z: 0,
            })
            .await?;
        }
        for i in 1..=3 {
            conn.send(play::clientbound::Packet::AddEntity {
                entity_id: i + 20,
                entity_uuid: Uuid::new_v4(),
                entity_type: registries::entity_type("minecraft:phantom")
                    .unwrap()
                    .protocol_id,
                x: 8.0 - f64::sin(f64::consts::PI * hour_angle as f64 / 128.0) * i as f64,
                y: 8.0 + f64::cos(f64::consts::PI * hour_angle as f64 / 128.0) * i as f64,
                z: 15.5,
                pitch: ((256 + 64 - abs_hour_angle) % 256) as u8,
                yaw: if mirror_hour { 192 } else { 64 },
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
            y: 1.0,
            z: 2.0,
            velocity_x: 0.0,
            velocity_y: 0.0,
            velocity_z: 0.0,
            yaw: 0.0,
            pitch: -23.0,
        })
        .await?;

        Ok(())
    }
}
