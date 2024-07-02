use crate::memory::mmu::Mmu;
use crate::video::palette::Palette;
use crate::video::sprite::{Sprite, SpriteAttributes};

#[derive(Debug, Copy, Clone)]
pub struct Tile {
    pub pixels: [[Palette; 8]; 8],
}

impl Tile {
    pub fn from_background_addr(mmu: &Mmu, address: u16, is_window: bool) -> Tile {
        let mut pixels = [[Palette::default(); 8]; 8];

        for y in 0..8 {
            let lsb = mmu.read_unchecked(address + (y * 2));
            let msb = mmu.read_unchecked((address + (y * 2)) + 1);

            for x in 0..8 {
                let lsb_bit = (lsb >> (7 - x)) & 0b0000_0001;
                let msb_bit = (msb >> (7 - x)) & 0b0000_0001;
                let color = (msb_bit << 1) | lsb_bit;

                pixels[y as usize][x as usize] = Palette::from_background(color, mmu, is_window);
            }
        }

        Tile { pixels }
    }

    pub fn from_sprite_addr(mmu: &Mmu, address: u16, sprite: &Sprite) -> Tile {
        let mut pixels = [[Palette::default(); 8]; 8];

        for y in 0..8 {
            let lsb = mmu.read_unchecked(address + (y * 2));
            let msb = mmu.read_unchecked((address + (y * 2)) + 1);

            for x in 0..8 {
                let lsb_bit = (lsb >> (7 - x)) & 0b0000_0001;
                let msb_bit = (msb >> (7 - x)) & 0b0000_0001;
                let color = (msb_bit << 1) | lsb_bit;

                pixels[y as usize][x as usize] = Palette::from_object(color, mmu, sprite, true);
            }
        }

        if sprite.attributes.contains(SpriteAttributes::FLIP_X) {
            pixels.iter_mut().for_each(|row| row.reverse());
        }

        if sprite.attributes.contains(SpriteAttributes::FLIP_Y) {
            pixels.reverse();
        }

        Tile { pixels }
    }
}

impl Default for Tile {
    fn default() -> Tile {
        Tile {
            pixels: [[Palette::default(); 8]; 8],
        }
    }
}
