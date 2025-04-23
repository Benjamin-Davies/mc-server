use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::packets::{
    deserialize::Deserializer,
    serialize::{Serialize, Serializer},
};

pub struct Connection {
    stream: TcpStream,
    recv_buf: Vec<u8>,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Connection {
            stream,
            recv_buf: Vec::new(),
        }
    }

    pub async fn recv_raw(&mut self) -> anyhow::Result<Vec<u8>> {
        loop {
            let mut d = Deserializer::new(&self.recv_buf);
            if let Ok(packet) = d.deserialize_prefixed_byte_array() {
                let packet = packet.to_owned();

                let consumed = self.recv_buf.len() - d.take_remaining().len();
                self.recv_buf.drain(..consumed);

                return Ok(packet);
            }

            self.stream.read_buf(&mut self.recv_buf).await?;
        }
    }

    async fn send_raw(&mut self, packet: &[u8]) -> anyhow::Result<()> {
        let mut s = Serializer::new();
        s.serialize_prefixed_byte_array(packet)?;
        let buf = s.finish();

        self.stream.write_all(&buf).await?;
        self.stream.flush().await?;

        Ok(())
    }

    pub async fn send(&mut self, packet: impl Serialize) -> anyhow::Result<()> {
        let mut s = Serializer::new();
        packet.serialize(&mut s)?;
        let buf = s.finish();

        self.send_raw(&buf).await?;

        Ok(())
    }
}
