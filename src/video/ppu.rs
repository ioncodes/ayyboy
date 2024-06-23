use crate::memory::mmu::Mmu;
use crate::video::tile::Tile;

pub const BACKGROUND_WIDTH: usize = 256;
pub const BACKGROUND_HEIGHT: usize = 256;

const TILEMAP_ADDRESS: u16 = 0x8000;
const BACKGROUND_0_ADDRESS: u16 = 0x9800;
const BACKGROUND_1_ADDRESS: u16 = 0x9c00;

const CONTROL_REGISTER: u16 = 0xff40;
const STATUS_REGISTER: u16 = 0xff41;
const SCROLL_Y_REGISTER: u16 = 0xff42;
const SCROLL_X_REGISTER: u16 = 0xff43;
const SCANLINE_Y_REGISTER: u16 = 0xff44;
const SCANLINE_Y_COMPARE_REGISTER: u16 = 0xff45;
pub const BG_PALETTE_REGISTER: u16 = 0xff47;

#[derive(Debug)]
pub struct Ppu {}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {}
    }

    pub fn tick(&mut self, mmu: &mut Mmu) {
        let ly = mmu.read(SCANLINE_Y_REGISTER);
        mmu.write(SCANLINE_Y_REGISTER, ly.wrapping_add(1));
        //mmu.write(SCANLINE_Y_REGISTER, 0x90); // FIXME: stub for trace
    }

    pub fn render_tilemap(&mut self, mmu: &Mmu) -> Vec<Tile> {
        let mut tiles: Vec<Tile> = Vec::new();

        for tile_nr in 0..384 {
            let addr = TILEMAP_ADDRESS + (tile_nr as u16 * 16);
            let tile = Tile::from_addr(mmu, addr);
            tiles.push(tile);
        }

        tiles
    }

    pub fn is_vblank(&self, mmu: &Mmu) -> bool {
        mmu.read(SCANLINE_Y_REGISTER) >= 144
    }
}
