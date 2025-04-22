use std::{
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use chrono::Datelike;
use uuid::Uuid;

use net::nbt;

use crate::{
    decode::Parse,
    types::{
        ConfigurationRequest, ConfigurationResponse, GameEvent, HandshakeRequest,
        HandshakeRequestNextState, LoginRequest, LoginResponse, PlayRequest, PlayResponse, Players,
        Status, StatusRequest, StatusResponse, TextComponent, Version,
    },
};

mod connection;
mod decode;
mod encode;
mod types;

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:25565")?;
    println!("Listening on port 25565");
    loop {
        let (mut stream, src) = listener.accept()?;
        dbg!(src);
        thread::spawn(move || {
            handle_connection(&mut stream).unwrap();
        });
    }
}

fn handle_connection(stream: &mut TcpStream) -> anyhow::Result<()> {
    #[derive(Debug)]
    enum State {
        Handshaking,
        Status,
        Login,
        Configuration,
        Play,
    }

    // https://minecraft.wiki/w/Java_Edition_protocol?oldid=2874788
    const PROTOCOL_VERSION: i32 = 769;
    const GAME_VERSION: &str = "1.21.4";

    let mut state = State::Handshaking;

    while let Ok(packet) = connection::read_packet(stream) {
        match state {
            State::Handshaking => match packet.parse()? {
                HandshakeRequest::Intention {
                    protocol_version,
                    server_address,
                    server_port,
                    next_state,
                } => {
                    anyhow::ensure!(protocol_version == PROTOCOL_VERSION);
                    dbg!(server_address, server_port);
                    match next_state {
                        HandshakeRequestNextState::Status => state = State::Status,
                        HandshakeRequestNextState::Login => state = State::Login,
                        _ => todo!("handshake request next state: {next_state:?}"),
                    }
                }
            },
            State::Status => match packet.parse()? {
                StatusRequest::StatusRequest => {
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

                    connection::write_packet(stream, StatusResponse::StatusResponse { status })?;
                }
                StatusRequest::PingRequest { timestamp } => {
                    connection::write_packet(stream, StatusResponse::PongResponse { timestamp })?;
                }
            },
            State::Login => match packet.parse()? {
                LoginRequest::Hello { name, player_uuid } => {
                    dbg!(name, player_uuid);
                    let uuid = Uuid::new_v4();
                    let username = name;

                    connection::write_packet(
                        stream,
                        LoginResponse::LoginFinished { uuid, username },
                    )?;
                }
                LoginRequest::LoginAcknowledged => state = State::Configuration,
            },
            State::Configuration => match packet.parse()? {
                ConfigurationRequest::ClientInformation { .. } => {
                    connection::write_packet(
                        stream,
                        ConfigurationResponse::SelectKnownPacks {
                            known_packs: &[("minecraft", "core", GAME_VERSION)],
                        },
                    )?;
                }
                ConfigurationRequest::CustomPayload { .. } => {}
                ConfigurationRequest::FinishConfiguration => {
                    state = State::Play;
                    dbg!(&state);

                    // https://minecraft.wiki/w/Java_Edition_protocol/FAQ#%E2%80%A6my_player_isn't_spawning!
                    connection::write_packet(
                        stream,
                        PlayResponse::Login {
                            entity_id: 1,
                            enforces_secure_chat: true,
                        },
                    )?;
                    connection::write_packet(
                        stream,
                        PlayResponse::GameEvent {
                            event: GameEvent::StartChunks,
                            value: 0.0,
                        },
                    )?;
                    connection::write_packet(
                        stream,
                        PlayResponse::PlayerPosition {
                            teleport_id: 0,
                            x: 0.0,
                            y: -16.0,
                            z: 0.0,
                            velocity_x: 0.0,
                            velocity_y: 0.0,
                            velocity_z: 0.0,
                            yaw: 0.0,
                            pitch: 0.0,
                        },
                    )?;
                }
                ConfigurationRequest::SelectKnownPacks { known_packs } => {
                    dbg!(known_packs);

                    send_registry_data(stream)?;

                    connection::write_packet(stream, ConfigurationResponse::FinishConfiguration)?;
                }
            },
            State::Play => match packet.parse().unwrap_or(PlayRequest::ClientTickEnd) {
                PlayRequest::AcceptTeleportation { teleport_id } => {
                    dbg!(teleport_id);

                    loop {
                        connection::write_packet(
                            stream,
                            PlayResponse::KeepAlive { keep_alive_id: 0 },
                        )?;
                        thread::sleep(Duration::from_secs(10));
                    }
                }
                _ => {}
            },
        }
    }

    Ok(())
}

fn send_registry_data(stream: &mut TcpStream) -> anyhow::Result<()> {
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
        ConfigurationResponse::RegistryData {
            registry_id: "damage_type",
            entries: &damage_types,
        },
        ConfigurationResponse::RegistryData {
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
                        height: 384,
                        infiniburn: "#minecraft:infiniburn_overworld",
                        logical_height: 64,
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
        ConfigurationResponse::RegistryData {
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
        ConfigurationResponse::RegistryData {
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
        ConfigurationResponse::RegistryData {
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
        connection::write_packet(stream, registry)?;
    }

    Ok(())
}
