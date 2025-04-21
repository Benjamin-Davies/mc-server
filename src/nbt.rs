/// https://minecraft.wiki/w/NBT_format#Binary_format
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
#[allow(dead_code)]
pub enum Kind {
    End = 0,
    Byte = 1,
    Short = 2,
    Int = 3,
    Long = 4,
    Float = 5,
    Double = 6,
    ByteArray = 7,
    String = 8,
    List = 9,
    Compound = 10,
    IntArray = 11,
    LongArray = 12,
}

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum Tag {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<u8>),
    String(String),
    List(Vec<Tag>),
    Compound(Vec<(String, Tag)>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl Tag {
    pub fn kind(&self) -> Kind {
        match self {
            Tag::End => Kind::End,
            Tag::Byte(_) => Kind::Byte,
            Tag::Short(_) => Kind::Short,
            Tag::Int(_) => Kind::Int,
            Tag::Long(_) => Kind::Long,
            Tag::Float(_) => Kind::Float,
            Tag::Double(_) => Kind::Double,
            Tag::ByteArray(_) => Kind::ByteArray,
            Tag::String(_) => Kind::String,
            Tag::List(_) => Kind::List,
            Tag::Compound(_) => Kind::Compound,
            Tag::IntArray(_) => Kind::IntArray,
            Tag::LongArray(_) => Kind::LongArray,
        }
    }
}

impl From<bool> for Tag {
    fn from(value: bool) -> Self {
        Tag::Byte(if value { 1 } else { 0 })
    }
}

impl From<i8> for Tag {
    fn from(value: i8) -> Self {
        Tag::Byte(value)
    }
}

impl From<i16> for Tag {
    fn from(value: i16) -> Self {
        Tag::Short(value)
    }
}

impl From<i32> for Tag {
    fn from(value: i32) -> Self {
        Tag::Int(value)
    }
}

impl From<i64> for Tag {
    fn from(value: i64) -> Self {
        Tag::Long(value)
    }
}

impl From<f32> for Tag {
    fn from(value: f32) -> Self {
        Tag::Float(value)
    }
}

impl From<f64> for Tag {
    fn from(value: f64) -> Self {
        Tag::Double(value)
    }
}

impl From<&str> for Tag {
    fn from(value: &str) -> Self {
        Tag::String(value.to_owned())
    }
}

macro_rules! nbt {
    ( $( () )? ) => {
        $crate::nbt::Tag::End
    };
    ( $value:literal ) => {
        $crate::nbt::Tag::from($value)
    };
    ( ( - $value:literal ) ) => {
        $crate::nbt::Tag::from(-$value)
    };
    ({ $( $key:ident : $value:tt ),* $( , )? }) => {
        $crate::nbt::Tag::Compound(vec![ $( (stringify!($key).to_owned(), nbt!($value) ) ),* ])
    };
}
