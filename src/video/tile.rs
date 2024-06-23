use crate::memory::mmu::Mmu;
use crate::video::palette::Palette;

#[derive(Debug, Clone)]
pub struct Tile {
    pub pixels: [[Palette; 8]; 8],
}

impl Tile {
    pub fn from_addr(mmu: &Mmu, address: u16) -> Tile {
        let mut pixels = [[Palette::default(); 8]; 8];

        for y in 0..8 {
            let lsb = mmu.read(address + (y * 2));
            let msb = mmu.read((address + (y * 2)) + 1);

            for x in 0..8 {
                let lsb_bit = (lsb >> (7 - x)) & 0b0000_0001;
                let msb_bit = (msb >> (7 - x)) & 0b0000_0001;
                let color = (msb_bit << 1) | lsb_bit;

                pixels[y as usize][x as usize] = Palette::from(color, mmu);
            }
        }

        Tile { pixels }
    }
}
