use crate::{packets::serialize::Serializer, registries};

#[derive(Debug)]
pub struct Subchunk {
    blocks: Vec<Block>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Block {
    Air,
    Stone,
    OakStairs,
    PistonHeadUp,
    PistonHeadDown,
}

impl Subchunk {
    pub fn empty() -> Self {
        Self {
            blocks: vec![Block::Air; 16 * 16 * 16],
        }
    }

    pub fn demo() -> Self {
        let mut subchunk = Self::empty();
        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    if (x as f32 - 7.5).powi(2)
                        + (y as f32 - 7.5).powi(2)
                        + (z as f32 - 7.5).powi(2)
                        > 7f32.powi(2)
                    {
                        let block = if y % 2 == 0 {
                            Block::PistonHeadDown
                        } else {
                            Block::PistonHeadUp
                        };
                        subchunk.set_block(x, y, z, block);
                    }
                }
            }
        }
        subchunk
    }

    #[inline]
    fn block_index(&self, x: u8, y: u8, z: u8) -> usize {
        (y as usize * 16 + z as usize) * 16 + x as usize
    }

    #[inline]
    pub fn set_block(&mut self, x: u8, y: u8, z: u8, block: Block) {
        let index = self.block_index(x, y, z);
        self.blocks[index] = block;
    }

    #[inline]
    pub fn block(&self, x: u8, y: u8, z: u8) -> Block {
        let index = self.block_index(x, y, z);
        self.blocks[index]
    }

    pub fn chunk_data(&self) -> anyhow::Result<Vec<u8>> {
        let mut s = Serializer::new();

        // Block count
        let block_count = self.blocks.iter().filter(|&&b| b != Block::Air).count() as i16;
        s.serialize_short(block_count)?;

        // Block states
        // https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Chunk_Format?oldid=2845070#Paletted_Container_structure
        s.serialize_ubyte(4)?; // Bytes per entry
        s.serialize_prefixed_array(
            &[
                0,
                1,
                registries::block_state(
                    "minecraft:oak_stairs",
                    &[
                        ("facing", "east"),
                        ("half", "top"),
                        ("shape", "straight"),
                        ("waterlogged", "false"),
                    ],
                )?
                .id,
                registries::block_state(
                    "minecraft:piston_head",
                    &[("facing", "up"), ("short", "true"), ("type", "normal")],
                )?
                .id,
                registries::block_state(
                    "minecraft:piston_head",
                    &[("facing", "down"), ("short", "true"), ("type", "normal")],
                )?
                .id,
            ],
            |s, b| s.serialize_varint(*b),
        )?; // Pallete
        s.serialize_varint(16 * 16 * 16 / 2 / 8)?; // Length in i64s
        for chunk in self.blocks.chunks(2) {
            let x = chunk[0] as u8 & 0x0F;
            let y = chunk[1] as u8 & 0x0F;
            let b = (x << 4) | y;
            s.serialize_ubyte(b)?;
        }

        // Biomes
        s.serialize_byte_array(&[0x00, 0x00, 0x00])?;

        Ok(s.finish())
    }
}
