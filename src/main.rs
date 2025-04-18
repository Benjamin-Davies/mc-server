use std::{
    net::{TcpListener, TcpStream},
    thread,
};

use types::{HandshakeRequestNextState, StatusRequest, StatusResponse};

use crate::{decode::Parse, types::HandshakeRequest};

mod connection;
mod decode;
mod encode;
mod types;

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:25565")?;
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
                    match next_state {
                        HandshakeRequestNextState::Status => state = State::Status,
                        _ => todo!("handshake request next state: {next_state:?}"),
                    }
                }
            },
            State::Status => match packet.parse()? {
                StatusRequest::Status => {
                    let json_response = r#"{
                        "version": {
                            "name": "",
                            "protocol": 769
                        },
                        "players": {
                            "max": 100,
                            "online": 5
                        },
                        "description": {
                            "text": "Hello, world!"
                        }
                    }"#;

                    connection::write_packet(stream, StatusResponse::Status { json_response })?;
                }
                StatusRequest::Ping { timestamp } => {
                    connection::write_packet(stream, StatusResponse::Pong { timestamp })?;
                }
            },
        }
    }

    Ok(())
}
