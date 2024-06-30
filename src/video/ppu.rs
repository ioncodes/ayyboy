use crate::memory::mmu::Mmu;
use crate::memory::registers::InterruptFlags;
use crate::memory::INTERRUPT_FLAGS_REGISTER;
use crate::video::palette::Palette;
use crate::video::tile::Tile;
use crate::video::{SCREEN_HEIGHT, SCREEN_WIDTH};

pub const BACKGROUND_WIDTH: usize = 256;
pub const BACKGROUND_HEIGHT: usize = 256;

pub const TILEMAP_ADDRESS: u16 = 0x8000;
pub const BACKGROUND_0_ADDRESS: u16 = 0x9800;
pub const BACKGROUND_1_ADDRESS: u16 = 0x9c00;
pub const BACKGROUND_MAP_SIZE: usize = 32 * 32;

pub const LCD_CONTROL_REGISTER: u16 = 0xff40;
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
        let scanline = mmu.read_unchecked(SCANLINE_Y_REGISTER);
        let scanline = scanline.wrapping_add(1);
        if scanline >= 154 {
            mmu.write_unchecked(SCANLINE_Y_REGISTER, 0);
        } else {
            mmu.write_unchecked(SCANLINE_Y_REGISTER, scanline);
        }

        let mut interrupt_flags = mmu.read_as_unchecked::<InterruptFlags>(INTERRUPT_FLAGS_REGISTER);

        // Raise vblank IRQ
        if scanline >= 144 {
            interrupt_flags |= InterruptFlags::VBLANK;
        }

        // Raise stat IRQ
        let lyc = mmu.read_unchecked(SCANLINE_Y_COMPARE_REGISTER);
        if scanline == lyc {
            interrupt_flags |= InterruptFlags::LCD_STAT;
        }

        mmu.write_unchecked(INTERRUPT_FLAGS_REGISTER, interrupt_flags.bits());
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

        let bg_map_addr = if mmu.read_unchecked(LCD_CONTROL_REGISTER) & 0b1000 == 0 {
            BACKGROUND_0_ADDRESS
        } else {
            BACKGROUND_1_ADDRESS
        };

        for idx in 0..BACKGROUND_MAP_SIZE {
            let tile_nr = mmu.read_unchecked(bg_map_addr + idx as u16);
            let addr = TILEMAP_ADDRESS + (tile_nr as u16 * 16);
            let tile = Tile::from_addr(mmu, addr);
            tiles.push(tile);
        }

        tiles
    }

    pub fn render_background(&self, mmu: &Mmu) -> [[Palette; SCREEN_WIDTH]; SCREEN_HEIGHT] {
        let mut background: [[Palette; SCREEN_WIDTH]; SCREEN_HEIGHT] = [[Palette::White; SCREEN_WIDTH]; SCREEN_HEIGHT];
        let bg_map = self.render_backgroundmap(mmu);
        let scroll_y = mmu.read_unchecked(SCROLL_Y_REGISTER);
        let scroll_x = mmu.read_unchecked(SCROLL_X_REGISTER);

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
        mmu.read_unchecked(SCANLINE_Y_REGISTER) >= 144
    }
}
