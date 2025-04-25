use std::{io, time::Duration};

use snafu::prelude::*;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::packets::{
    configuration,
    deserialize::{self, Deserialize, Deserializer},
    handshake, login, play,
    serialize::{Serialize, Serializer},
    status,
};

pub struct Connection {
    stream: TcpStream,
    recv_buf: Vec<u8>,
    state: State,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Handshake,
    Status,
    Login,
    Configuration,
    Play,
}

pub trait ClientboundPacket: Serialize {
    fn expected_state(&self) -> State;
}

#[derive(Debug)]
pub enum ServerboundPacket {
    Handshake(handshake::serverbound::Packet),
    Status(status::serverbound::Packet),
    Login(login::serverbound::Packet),
    Configuration(configuration::serverbound::Packet),
    Play(play::serverbound::Packet),
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Client timed out"))]
    ClientTimedOut,
    #[snafu(transparent)]
    DeserializeError { source: deserialize::Error },
    #[snafu(transparent)]
    IOError { source: io::Error },
}

const RECV_TIMEOUT: Duration = Duration::from_secs(10);

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Connection {
            stream,
            recv_buf: Vec::new(),
            state: State::Handshake,
        }
    }

    async fn send_raw(&mut self, packet: &[u8]) -> Result<(), Error> {
        let mut s = Serializer::new();
        s.serialize_prefixed_byte_array(packet);
        let write_buf = s.finish();

        self.stream.write_all(&write_buf).await?;

        Ok(())
    }

    async fn recv_raw(&mut self) -> Result<Vec<u8>, Error> {
        loop {
            let mut d = Deserializer::new(&self.recv_buf);
            if let Ok(packet) = d.deserialize_prefixed_byte_array() {
                let packet = packet.to_owned();

                let consumed = self.recv_buf.len() - d.take_remaining().len();
                self.recv_buf.drain(..consumed);

                return Ok(packet);
            }

            match tokio::time::timeout(RECV_TIMEOUT, self.stream.read_buf(&mut self.recv_buf)).await
            {
                Err(_) => ClientTimedOutSnafu.fail()?,
                Ok(Err(e)) => return Err(e.into()),
                Ok(Ok(_)) => {}
            }
        }
    }

    pub async fn send(&mut self, packet: impl ClientboundPacket) -> Result<(), Error> {
        assert_eq!(
            self.state,
            packet.expected_state(),
            "Invalid state for packet",
        );

        let mut s = Serializer::new();
        packet.serialize(&mut s);
        let raw = s.finish();

        self.send_raw(&raw).await?;

        Ok(())
    }

    pub async fn recv(&mut self) -> Result<ServerboundPacket, Error> {
        let raw = self.recv_raw().await?;

        match self.state {
            State::Handshake => {
                let packet = deserialize::<handshake::serverbound::Packet>(&raw)?;

                match &packet {
                    handshake::serverbound::Packet::Intention { next_state, .. } => {
                        self.state = match next_state {
                            handshake::serverbound::NextState::Status => State::Status,
                            handshake::serverbound::NextState::Login => State::Login,
                            handshake::serverbound::NextState::Transfer => todo!(),
                        };
                    }
                }

                Ok(ServerboundPacket::Handshake(packet))
            }
            State::Status => {
                let packet = deserialize::<status::serverbound::Packet>(&raw)?;

                Ok(ServerboundPacket::Status(packet))
            }
            State::Login => {
                let packet = deserialize::<login::serverbound::Packet>(&raw)?;

                match &packet {
                    login::serverbound::Packet::LoginAcknowledged => {
                        self.state = State::Configuration;
                    }
                    _ => {}
                }

                Ok(ServerboundPacket::Login(packet))
            }
            State::Configuration => {
                let packet = deserialize::<configuration::serverbound::Packet>(&raw)?;

                match &packet {
                    configuration::serverbound::Packet::FinishConfiguration => {
                        self.state = State::Play;
                    }
                    _ => {}
                }

                Ok(ServerboundPacket::Configuration(packet))
            }
            State::Play => {
                let packet = deserialize::<play::serverbound::Packet>(&raw)?;

                Ok(ServerboundPacket::Play(packet))
            }
        }
    }

    pub async fn disconnect(mut self, reason: &str) -> Result<(), Error> {
        match self.state {
            State::Handshake | State::Status => {}
            State::Login => {
                self.send(login::clientbound::Packet::LoginDisconnect { reason })
                    .await?;
            }
            State::Configuration => {
                self.send(configuration::clientbound::Packet::Disconnect { reason })
                    .await?;
            }
            State::Play => {
                self.send(play::clientbound::Packet::Disconnect { reason })
                    .await?;
            }
        }
        Ok(())
    }
}

fn deserialize<'de, T: Deserialize<'de>>(bytes: &'de [u8]) -> Result<T, deserialize::Error> {
    let mut d = Deserializer::new(bytes);
    let result = T::deserialize(&mut d)?;
    d.finish()?;
    Ok(result)
}

impl ClientboundPacket for status::clientbound::Packet<'_> {
    #[allow(private_interfaces)]
    fn expected_state(&self) -> State {
        State::Status
    }
}

impl ClientboundPacket for login::clientbound::Packet<'_> {
    #[allow(private_interfaces)]
    fn expected_state(&self) -> State {
        State::Login
    }
}

impl ClientboundPacket for configuration::clientbound::Packet<'_> {
    #[allow(private_interfaces)]
    fn expected_state(&self) -> State {
        State::Configuration
    }
}

impl ClientboundPacket for play::clientbound::Packet<'_> {
    #[allow(private_interfaces)]
    fn expected_state(&self) -> State {
        State::Play
    }
}
