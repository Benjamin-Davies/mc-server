use std::{
    f64,
    time::{Duration, Instant},
};

use chrono::{Datelike, Timelike};
use tokio::net::TcpListener;
use uuid::Uuid;

use net::{
    chunk::Subchunk,
    connection::{Connection, Error, ServerboundPacket},
    nbt,
    packets::{
        configuration, deserialize, handshake, login,
        play::{
            self,
            clientbound::{ChunkData, GameEvent, LightData},
        },
        status::{
            self,
            clientbound::{Players, Status, TextComponent, Version},
        },
    },
    registries,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let listener = TcpListener::bind("0.0.0.0:25565").await?;
    println!("Listening on port 25565");
    loop {
        let (stream, src) = listener.accept().await?;
        dbg!(src);
        let connection = Connection::new(stream);
        tokio::spawn(async move {
            let _ = handle_connection(connection).await;
        });
    }
}

async fn handle_connection(mut connection: Connection) -> Result<(), Error> {
    let mut last_keepalive = Instant::now();

    loop {
        match handle_packet(&mut connection, &mut last_keepalive).await {
            Ok(packet) => packet,
            Err(
                err @ Error::DeserializeError {
                    source: deserialize::Error::InvalidPacketId { .. },
                },
            ) => {
                eprintln!("{err}");
            }
            Err(Error::ClientTimedOut) => return Ok(()),
            Err(err) => {
                eprintln!("Error handling connection: {err}");
                let _ = connection.disconnect(&err.to_string()).await;
                return Err(err);
            }
        };
    }
}

async fn handle_packet(
    connection: &mut Connection,
    last_keepalive: &mut Instant,
) -> Result<(), Error> {
    // https://minecraft.wiki/w/Java_Edition_protocol?oldid=2874788
    const PROTOCOL_VERSION: i32 = 769;
    const GAME_VERSION: &str = "1.21.4";

    match connection.recv().await? {
        ServerboundPacket::Handshake(packet) => match packet {
            handshake::serverbound::Packet::Intention {
                protocol_version,
                server_address,
                server_port,
                ..
            } => {
                assert_eq!(protocol_version, PROTOCOL_VERSION);
                dbg!(server_address, server_port);
            }
        },
        ServerboundPacket::Status(packet) => match packet {
            status::serverbound::Packet::StatusRequest => {
                let dt = chrono::Local::now();
                let time_str = dt.format("%H:%M:%S").to_string();

                let status = Status {
                    version: Version {
                        name: GAME_VERSION,
                        protocol: PROTOCOL_VERSION,
                    },
                    players: Players {
                        max: dt.month(),
                        online: dt.day(),
                    },
                    description: TextComponent { text: &time_str },
                };

                connection
                    .send(status::clientbound::Packet::StatusResponse { status })
                    .await?;
            }
            status::serverbound::Packet::PingRequest { timestamp } => {
                connection
                    .send(status::clientbound::Packet::PongResponse { timestamp })
                    .await?;
            }
        },
        ServerboundPacket::Login(packet) => match packet {
            login::serverbound::Packet::Hello { name, player_uuid } => {
                dbg!(&name, player_uuid);
                let uuid = Uuid::new_v4();
                let username = &name;

                connection
                    .send(login::clientbound::Packet::LoginFinished { uuid, username })
                    .await?;
            }
            _ => {}
        },
        ServerboundPacket::Configuration(packet) => match packet {
            configuration::serverbound::Packet::ClientInformation { .. } => {
                connection
                    .send(configuration::clientbound::Packet::SelectKnownPacks {
                        known_packs: &[("minecraft", "core", GAME_VERSION)],
                    })
                    .await?;
            }
            configuration::serverbound::Packet::FinishConfiguration => {
                // https://minecraft.wiki/w/Java_Edition_protocol/FAQ#%E2%80%A6my_player_isn't_spawning!
                connection
                    .send(play::clientbound::Packet::Login {
                        entity_id: 1,
                        game_mode: 1,
                        is_flat: true,
                        enforces_secure_chat: true,
                    })
                    .await?;
                connection
                    .send(play::clientbound::Packet::GameEvent {
                        event: GameEvent::StartChunks,
                        value: 0.0,
                    })
                    .await?;

                let chunk = Subchunk::demo();
                connection
                    .send(play::clientbound::Packet::LevelChunkWithLight {
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
                    connection
                        .send(play::clientbound::Packet::AddEntity {
                            entity_id: i + 10,
                            entity_uuid: Uuid::new_v4(),
                            entity_type: registries::entity_type("minecraft:phantom")
                                .unwrap()
                                .protocol_id,
                            x: 8.0
                                - f64::sin(f64::consts::PI * minute_angle as f64 / 128.0)
                                    * i as f64,
                            y: 8.0
                                + f64::cos(f64::consts::PI * minute_angle as f64 / 128.0)
                                    * i as f64,
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
                    connection
                        .send(play::clientbound::Packet::AddEntity {
                            entity_id: i + 20,
                            entity_uuid: Uuid::new_v4(),
                            entity_type: registries::entity_type("minecraft:phantom")
                                .unwrap()
                                .protocol_id,
                            x: 8.0
                                - f64::sin(f64::consts::PI * hour_angle as f64 / 128.0) * i as f64,
                            y: 8.0
                                + f64::cos(f64::consts::PI * hour_angle as f64 / 128.0) * i as f64,
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

                connection
                    .send(play::clientbound::Packet::PlayerPosition {
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
            }
            configuration::serverbound::Packet::SelectKnownPacks { known_packs } => {
                dbg!(known_packs);

                send_registry_data(connection).await?;

                connection
                    .send(configuration::clientbound::Packet::FinishConfiguration)
                    .await?;
            }
            _ => {}
        },
        ServerboundPacket::Play(packet) => match packet {
            play::serverbound::Packet::ClientTickEnd => {
                if Instant::now() - *last_keepalive >= Duration::from_secs(10) {
                    connection
                        .send(play::clientbound::Packet::KeepAlive { keep_alive_id: 0 })
                        .await?;
                    *last_keepalive = Instant::now();
                }
            }
            _ => {}
        },
    }

    Ok(())
}

async fn send_registry_data(connection: &mut Connection) -> Result<(), Error> {
    let damage_types = [
        "minecraft:arrow",
        "minecraft:bad_respawn_point",
        "minecraft:cactus",
        "minecraft:campfire",
        "minecraft:cramming",
        "minecraft:dragon_breath",
        "minecraft:drown",
        "minecraft:dry_out",
        "minecraft:ender_pearl",
        "minecraft:explosion",
        "minecraft:fall",
        "minecraft:falling_anvil",
        "minecraft:falling_block",
        "minecraft:falling_stalactite",
        "minecraft:fireball",
        "minecraft:fireworks",
        "minecraft:fly_into_wall",
        "minecraft:freeze",
        "minecraft:generic",
        "minecraft:generic_kill",
        "minecraft:hot_floor",
        "minecraft:in_fire",
        "minecraft:in_wall",
        "minecraft:indirect_magic",
        "minecraft:lava",
        "minecraft:lightning_bolt",
        "minecraft:magic",
        "minecraft:mob_attack",
        "minecraft:mob_attack_no_aggro",
        "minecraft:mob_projectile",
        "minecraft:on_fire",
        "minecraft:out_of_world",
        "minecraft:outside_border",
        "minecraft:player_attack",
        "minecraft:player_explosion",
        "minecraft:sonic_boom",
        "minecraft:spit",
        "minecraft:stalagmite",
        "minecraft:starve",
        "minecraft:sting",
        "minecraft:sweet_berry_bush",
        "minecraft:thorns",
        "minecraft:thrown",
        "minecraft:trident",
        "minecraft:unattributed_fireball",
        "minecraft:wind_charge",
        "minecraft:wither",
        "minecraft:wither_skull",
    ]
    .into_iter()
    .map(|key| {
        (
            key,
            Some(nbt!(
                {
                    exhaustion: 0.0,
                    message_id: "onFire",
                    scaling: "when_caused_by_living_non_player"
                }
            )),
        )
    })
    .collect::<Vec<_>>();

    let registries = [
        configuration::clientbound::Packet::RegistryData {
            registry_id: "damage_type",
            entries: &damage_types,
        },
        configuration::clientbound::Packet::RegistryData {
            registry_id: "dimension_type",
            entries: &[(
                "minecraft:overworld",
                Some(nbt!(
                    {
                        ambient_light: 0.0,
                        bed_works: 1,
                        coordinate_scale: 1.0,
                        effects: "minecraft:overworld",
                        has_ceiling: 0,
                        has_raids: 1,
                        has_skylight: 1,
                        height: 16,
                        infiniburn: "#minecraft:infiniburn_overworld",
                        logical_height: 16,
                        min_y: 0,
                        monster_spawn_block_light_limit: 0,
                        monster_spawn_light_level: {
                            max_inclusive: 7,
                            min_inclusive: 0,
                            type: "minecraft:uniform"
                        },
                        natural: 1,
                        piglin_safe: 0,
                        respawn_anchor_works: 0,
                        ultrawarm: 0,
                    }
                )),
            )],
        },
        configuration::clientbound::Packet::RegistryData {
            registry_id: "painting_variant",
            entries: &[(
                "placeholder",
                Some(nbt!(
                    {
                        asset_id: "minecraft:alban",
                        width: 1,
                        height: 1,
                    }
                )),
            )],
        },
        configuration::clientbound::Packet::RegistryData {
            registry_id: "wolf_variant",
            entries: &[(
                "placeholder",
                Some(nbt!(
                    {
                        angry_texture: "minecraft:entity/wolf/wolf_ashen_angry",
                        biomes: "minecraft:snowy_taiga",
                        tame_texture: "minecraft:entity/wolf/wolf_ashen_tame",
                        wild_texture: "minecraft:entity/wolf/wolf_ashen",
                    }
                )),
            )],
        },
        configuration::clientbound::Packet::RegistryData {
            registry_id: "worldgen/biome",
            entries: &[
                (
                    "minecraft:snowy_taiga",
                    Some(nbt!(
                        {
                            downfall: 0.4000000059604645,
                            effects: {
                                fog_color: 12638463,
                                mood_sound: {
                                    block_search_extent: 8,
                                    offset: 2.0,
                                    sound: "minecraft:ambient.cave",
                                    tick_delay: 6000
                                },
                                sky_color: 8625919,
                                water_color: 4020182,
                                water_fog_color: 329011
                            },
                            has_precipitation: true,
                            temperature: (-0.5),
                        }
                    )),
                ),
                (
                    "minecraft:plains",
                    Some(nbt!(
                        {
                            downfall: 0.4000000059604645,
                            effects: {
                                fog_color: 12638463,
                                mood_sound: {
                                    block_search_extent: 8,
                                    offset: 2.0,
                                    sound: "minecraft:ambient.cave",
                                    tick_delay: 6000
                                },
                                sky_color: 8625919,
                                water_color: 4020182,
                                water_fog_color: 329011
                            },
                            has_precipitation: true,
                            temperature: 0.8,
                        }
                    )),
                ),
            ],
        },
    ];

    for registry in registries {
        connection.send(registry).await?;
    }

    Ok(())
}
