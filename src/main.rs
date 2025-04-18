use std::{
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use chrono::Datelike;
use types::LoginResponse;

use crate::{
    decode::Parse,
    types::{
        HandshakeRequest, HandshakeRequestNextState, Players, Status, StatusRequest,
        StatusResponse, TextComponent, Version,
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
    }

    let mut state = State::Handshaking;
    let mut protocol = 0;

    while let Ok(packet) = connection::read_packet(stream) {
        match state {
            State::Handshaking => match packet.parse()? {
                HandshakeRequest::Handshake {
                    protocol_version,
                    server_address,
                    server_port,
                    next_state,
                } => {
                    dbg!(protocol_version, server_address, server_port);
                    protocol = protocol_version;
                    match next_state {
                        HandshakeRequestNextState::Status => state = State::Status,
                        HandshakeRequestNextState::Login => {
                            let reason = TextComponent {
                                text: "You can't login to a clock, silly!",
                            };
                            connection::write_packet(stream, LoginResponse::Disconnect { reason })?;
                            return Ok(());
                        }
                        _ => todo!("handshake request next state: {next_state:?}"),
                    }
                }
            },
            State::Status => match packet.parse()? {
                StatusRequest::Status => {
                    let dt = chrono::Local::now();
                    let time_str = dt.format("%H:%M:%S").to_string();

                    let status = Status {
                        version: Version {
                            name: "Clock Server",
                            protocol,
                        },
                        players: Players {
                            max: dt.month(),
                            online: dt.day(),
                        },
                        description: TextComponent { text: &time_str },
                    };

                    connection::write_packet(stream, StatusResponse::Status { status })?;
                }
                StatusRequest::Ping { timestamp } => {
                    let dt = chrono::Local::now();
                    thread::sleep(Duration::from_millis(dt.timestamp_subsec_millis() as u64));

                    connection::write_packet(stream, StatusResponse::Pong { timestamp })?;
                }
            },
        }
    }

    Ok(())
}
