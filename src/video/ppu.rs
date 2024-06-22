use crate::memory::mmu::Mmu;

pub const BACKGROUND_WIDTH: usize = 256;
pub const BACKGROUND_HEIGHT: usize = 256;

const TILE_ADDRESS: u16 = 0x8000;
const BACKGROUND_0_ADDRESS: u16 = 0x9800;
const BACKGROUND_1_ADDRESS: u16 = 0x9c00;

const CONTROL_REGISTER: u16 = 0xff40;
const STATUS_REGISTER: u16 = 0xff41;
const SCROLL_Y_REGISTER: u16 = 0xff42;
const SCROLL_X_REGISTER: u16 = 0xff43;
const SCANLINE_Y_REGISTER: u16 = 0xff44;
const SCANLINE_Y_COMPARE_REGISTER: u16 = 0xff45;

#[derive(Debug)]
pub struct Tile {
    pub pixels: [[u8; 8]; 8],
}

impl Tile {
    pub fn from_addr(mmu: &Mmu, address: u16) -> Tile {
        let mut pixels = [[0; 8]; 8];

        for y in 0..8 {
            let lsb = mmu.read(address + y * 2);
            let msb = mmu.read(address + y * 2 + 1);

            for x in 0..8 {
                let lsb_bit = (lsb >> (7 - x)) & 1;
                let msb_bit = (msb >> (7 - x)) & 1;

                let color = (msb_bit << 1) | lsb_bit;
                pixels[y as usize][x as usize] = color;
            }
        }

        Tile { pixels }
    }
}

#[derive(Debug)]
pub struct Ppu {}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {}
    }

    pub fn tick(&mut self, mmu: &mut Mmu) {
        //let ly = mmu.read(SCANLINE_Y_REGISTER);
        //mmu.write(SCANLINE_Y_REGISTER, ly.wrapping_add(1));
        mmu.write(SCANLINE_Y_REGISTER, 0x90); // FIXME: stub for trace
    }

    pub fn render_tilemap(&self, mmu: &Mmu) -> Vec<Tile> {
        let mut tilemap: Vec<Tile> = Vec::new();

        for tile_nr in 0..384 {
            let tile_address = TILE_ADDRESS + (tile_nr as u16 * 16);
            let tile = Tile::from_addr(mmu, tile_address);
            tilemap.push(tile);
        }

        tilemap
    }
}
