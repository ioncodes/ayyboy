use crate::gameboy::Mode;
use crate::memory::mmu::Mmu;
use crate::video::palette::Palette;
use crate::video::sprite::Sprite;
use bitflags::bitflags;

bitflags! {
    #[derive(Clone)]
    pub struct TileAttributes: u8 {
        const PALETTE   = 0b0000_0111;
        const BANK      = 0b0000_1000;
        const FLIP_X    = 0b0010_0000;
        const FLIP_Y    = 0b0100_0000;
        const PRIORITY  = 0b1000_0000;
    }
}

#[derive(Clone)]
pub struct Tile {
    pub pixels: [[Palette; 8]; 8],
    pub attributes: TileAttributes,
}

impl Tile {
    pub fn from(mmu: &Mmu, address: u16, mode: &Mode, attributes: TileAttributes) -> Tile {
        let mut pixels = [[Palette::default(); 8]; 8];

        // This is a closure that reads from VRAM, taking into account
        // which bank to read from based on the tile map attributes
        let read_from_vram = |addr: u16| -> u8 {
            if attributes.contains(TileAttributes::BANK) {
                mmu.read_from_vram(addr, 1)
            } else {
                mmu.read_from_vram(addr, 0)
            }
        };

        for y in 0..8 {
            let lsb = read_from_vram(address + (y * 2));
            let msb = read_from_vram((address + (y * 2)) + 1);

            for x in 0..8 {
                let lsb_bit = (lsb >> (7 - x)) & 0b0000_0001;
                let msb_bit = (msb >> (7 - x)) & 0b0000_0001;
                let color = (msb_bit << 1) | lsb_bit;

                pixels[y as usize][x as usize] =
                    Palette::from_background(color, mmu, mode, &attributes);
            }
        }

        Tile { pixels, attributes }
    }

    pub fn from_sprite(
        mmu: &Mmu, address: u16, sprite: &Sprite, mode: &Mode, attributes: TileAttributes,
    ) -> Tile {
        let mut pixels = [[Palette::default(); 8]; 8];

        // This is a closure that reads from VRAM, taking into account
        // which bank to read from based on the tile map attributes
        let read_from_vram = |addr: u16| -> u8 {
            if attributes.contains(TileAttributes::BANK) {
                mmu.read_from_vram(addr, 1)
            } else {
                mmu.read_from_vram(addr, 0)
            }
        };

        for y in 0..8 {
            let lsb = read_from_vram(address + (y * 2));
            let msb = read_from_vram((address + (y * 2)) + 1);

            for x in 0..8 {
                let lsb_bit = (lsb >> (7 - x)) & 0b0000_0001;
                let msb_bit = (msb >> (7 - x)) & 0b0000_0001;
                let color = (msb_bit << 1) | lsb_bit;

                pixels[y as usize][x as usize] =
                    Palette::from_object(color, mmu, sprite, true, mode, &attributes);
            }
        }

        Tile { pixels, attributes }
    }
}

impl Default for Tile {
    fn default() -> Tile {
        Tile {
            pixels: [[Palette::default(); 8]; 8],
            attributes: TileAttributes::empty(),
        }
    }
}
