use std::{
    io,
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use snafu::prelude::*;
use tokio::net::TcpListener;
use uuid::Uuid;

use crate::{
    connection::{self, Connection, ServerboundPacket},
    nbt,
    packets::{
        configuration::{
            self,
            clientbound::{KnownPack, RegistryEntry},
        },
        deserialize, handshake, login, play,
        status::{
            self,
            clientbound::{Players, Status, TextComponent, Version},
        },
    },
};

#[async_trait]
pub trait Callbacks: Send + Sync {
    fn description(&self) -> TextComponent {
        TextComponent {
            text: "Minecraft Server".to_owned(),
        }
    }

    fn players(&self) -> Players {
        Players { max: 20, online: 0 }
    }

    fn dimension_data(&self) -> DimensionData;

    async fn on_login(&self, conn: &mut Connection) -> Result<(), Error>;
    async fn on_tick(&self, conn: &mut Connection) -> Result<(), Error>;
}

pub struct Server {
    callbacks: Box<dyn Callbacks>,
}

struct Client {
    connection: Connection,
    last_keepalive: Instant,
    server: Arc<Server>,
}

pub struct DimensionData {
    pub height: i32,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(transparent)]
    IOError { source: io::Error },
    #[snafu(transparent)]
    ConnectionError { source: connection::Error },
}

// https://minecraft.wiki/w/Java_Edition_protocol?oldid=2874788
const PROTOCOL_VERSION: i32 = 769;
const GAME_VERSION: &str = "1.21.4";

impl Server {
    pub fn new(callbacks: impl Callbacks + 'static) -> Self {
        Self {
            callbacks: Box::new(callbacks),
        }
    }

    pub async fn listen(self, addr: &str) -> Result<(), Error> {
        let listener = TcpListener::bind(addr).await?;
        println!("Listening at {addr}");
        let server = Arc::new(self);
        loop {
            let (stream, _) = listener.accept().await?;
            let client = Client {
                connection: Connection::new(stream),
                last_keepalive: Instant::now(),
                server: server.clone(),
            };
            tokio::spawn(async move {
                client.handle_connection().await;
            });
        }
    }
}

impl Client {
    async fn handle_connection(mut self) {
        loop {
            match self.handle_packet().await {
                Ok(packet) => packet,
                Err(
                    err @ Error::ConnectionError {
                        source:
                            connection::Error::DeserializeError {
                                source: deserialize::Error::InvalidPacketId { .. },
                            },
                    },
                ) => {
                    eprintln!("{err}");
                }
                Err(Error::ConnectionError {
                    source: connection::Error::ClientTimedOut,
                }) => return,
                Err(err) => {
                    eprintln!("Error handling connection: {err}");
                    let _ = self.connection.disconnect(&err.to_string()).await;
                    return;
                }
            };
        }
    }

    async fn handle_packet(&mut self) -> Result<(), Error> {
        match self.connection.recv().await? {
            ServerboundPacket::Handshake(packet) => match packet {
                handshake::serverbound::Packet::Intention {
                    protocol_version, ..
                } => {
                    assert_eq!(protocol_version, PROTOCOL_VERSION);
                }
            },
            ServerboundPacket::Status(packet) => match packet {
                status::serverbound::Packet::StatusRequest => {
                    let status = Status {
                        version: Version {
                            name: GAME_VERSION,
                            protocol: PROTOCOL_VERSION,
                        },
                        players: self.server.callbacks.players(),
                        description: self.server.callbacks.description(),
                    };

                    self.connection
                        .send(status::clientbound::Packet::StatusResponse { status })
                        .await?;
                }
                status::serverbound::Packet::PingRequest { timestamp } => {
                    self.connection
                        .send(status::clientbound::Packet::PongResponse { timestamp })
                        .await?;
                }
            },
            ServerboundPacket::Login(packet) => match packet {
                login::serverbound::Packet::Hello { name, .. } => {
                    let uuid = Uuid::new_v4();
                    let username = &name;

                    self.connection
                        .send(login::clientbound::Packet::LoginFinished {
                            uuid,
                            username,
                            properties: &[],
                        })
                        .await?;
                }
                _ => {}
            },
            ServerboundPacket::Configuration(packet) => match packet {
                configuration::serverbound::Packet::ClientInformation { .. } => {
                    self.connection
                        .send(configuration::clientbound::Packet::SelectKnownPacks {
                            known_packs: &[KnownPack {
                                namespace: "minecraft",
                                id: "core",
                                version: GAME_VERSION,
                            }],
                        })
                        .await?;
                }
                configuration::serverbound::Packet::FinishConfiguration => {
                    self.server.callbacks.on_login(&mut self.connection).await?;
                }
                configuration::serverbound::Packet::SelectKnownPacks { .. } => {
                    send_registry_data(
                        &mut self.connection,
                        self.server.callbacks.dimension_data(),
                    )
                    .await?;

                    self.connection
                        .send(configuration::clientbound::Packet::FinishConfiguration)
                        .await?;
                }
                _ => {}
            },
            ServerboundPacket::Play(packet) => match packet {
                play::serverbound::Packet::ClientTickEnd => {
                    if Instant::now() - self.last_keepalive >= Duration::from_secs(10) {
                        self.connection
                            .send(play::clientbound::Packet::KeepAlive { keep_alive_id: 0 })
                            .await?;
                        self.last_keepalive = Instant::now();
                    }

                    self.server.callbacks.on_tick(&mut self.connection).await?;
                }
                _ => {}
            },
        }

        Ok(())
    }
}

async fn send_registry_data(
    connection: &mut Connection,
    dimension_data: DimensionData,
) -> Result<(), Error> {
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
    .map(|entry_id| RegistryEntry {
        entry_id,
        entry_data: Some(nbt!(
            {
                exhaustion: 0.0,
                message_id: "onFire",
                scaling: "when_caused_by_living_non_player"
            }
        )),
    })
    .collect::<Vec<_>>();

    let registries = [
        configuration::clientbound::Packet::RegistryData {
            registry_id: "damage_type",
            entries: &damage_types,
        },
        configuration::clientbound::Packet::RegistryData {
            registry_id: "dimension_type",
            entries: &[RegistryEntry {
                entry_id: "minecraft:overworld",
                entry_data: Some(nbt!(
                    {
                        ambient_light: 0.0,
                        bed_works: 1,
                        coordinate_scale: 1.0,
                        effects: "minecraft:overworld",
                        has_ceiling: 0,
                        has_raids: 1,
                        has_skylight: 1,
                        height: (dimension_data.height),
                        infiniburn: "#minecraft:infiniburn_overworld",
                        logical_height: (dimension_data.height),
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
            }],
        },
        configuration::clientbound::Packet::RegistryData {
            registry_id: "painting_variant",
            entries: &[RegistryEntry {
                entry_id: "placeholder",
                entry_data: Some(nbt!(
                    {
                        asset_id: "minecraft:alban",
                        width: 1,
                        height: 1,
                    }
                )),
            }],
        },
        configuration::clientbound::Packet::RegistryData {
            registry_id: "wolf_variant",
            entries: &[RegistryEntry {
                entry_id: "placeholder",
                entry_data: Some(nbt!(
                    {
                        angry_texture: "minecraft:entity/wolf/wolf_ashen_angry",
                        biomes: "minecraft:snowy_taiga",
                        tame_texture: "minecraft:entity/wolf/wolf_ashen_tame",
                        wild_texture: "minecraft:entity/wolf/wolf_ashen",
                    }
                )),
            }],
        },
        configuration::clientbound::Packet::RegistryData {
            registry_id: "worldgen/biome",
            entries: &[
                RegistryEntry {
                    entry_id: "minecraft:snowy_taiga",
                    entry_data: Some(nbt!(
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
                },
                RegistryEntry {
                    entry_id: "minecraft:plains",
                    entry_data: Some(nbt!(
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
                },
            ],
        },
    ];

    for registry in registries {
        connection.send(registry).await?;
    }

    Ok(())
}
