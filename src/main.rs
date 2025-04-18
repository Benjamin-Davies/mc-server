use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    str, thread,
};

use anyhow::Context;

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

    while let Ok(packet) = read_prefixed_byte_array(stream) {
        let (packet_id, rest) = parse_varint(&packet)?;

        match state {
            State::Handshaking => match packet_id {
                // Handshake
                0x00 => {
                    let (protocol_version, rest) = parse_varint(rest)?;
                    let (server_address, rest) = parse_string(rest)?;
                    let (server_port, rest) = parse_ushort(rest)?;
                    let (next_state, _rest) = parse_varint(rest)?;
                    dbg!(protocol_version, server_address, server_port);

                    match next_state {
                        1 => state = State::Status,
                        _ => todo!("Next state after handshake: {next_state}"),
                    }
                }
                _ => todo!("Packet ID (Handshaking): {packet_id}"),
            },
            State::Status => match packet_id {
                // Status
                0x00 => {
                    let json = r#"{
                        "version": {
                            "name": "1.21.2",
                            "protocol": 768
                        },
                        "players": {
                            "max": 100,
                            "online": 5,
                            "sample": [
                                {
                                    "name": "thinkofdeath",
                                    "id": "4566e69f-c907-48ee-8d71-d7ba5aa00d20"
                                }
                            ]
                        },
                        "description": {
                            "text": "Hello, world!"
                        },
                        "favicon": "data:image/png;base64,<data>",
                        "enforcesSecureChat": false
                    }"#;

                    let mut packet = Vec::new();
                    write_varint(&mut packet, 0x00);
                    write_string(&mut packet, json);

                    let mut buf = Vec::new();
                    write_prefixed_byte_array(&mut buf, &packet);

                    stream.write_all(&buf)?;
                }
                // Ping
                0x01 => {
                    let mut buf = vec![0x00, 0x01];
                    buf.extend_from_slice(rest);
                    stream.write_all(&buf)?;
                }
                _ => todo!("Packet ID (Status): {packet_id}"),
            },
        }
    }

    Ok(())
}

fn read_prefixed_byte_array(stream: &mut impl Read) -> Result<Vec<u8>, anyhow::Error> {
    let len = read_varint(stream)?;
    let mut buf = vec![0; len as usize];
    stream.read_exact(&mut buf)?;
    Ok(buf)
}

fn read_varint(stream: &mut impl Read) -> anyhow::Result<u32> {
    let mut n = 0;
    let mut shift = 0;

    for _ in 0..5 {
        let mut buf = [0; 1];
        stream.read_exact(&mut buf)?;

        let [byte] = buf;
        n |= ((byte & 0x7F) as u32) << shift;
        shift += 7;

        if byte & 0x80 == 0 {
            break;
        }
    }

    Ok(n)
}

fn parse_varint(bytes: &[u8]) -> anyhow::Result<(u32, &[u8])> {
    let mut n = 0;
    let mut shift = 0;
    let mut i = 0;

    while i < 5 {
        let byte = bytes.get(i).context("Invalid varint")?;
        n |= ((byte & 0x7F) as u32) << shift;
        shift += 7;
        i += 1;

        if byte & 0x80 == 0 {
            break;
        }
    }

    Ok((n, &bytes[i..]))
}

fn parse_string(bytes: &[u8]) -> anyhow::Result<(&str, &[u8])> {
    let (len, rest) = parse_varint(bytes)?;
    let s = str::from_utf8(&rest[..len as usize])?;
    Ok((s, &rest[len as usize..]))
}

fn parse_ushort(bytes: &[u8]) -> anyhow::Result<(u16, &[u8])> {
    let (buf, rest) = parse_exact_byte_array(bytes).context("Invalid ushort")?;
    let n = u16::from_be_bytes(buf);
    Ok((n, rest))
}

fn parse_exact_byte_array<const N: usize>(bytes: &[u8]) -> Option<([u8; N], &[u8])> {
    if bytes.len() >= N {
        let (head, tail) = bytes.split_at(N);
        Some((head.try_into().unwrap(), tail))
    } else {
        None
    }
}

fn write_prefixed_byte_array(buf: &mut Vec<u8>, data: &[u8]) {
    let len = data.len() as u32;
    write_varint(buf, len);
    buf.extend_from_slice(data);
}

fn write_string(buf: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    write_prefixed_byte_array(buf, bytes);
}

fn write_varint(buf: &mut Vec<u8>, n: u32) {
    let mut n = n;
    loop {
        let mut byte = (n & 0x7F) as u8;
        n >>= 7;
        if n != 0 {
            byte |= 0x80;
        }
        buf.push(byte);
        if n == 0 {
            break;
        }
    }
}
