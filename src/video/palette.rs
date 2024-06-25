use crate::memory::mmu::Mmu;
use crate::video::ppu::BG_PALETTE_REGISTER;

pub type Color = [u8; 3];

#[derive(Debug, Clone, Copy)]
pub enum Palette {
    White,
    LightGray,
    DarkGray,
    Black,
}

impl Palette {
    pub fn from(value: u8, mmu: &Mmu) -> Palette {
        let bgp_shade = mmu.read(BG_PALETTE_REGISTER);

        let shade = match value {
            0b00 => bgp_shade & 0b0000_0011,
            0b01 => (bgp_shade & 0b0000_1100) >> 2,
            0b10 => (bgp_shade & 0b0011_0000) >> 4,
            0b11 => (bgp_shade & 0b1100_0000) >> 6,
            _ => panic!("Invalid color value: {}", value),
        };

        match shade {
            0b00 => Palette::White,
            0b01 => Palette::LightGray,
            0b10 => Palette::DarkGray,
            0b11 => Palette::Black,
            _ => panic!("Invalid shade value: {}", shade),
        }
    }
}

impl Into<Color> for Palette {
    fn into(self) -> Color {
        match self {
            Palette::White => [0xe0, 0xf0, 0xe7],
            Palette::LightGray => [0x8b, 0xa3, 0x94],
            Palette::DarkGray => [0x55, 0x64, 0x5a],
            Palette::Black => [0x34, 0x3d, 0x37],
        }
    }
}

impl Default for Palette {
    fn default() -> Palette {
        Palette::White
    }
}
