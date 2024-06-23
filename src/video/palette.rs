use crate::memory::mmu::Mmu;
use crate::video::ppu::BG_PALETTE_REGISTER;
use sdl2::pixels::Color;

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

        // let shade = match value {
        //     0b00 => bgp_shade & 0b0000_0011,
        //     0b01 => bgp_shade & 0b0000_1100 >> 2,
        //     0b10 => bgp_shade & 0b0011_0000 >> 4,
        //     0b11 => bgp_shade & 0b1100_0000 >> 6,
        //     _ => panic!("Invalid color value: {}", value),
        // };
        //
        // match shade {
        //     0 => Palette::White,
        //     1 => Palette::LightGray,
        //     2 => Palette::DarkGray,
        //     3 => Palette::Black,
        //     _ => panic!("Invalid shade value: {}", shade),
        // }

        //
        // let shade3 = bgp_shade & 0b1100_0000 >> 6;
        // let shade2 = bgp_shade & 0b0011_0000 >> 4;
        // let shade1 = bgp_shade & 0b0000_1100 >> 2;
        // let shade0 = bgp_shade & 0b0000_0011;
        //
        // let shade = match value {
        //     0b00 => shade0,
        //     0b01 => shade1,
        //     0b10 => shade2,
        //     0b11 => shade3,
        //     _ => panic!("Invalid color value: {}", value),
        // };
        //
        // match shade {
        //     0 => Palette::White,
        //     1 => Palette::LightGray,
        //     2 => Palette::DarkGray,
        //     3 => Palette::Black,
        //     _ => panic!("Invalid shade value: {}", shade),
        // }
        //
        match value {
            0 => Palette::White,
            1 => Palette::Black,
            _ => panic!("Invalid color value: {}", value),
        }
    }
}

impl Into<Color> for Palette {
    fn into(self) -> Color {
        match self {
            Palette::White => Color::RGB(0xe0, 0xf0, 0xe7),
            Palette::LightGray => Color::RGB(0x8b, 0xa3, 0x94),
            Palette::DarkGray => Color::RGB(0x55, 0x64, 0x5a),
            Palette::Black => Color::RGB(0x34, 0x3d, 0x37),
        }
    }
}

impl Default for Palette {
    fn default() -> Palette {
        Palette::White
    }
}
