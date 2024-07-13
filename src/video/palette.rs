use crate::gameboy::Mode;
use crate::memory::mmu::Mmu;
use crate::video::sprite::{Sprite, SpriteAttributes};
use crate::video::{BG_PALETTE_REGISTER, OBJ0_PALETTE_REGISTER, OBJ1_PALETTE_REGISTER};

use super::tile::TileAttributes;

pub type Color = [u8; 3];

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Palette {
    White(u8),
    LightGray(u8),
    DarkGray(u8),
    Black(u8),
    Transparent(u8),
    Color(u8, u8, u8, u8),
}

impl Palette {
    pub fn from_background(
        value: u8, mmu: &Mmu, mode: &Mode, attributes: &TileAttributes,
    ) -> Palette {
        if *mode == Mode::Dmg {
            let bgp_shade = mmu.read_unchecked(BG_PALETTE_REGISTER);

            let shade = match value {
                0b00 => bgp_shade & 0b0000_0011, // shouldn't happen for window layer, only background
                0b01 => (bgp_shade & 0b0000_1100) >> 2,
                0b10 => (bgp_shade & 0b0011_0000) >> 4,
                0b11 => (bgp_shade & 0b1100_0000) >> 6,
                _ => panic!("Invalid color value: {}", value),
            };

            match shade {
                0b00 => Palette::White(value),
                0b01 => Palette::LightGray(value),
                0b10 => Palette::DarkGray(value),
                0b11 => Palette::Black(value),
                _ => panic!("Invalid shade value: {}", shade),
            }
        } else {
            let palette = (attributes.bits() & TileAttributes::PALETTE.bits()) as u8;

            let color = match value {
                0b00 => mmu.cgb_cram.fetch_bg(palette, 0),
                0b01 => mmu.cgb_cram.fetch_bg(palette, 2),
                0b10 => mmu.cgb_cram.fetch_bg(palette, 4),
                0b11 => mmu.cgb_cram.fetch_bg(palette, 6),
                _ => panic!("Invalid color value: {}", value),
            }
            .reverse_bits();

            let r = ((color & 0b1111_1000_0000_0000) >> 11) as u8;
            let g = ((color & 0b0000_0111_1100_0000) >> 6) as u8;
            let b = ((color & 0b0000_0000_0011_1110) >> 1) as u8;

            Palette::Color(
                value,
                (r << 3) | (r >> 2),
                (g << 3) | (g >> 2),
                (b << 3) | (b >> 2),
            )
        }
    }

    pub fn from_object(
        value: u8, mmu: &Mmu, sprite: &Sprite, allow_transparency: bool, mode: &Mode,
    ) -> Palette {
        if allow_transparency && value == 0 {
            return Palette::Transparent(0);
        }

        if *mode == Mode::Dmg {
            let objp_shade = if !sprite.attributes.contains(SpriteAttributes::DMG_PALETTE) {
                mmu.read_unchecked(OBJ0_PALETTE_REGISTER)
            } else {
                mmu.read_unchecked(OBJ1_PALETTE_REGISTER)
            };

            let shade = match value {
                0b00 => objp_shade & 0b0000_0011, // this case should be handled above (transparent)
                0b01 => (objp_shade & 0b0000_1100) >> 2,
                0b10 => (objp_shade & 0b0011_0000) >> 4,
                0b11 => (objp_shade & 0b1100_0000) >> 6,
                _ => panic!("Invalid color value: {}", value),
            };

            match shade {
                0b00 => Palette::White(value),
                0b01 => Palette::LightGray(value),
                0b10 => Palette::DarkGray(value),
                0b11 => Palette::Black(value),
                _ => panic!("Invalid shade value: {}", shade),
            }
        } else {
            let palette = (sprite.attributes.bits() & SpriteAttributes::CGB_PALETTE.bits()) as u8;

            let color = match value {
                0b00 => mmu.cgb_cram.fetch_obj(palette, 0),
                0b01 => mmu.cgb_cram.fetch_obj(palette, 2),
                0b10 => mmu.cgb_cram.fetch_obj(palette, 4),
                0b11 => mmu.cgb_cram.fetch_obj(palette, 6),
                _ => panic!("Invalid color value: {}", value),
            }
            .reverse_bits();

            let r = ((color & 0b1111_1000_0000_0000) >> 11) as u8;
            let g = ((color & 0b0000_0111_1100_0000) >> 6) as u8;
            let b = ((color & 0b0000_0000_0011_1110) >> 1) as u8;

            Palette::Color(
                value,
                (r << 3) | (r >> 2),
                (g << 3) | (g >> 2),
                (b << 3) | (b >> 2),
            )
        }
    }

    pub fn is_transparent(&self) -> bool {
        *self == Palette::Transparent(0)
    }

    pub fn is_color(&self, index: u8) -> bool {
        match self {
            Palette::White(i) => *i == index,
            Palette::LightGray(i) => *i == index,
            Palette::DarkGray(i) => *i == index,
            Palette::Black(i) => *i == index,
            Palette::Transparent(i) => *i == index,
            Palette::Color(i, _, _, _) => *i == index,
        }
    }
}

impl Into<Color> for Palette {
    fn into(self) -> Color {
        match self {
            Palette::White(_) => [0xff, 0xff, 0xff],
            Palette::LightGray(_) => [0xaa, 0xaa, 0xaa],
            Palette::DarkGray(_) => [0x55, 0x55, 0x55],
            Palette::Black(_) => [0x00, 0x00, 0x00],
            Palette::Transparent(_) => [0x00, 0x00, 0x00],
            Palette::Color(_, r, g, b) => [r, g, b],
        }
    }
}

impl Default for Palette {
    fn default() -> Palette {
        Palette::White(0)
    }
}
