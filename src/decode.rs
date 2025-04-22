use net::packets::deserialize::{Deserialize, Deserializer};

pub trait Decode<'a>: Sized {
    fn decode(bytes: &'a [u8]) -> anyhow::Result<Self>;
}

pub trait Parse {
    fn parse<'a, T: Decode<'a>>(&'a self) -> anyhow::Result<T>;
}

impl<'a, D: Deserialize<'a>> Decode<'a> for D {
    fn decode(bytes: &'a [u8]) -> anyhow::Result<Self> {
        let mut d = Deserializer::new(bytes);
        let req = Self::deserialize(&mut d)?;
        d.finish()?;
        Ok(req)
    }
}

impl Parse for [u8] {
    fn parse<'a, T: Decode<'a>>(&'a self) -> anyhow::Result<T> {
        T::decode(self)
    }
}
