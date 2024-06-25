use crate::memory::mmu::Mmu;
use crate::video::palette::Palette;
use crate::video::tile::Tile;
use crate::video::{SCREEN_HEIGHT, SCREEN_WIDTH};

pub const BACKGROUND_WIDTH: usize = 256;
pub const BACKGROUND_HEIGHT: usize = 256;

pub const TILEMAP_ADDRESS: u16 = 0x8000;
pub const BACKGROUND_0_ADDRESS: u16 = 0x9800;
pub const BACKGROUND_1_ADDRESS: u16 = 0x9c00;
pub const BACKGROUND_MAP_SIZE: usize = 32 * 32;

pub const CONTROL_REGISTER: u16 = 0xff40;
pub const STATUS_REGISTER: u16 = 0xff41;
pub const SCROLL_Y_REGISTER: u16 = 0xff42;
pub const SCROLL_X_REGISTER: u16 = 0xff43;
pub const SCANLINE_Y_REGISTER: u16 = 0xff44;
pub const SCANLINE_Y_COMPARE_REGISTER: u16 = 0xff45;
pub const BG_PALETTE_REGISTER: u16 = 0xff47;

#[derive(Debug)]
pub struct Ppu {}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {}
    }

    pub fn tick(&mut self, mmu: &mut Mmu) {
        let scanline = mmu.read(SCANLINE_Y_REGISTER);
        let scanline = scanline.wrapping_add(1);
        if scanline >= 154 {
            mmu.write(SCANLINE_Y_REGISTER, 0);
        } else {
            mmu.write(SCANLINE_Y_REGISTER, scanline);
        }

        //mmu.write(SCANLINE_Y_REGISTER, 0x90); // stub for trace
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

    pub fn render_backgroundmap(&self, mmu: &Mmu) -> Vec<Tile> {
        let mut tiles: Vec<Tile> = Vec::new();

        for idx in 0..BACKGROUND_MAP_SIZE {
            let tile_nr = mmu.read(BACKGROUND_0_ADDRESS + idx as u16);
            let addr = TILEMAP_ADDRESS + (tile_nr as u16 * 16);
            let tile = Tile::from_addr(mmu, addr);
            tiles.push(tile);
        }

        tiles
    }

    pub fn render_background(&self, mmu: &Mmu) -> [[Palette; SCREEN_WIDTH]; SCREEN_HEIGHT] {
        let mut background: [[Palette; SCREEN_WIDTH]; SCREEN_HEIGHT] = [[Palette::White; SCREEN_WIDTH]; SCREEN_HEIGHT];
        let bg_map = self.render_backgroundmap(mmu);
        let scroll_y = mmu.read(SCROLL_Y_REGISTER);
        let scroll_x = mmu.read(SCROLL_X_REGISTER);

        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let tile_x = (x + scroll_x as usize) % BACKGROUND_WIDTH;
                let tile_y = (y + scroll_y as usize) % BACKGROUND_HEIGHT;
                let tile_nr = (tile_y / 8) * 32 + (tile_x / 8);
                let tile = &bg_map[tile_nr];
                let pixel_x = tile_x % 8;
                let pixel_y = tile_y % 8;
                background[y][x] = tile.pixels[pixel_y][pixel_x];
            }
        }

        background
    }

    pub fn is_vblank(&self, mmu: &Mmu) -> bool {
        mmu.read(SCANLINE_Y_REGISTER) >= 144
    }
}
