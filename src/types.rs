pub type HandshakeRequest<'a> = net::packets::handshake::serverbound::Packet<'a>;

pub type HandshakeRequestNextState = net::packets::handshake::serverbound::NextState;

pub type StatusRequest = net::packets::status::serverbound::Packet;
pub type StatusResponse<'a> = net::packets::status::clientbound::Packet<'a>;

pub type Status<'a> = net::packets::status::clientbound::Status<'a>;
pub type Version<'a> = net::packets::status::clientbound::Version<'a>;
pub type Players = net::packets::status::clientbound::Players;
pub type TextComponent<'a> = net::packets::status::clientbound::TextComponent<'a>;

pub type LoginRequest<'a> = net::packets::login::serverbound::Packet<'a>;
pub type LoginResponse<'a> = net::packets::login::clientbound::Packet<'a>;

pub type ConfigurationRequest<'a> = net::packets::configuration::serverbound::Packet<'a>;
pub type ConfigurationResponse<'a> = net::packets::configuration::clientbound::Packet<'a>;

pub type PlayRequest<'a> = net::packets::play::serverbound::Packet<'a>;
pub type PlayResponse = net::packets::play::clientbound::Packet;

pub type GameEvent = net::packets::play::clientbound::GameEvent;
