use crate::memory::mmu::Mmu;
use crate::memory::registers::InterruptFlags;
use crate::memory::INTERRUPT_FLAGS_REGISTER;
use crate::video::palette::{Color, Palette};
use crate::video::sprite::Sprite;
use crate::video::tile::Tile;
use crate::video::{SCREEN_HEIGHT, SCREEN_WIDTH};

pub const BACKGROUND_WIDTH: usize = 256;
pub const BACKGROUND_HEIGHT: usize = 256;

pub const TILEMAP_0_ADDRESS: u16 = 0x8000;
pub const TILEMAP_1_ADDRESS: u16 = 0x8800;
pub const BACKGROUND_0_ADDRESS: u16 = 0x9800;
pub const BACKGROUND_1_ADDRESS: u16 = 0x9c00;
pub const OAM_ADDRESS: u16 = 0xfe00;

pub const BACKGROUND_MAP_SIZE: usize = 32 * 32;

pub const LCD_CONTROL_REGISTER: u16 = 0xff40;
pub const SCROLL_Y_REGISTER: u16 = 0xff42;
pub const SCROLL_X_REGISTER: u16 = 0xff43;
pub const SCANLINE_Y_REGISTER: u16 = 0xff44;
pub const SCANLINE_Y_COMPARE_REGISTER: u16 = 0xff45;
pub const BG_PALETTE_REGISTER: u16 = 0xff47;

#[derive(Debug)]
pub struct Ppu {
    emulated_frame: [[Palette; SCREEN_WIDTH]; SCREEN_HEIGHT],
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            emulated_frame: [[Palette::White; SCREEN_WIDTH]; SCREEN_HEIGHT],
        }
    }

    pub fn tick(&mut self, mmu: &mut Mmu) {
        self.render_scanline(mmu);

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

    pub fn render_scanline(&mut self, mmu: &Mmu) {
        let scanline = mmu.read_unchecked(SCANLINE_Y_REGISTER) as usize;
        if scanline >= SCREEN_HEIGHT {
            return;
        }

        // Track visited OAMs for current scanline
        let mut visited_oams: Vec<u16> = Vec::new();

        for x in 0..SCREEN_WIDTH {
            if visited_oams.len() > 10 {
                break;
            }

            let background_color = self.fetch_background_pixel(mmu, x, scanline);

            if let Some((oam_id, sprite_color)) = self.fetch_sprite_pixel(mmu, x, scanline) {
                self.emulated_frame[scanline][x] = sprite_color;
                if !visited_oams.contains(&oam_id) {
                    visited_oams.push(oam_id);
                }
            } else {
                self.emulated_frame[scanline][x] = background_color;
            }
        }
    }

    pub fn get_frame(&self) -> &[[Palette; SCREEN_WIDTH]; SCREEN_HEIGHT] {
        &self.emulated_frame
    }

    pub fn render_tilemap(&mut self, mmu: &Mmu) -> Vec<Tile> {
        let mut tiles: Vec<Tile> = Vec::new();

        let tile_map_addr = if mmu.read_unchecked(LCD_CONTROL_REGISTER) & 0b0001_0000 == 0 {
            TILEMAP_1_ADDRESS
        } else {
            TILEMAP_0_ADDRESS
        };

        for tile_nr in 0..384 {
            let addr = tile_map_addr + (tile_nr as u16 * 16);
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

        let tile_map_addr = if mmu.read_unchecked(LCD_CONTROL_REGISTER) & 0b0001_0000 == 0 {
            TILEMAP_0_ADDRESS // should be 1?
        } else {
            TILEMAP_0_ADDRESS
        };

        for idx in 0..BACKGROUND_MAP_SIZE {
            let tile_nr = mmu.read_unchecked(bg_map_addr + idx as u16);
            let addr = tile_map_addr + (tile_nr as u16 * 16);
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

    fn fetch_background_pixel(&self, mmu: &Mmu, x: usize, y: usize) -> Palette {
        // Read scroll values from memory
        let scy = mmu.read_unchecked(SCROLL_Y_REGISTER);
        let scx = mmu.read_unchecked(SCROLL_X_REGISTER);

        // Read the background map and tile data addresses from memory
        let map_addr = self.background_map_address(mmu);
        let tile_data_addr = self.tile_map_address(mmu);

        // Calculate the tile coordinates in the background map
        let bg_map_x = ((x as u8 + scx) / 8) as u16;
        let bg_map_y = ((y as u8 + scy) / 8) as u16;
        let tile_number = mmu.read_unchecked((map_addr + (bg_map_y * 32)) + bg_map_x);

        // Calculate the address of the tile data
        let tile_addr = tile_data_addr + (tile_number as u16) * 16;
        let tile = Tile::from_addr(mmu, tile_addr);

        // Calculate the pixel coordinates in the tile
        let tile_x = (x as u8 + scx) % 8;
        let tile_y = (y as u8 + scy) % 8;

        // Get the color of the pixel
        tile.pixels[tile_y as usize][tile_x as usize]
    }

    fn fetch_sprite_pixel(&self, mmu: &Mmu, x: usize, y: usize) -> Option<(u16, Palette)> {
        for i in 0..40 {
            let sprite = Sprite::from_oam(mmu, i);

            if sprite.is_visible_on_scanline(y) {
                let sprite_y = sprite.y.wrapping_sub(16);
                let sprite_x = sprite.x.wrapping_sub(8);

                if x >= sprite_x as usize && x < (sprite_x as usize + 8) {
                    let tile_addr = self.tile_map_address(mmu) + (sprite.tile_index as u16) * 16;
                    let tile = Tile::from_addr(mmu, tile_addr);

                    let mut tile_x = (x - sprite_x as usize) as u8;
                    let mut tile_y = (y - sprite_y as usize) as u8;

                    let color = tile.pixels[tile_y as usize][tile_x as usize];

                    if !color.is_transparent() {
                        return Some((i, color));
                    }
                }
            }
        }

        None
    }

    fn background_map_address(&self, mmu: &Mmu) -> u16 {
        if mmu.read_unchecked(LCD_CONTROL_REGISTER) & 0b1000 == 0 {
            BACKGROUND_0_ADDRESS
        } else {
            BACKGROUND_1_ADDRESS
        }
    }

    fn tile_map_address(&self, mmu: &Mmu) -> u16 {
        if mmu.read_unchecked(LCD_CONTROL_REGISTER) & 0b0001_0000 == 0 {
            TILEMAP_0_ADDRESS // TODO: this should be 1?
        } else {
            TILEMAP_0_ADDRESS
        }
    }
}
