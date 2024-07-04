use crate::memory::mmu::Mmu;
use crate::video::OAM_ADDRESS;
use bitflags::bitflags;

bitflags! {
    #[derive(Clone)]
    pub struct SpriteAttributes: u8 {
        // CGB ONLY FLAGS HERE
        const PALETTE = 0b0001_0000;
        const FLIP_X = 0b0010_0000;
        const FLIP_Y = 0b0100_0000;
        const PRIORITY = 0b1000_0000;
    }
}

#[derive(Clone)]
pub struct Sprite {
    pub x: u8,
    pub y: u8,
    pub tile_index: u8,
    pub attributes: SpriteAttributes,
    pub oam_addr: u16,
}

impl Sprite {
    pub fn from_oam(mmu: &Mmu, index: u16) -> Self {
        let sprite_addr = OAM_ADDRESS + (index * 4);

        Sprite {
            y: mmu.read_unchecked(sprite_addr),
            x: mmu.read_unchecked(sprite_addr + 1),
            tile_index: mmu.read_unchecked(sprite_addr + 2),
            attributes: SpriteAttributes::from_bits_truncate(mmu.read_unchecked(sprite_addr + 3)),
            oam_addr: sprite_addr,
        }
    }
}
