use crate::{
    packets::serialize::{Serialize, Serializer},
    registries::{self, BlockState},
};

#[derive(Debug)]
pub struct Chunk {
    subchunks: Vec<Subchunk>,
}

#[derive(Debug)]
struct Subchunk {
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

impl Chunk {
    pub fn empty(subchunk_count: u8) -> Self {
        Self {
            subchunks: (0..subchunk_count).map(|_| Subchunk::empty()).collect(),
        }
    }

    pub fn demo(subchunk_count: u8) -> Self {
        let mut chunk = Self::empty(subchunk_count);
        chunk.set_block(7, 7, 15, Block::StairsWestTop);
        chunk.set_block(7, 8, 15, Block::StairsWestBottom);
        chunk.set_block(8, 7, 15, Block::StairsEastTop);
        chunk.set_block(8, 8, 15, Block::StairsEastBottom);
        chunk
    }

    #[inline]
    pub fn set_block(&mut self, x: u8, y: u16, z: u8, block: Block) {
        let subchunk = &mut self.subchunks[(y / 16) as usize];
        subchunk.set_block(x, (y % 16) as u8, z, block);
    }

    #[inline]
    pub fn block(&self, x: u8, y: u16, z: u8) -> Block {
        let subchunk = &self.subchunks[(y / 16) as usize];
        subchunk.block(x, (y % 16) as u8, z)
    }

    pub fn chunk_data(&self) -> Vec<u8> {
        let mut s = Serializer::new();

        for subchunk in &self.subchunks {
            subchunk.chunk_data(&mut s);
        }

        s.finish()
    }
}

impl Subchunk {
    fn empty() -> Self {
        Self {
            blocks: vec![Block::Air; 16 * 16 * 16],
        }
    }

    #[inline]
    fn block_index(&self, x: u8, y: u8, z: u8) -> usize {
        (y as usize * 16 + z as usize) * 16 + x as usize
    }

    #[inline]
    fn set_block(&mut self, x: u8, y: u8, z: u8, block: Block) {
        let index = self.block_index(x, y, z);
        self.blocks[index] = block;
    }

    #[inline]
    fn block(&self, x: u8, y: u8, z: u8) -> Block {
        let index = self.block_index(x, y, z);
        self.blocks[index]
    }

    fn chunk_data(&self, s: &mut Serializer) {
        // Block count
        let block_count = self.blocks.iter().filter(|&&b| b != Block::Air).count() as i16;
        s.serialize_short(block_count);

        // Block states
        // https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Chunk_Format?oldid=2845070#Paletted_Container_structure
        s.serialize_ubyte(4); // Bits per entry
        s.serialize_prefixed_array(&[
            registries::block_state("minecraft:air", &[]).unwrap(),
            registries::block_state("minecraft:gray_concrete", &[]).unwrap(),
            registries::block_state(
                "minecraft:deepslate_tile_stairs",
                &[
                    ("facing", "west"),
                    ("half", "top"),
                    ("shape", "straight"),
                    ("waterlogged", "false"),
                ],
            )
            .unwrap(),
            registries::block_state(
                "minecraft:deepslate_tile_stairs",
                &[
                    ("facing", "west"),
                    ("half", "bottom"),
                    ("shape", "straight"),
                    ("waterlogged", "false"),
                ],
            )
            .unwrap(),
            registries::block_state(
                "minecraft:deepslate_tile_stairs",
                &[
                    ("facing", "east"),
                    ("half", "top"),
                    ("shape", "straight"),
                    ("waterlogged", "false"),
                ],
            )
            .unwrap(),
            registries::block_state(
                "minecraft:deepslate_tile_stairs",
                &[
                    ("facing", "east"),
                    ("half", "bottom"),
                    ("shape", "straight"),
                    ("waterlogged", "false"),
                ],
            )
            .unwrap(),
        ]); // Pallete
        s.serialize_varint(16 * 16 * 16 / 2 / 8); // Length in i64s
        for chunk in self.blocks.chunks(2) {
            let x = chunk[0] as u8 & 0x0F;
            let y = chunk[1] as u8 & 0x0F;
            let b = (x << 4) | y;
            s.serialize_ubyte(b);
        }

        // Biomes
        s.serialize_byte_array(&[0x00, 0x00, 0x00]);
    }
}

impl Serialize for &BlockState {
    fn serialize(&self, s: &mut Serializer) {
        s.serialize_varint(self.id);
    }
}
