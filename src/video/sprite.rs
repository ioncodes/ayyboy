use crate::memory::mmu::Mmu;
use crate::video::ppu::OAM_ADDRESS;

pub struct Sprite {
    pub x: u8,
    pub y: u8,
    pub tile_index: u8,
    pub attributes: u8,
}

impl Sprite {
    pub fn from_oam(mmu: &Mmu, index: u16) -> Self {
        let sprite_addr = OAM_ADDRESS + (index * 4);

        Sprite {
            y: mmu.read_unchecked(sprite_addr),
            x: mmu.read_unchecked(sprite_addr + 1),
            tile_index: mmu.read_unchecked(sprite_addr + 2),
            attributes: mmu.read_unchecked(sprite_addr + 3),
        }
    }

    pub fn is_visible_on_scanline(&self, scanline: usize) -> bool {
        let sprite_y = self.y.wrapping_sub(16);
        let height = 8;
        scanline >= sprite_y as usize && scanline < (sprite_y as usize + height)
    }
}
