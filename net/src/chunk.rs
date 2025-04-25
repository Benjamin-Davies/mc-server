use crate::{packets::serialize::Serializer, registries};

#[derive(Debug)]
pub struct Subchunk {
    blocks: Vec<Block>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Block {
    Air,
    GrayConcrete,
    StairsWestTop,
    StairsWestBottom,
    StairsEastTop,
    StairsEastBottom,
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
            for z in 0..16 {
                subchunk.set_block(x, 0, z, Block::GrayConcrete);
            }
        }
        subchunk.set_block(7, 7, 15, Block::StairsWestTop);
        subchunk.set_block(7, 8, 15, Block::StairsWestBottom);
        subchunk.set_block(8, 7, 15, Block::StairsEastTop);
        subchunk.set_block(8, 8, 15, Block::StairsEastBottom);
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

    pub fn chunk_data(&self) -> Vec<u8> {
        let mut s = Serializer::new();

        // Block count
        let block_count = self.blocks.iter().filter(|&&b| b != Block::Air).count() as i16;
        s.serialize_short(block_count);

        // Block states
        // https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Chunk_Format?oldid=2845070#Paletted_Container_structure
        s.serialize_ubyte(4); // Bits per entry
        s.serialize_prefixed_array(
            &[
                0,
                registries::block_state("minecraft:gray_concrete", &[])
                    .unwrap()
                    .id,
                registries::block_state(
                    "minecraft:deepslate_tile_stairs",
                    &[
                        ("facing", "west"),
                        ("half", "top"),
                        ("shape", "straight"),
                        ("waterlogged", "false"),
                    ],
                )
                .unwrap()
                .id,
                registries::block_state(
                    "minecraft:deepslate_tile_stairs",
                    &[
                        ("facing", "west"),
                        ("half", "bottom"),
                        ("shape", "straight"),
                        ("waterlogged", "false"),
                    ],
                )
                .unwrap()
                .id,
                registries::block_state(
                    "minecraft:deepslate_tile_stairs",
                    &[
                        ("facing", "east"),
                        ("half", "top"),
                        ("shape", "straight"),
                        ("waterlogged", "false"),
                    ],
                )
                .unwrap()
                .id,
                registries::block_state(
                    "minecraft:deepslate_tile_stairs",
                    &[
                        ("facing", "east"),
                        ("half", "bottom"),
                        ("shape", "straight"),
                        ("waterlogged", "false"),
                    ],
                )
                .unwrap()
                .id,
            ],
            |s, b| s.serialize_varint(*b),
        ); // Pallete
        s.serialize_varint(16 * 16 * 16 / 2 / 8); // Length in i64s
        for chunk in self.blocks.chunks(2) {
            let x = chunk[0] as u8 & 0x0F;
            let y = chunk[1] as u8 & 0x0F;
            let b = (x << 4) | y;
            s.serialize_ubyte(b);
        }

        // Biomes
        s.serialize_byte_array(&[0x00, 0x00, 0x00]);

        s.finish()
    }
}
