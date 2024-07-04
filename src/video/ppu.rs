use std::collections::HashMap;

use crate::memory::mmu::Mmu;
use crate::memory::registers::{InterruptFlags, LcdControl, LcdStatus};
use crate::memory::INTERRUPT_FLAGS_REGISTER;
use crate::video::oam::Oam;
use crate::video::palette::Palette;
use crate::video::sprite::{Sprite, SpriteAttributes};
use crate::video::tile::Tile;
use crate::video::{
    LCD_CONTROL_REGISTER, LCD_STATUS_REGISTER, SCANLINE_Y_COMPARE_REGISTER, SCANLINE_Y_REGISTER, SCREEN_HEIGHT, SCREEN_WIDTH,
    SCROLL_X_REGISTER, SCROLL_Y_REGISTER, TILEMAP_0_ADDRESS, TILEMAP_1_ADDRESS, TILESET_0_ADDRESS, TILESET_1_ADDRESS, WINDOW_X_REGISTER,
    WINDOW_Y_REGISTER,
};

use super::{_BACKGROUND_HEIGHT, _BACKGROUND_MAP_SIZE, _BACKGROUND_WIDTH};

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
        self.progress_scanline(mmu);
        self.handle_interrupts(mmu);
    }

    pub fn render_scanline(&mut self, mmu: &Mmu) {
        let scanline = mmu.read_unchecked(SCANLINE_Y_REGISTER) as usize;
        if scanline >= SCREEN_HEIGHT {
            return;
        }

        let lcdc = mmu.read_as_unchecked::<LcdControl>(LCD_CONTROL_REGISTER);
        let sprite_height = if lcdc.contains(LcdControl::OBJ_SIZE) { 16 } else { 8 };
        let oams = self.fetch_oams(mmu, sprite_height);

        // Track visited OAMs for current scanline
        // Key: sprite address (as OAM identifier), Value: (x coordinate, pixel color)
        let mut visited_oams: HashMap<u16, Vec<(usize, Palette)>> = HashMap::new();

        for x in 0..SCREEN_WIDTH {
            let background_color = self.fetch_background_pixel(mmu, x, scanline);
            self.emulated_frame[scanline][x] = background_color;

            let window_color = self.fetch_window_pixel(mmu, x, scanline);
            if !window_color.is_transparent() {
                self.emulated_frame[scanline][x] = window_color;
            }

            if visited_oams.len() <= 10
                && mmu
                    .read_as_unchecked::<LcdControl>(LCD_CONTROL_REGISTER)
                    .contains(LcdControl::OBJ_DISPLAY)
                && let Some((sprite, sprite_color)) = self.fetch_sprite_pixel(&oams, x, scanline, sprite_height)
            // TODO: && !sprite.attributes.contains(SpriteAttributes::PRIORITY)
            {
                visited_oams
                    .entry(sprite.oam_addr)
                    .or_insert_with(Vec::new)
                    .push((x, sprite_color));
            }
        }

        for (_, oam) in visited_oams {
            for (x, color) in oam {
                self.emulated_frame[scanline][x] = color;
            }
        }
    }

    pub fn pull_frame(&self) -> [[Palette; SCREEN_WIDTH]; SCREEN_HEIGHT] {
        self.emulated_frame
    }

    pub fn _render_tilemap(&mut self, mmu: &Mmu) -> Vec<Tile> {
        let mut tiles: Vec<Tile> = Vec::new();

        let tile_map_addr = if mmu.read_unchecked(LCD_CONTROL_REGISTER) & 0b0001_0000 == 0 {
            TILEMAP_1_ADDRESS
        } else {
            TILESET_0_ADDRESS
        };

        for tile_nr in 0..384 {
            let addr = tile_map_addr + (tile_nr as u16 * 16);
            let tile = Tile::from_background_addr(mmu, addr, false);
            tiles.push(tile);
        }

        tiles
    }

    pub fn _render_backgroundmap(&self, mmu: &Mmu) -> Vec<Tile> {
        let mut tiles: Vec<Tile> = Vec::new();

        let bg_map_addr = if mmu.read_unchecked(LCD_CONTROL_REGISTER) & 0b1000 == 0 {
            TILEMAP_0_ADDRESS
        } else {
            TILEMAP_1_ADDRESS
        };

        let tile_map_addr = if mmu.read_unchecked(LCD_CONTROL_REGISTER) & 0b0001_0000 == 0 {
            TILESET_0_ADDRESS // should be 1?
        } else {
            TILESET_0_ADDRESS
        };

        for idx in 0.._BACKGROUND_MAP_SIZE {
            let tile_nr = mmu.read_unchecked(bg_map_addr + idx as u16);
            let addr = tile_map_addr + (tile_nr as u16 * 16);
            let tile = Tile::from_background_addr(mmu, addr, false);
            tiles.push(tile);
        }

        tiles
    }

    pub fn _render_background(&self, mmu: &Mmu) -> [[Palette; SCREEN_WIDTH]; SCREEN_HEIGHT] {
        let mut background: [[Palette; SCREEN_WIDTH]; SCREEN_HEIGHT] = [[Palette::White; SCREEN_WIDTH]; SCREEN_HEIGHT];
        let bg_map = self._render_backgroundmap(mmu);
        let scroll_y = mmu.read_unchecked(SCROLL_Y_REGISTER);
        let scroll_x = mmu.read_unchecked(SCROLL_X_REGISTER);

        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let tile_x = (x + scroll_x as usize) % _BACKGROUND_WIDTH;
                let tile_y = (y + scroll_y as usize) % _BACKGROUND_HEIGHT;
                let tile_nr = (tile_y / 8) * 32 + (tile_x / 8);
                let tile = &bg_map[tile_nr];
                let pixel_x = tile_x % 8;
                let pixel_y = tile_y % 8;
                background[y][x] = tile.pixels[pixel_y][pixel_x];
            }
        }

        background
    }

    fn progress_scanline(&self, mmu: &mut Mmu) {
        let mut scanline = mmu.read_unchecked(SCANLINE_Y_REGISTER) + 1;
        if scanline >= 154 {
            scanline = 0;
        }
        mmu.write_unchecked(SCANLINE_Y_REGISTER, scanline);
    }

    fn handle_interrupts(&self, mmu: &mut Mmu) {
        let scanline = mmu.read_unchecked(SCANLINE_Y_REGISTER);

        // Raise interrupts
        let mut interrupt_flags = mmu.read_as_unchecked::<InterruptFlags>(INTERRUPT_FLAGS_REGISTER);

        // Raise VBLANK IRQ
        if scanline == 144 {
            interrupt_flags |= InterruptFlags::VBLANK;
        }

        // Raise STAT IRQ
        // Emulate LYC=0 LY=153 quirk
        let lcd_status = mmu.read_as_unchecked::<LcdStatus>(LCD_STATUS_REGISTER);
        let lyc = mmu.read_unchecked(SCANLINE_Y_COMPARE_REGISTER);
        if lcd_status.contains(LcdStatus::LYC_EQ_LY_ENABLE) && (scanline == lyc || (scanline == 153 && lyc == 0)) {
            interrupt_flags |= InterruptFlags::STAT;
        }

        // Write back interrupt flags
        mmu.write_unchecked(INTERRUPT_FLAGS_REGISTER, interrupt_flags.bits());
    }

    fn fetch_background_pixel(&self, mmu: &Mmu, x: usize, y: usize) -> Palette {
        // Handle case where background is disabled
        // TODO: this break acid2 !
        if !mmu
            .read_as_unchecked::<LcdControl>(LCD_CONTROL_REGISTER)
            .contains(LcdControl::BG_DISPLAY)
        {
            return Palette::White;
        }

        // Read scroll values from memory
        let scy = mmu.read_unchecked(SCROLL_Y_REGISTER);
        let scx = mmu.read_unchecked(SCROLL_X_REGISTER);

        // Read the background map and tile data addresses from memory
        let tilemap = self.get_background_tilemap_address(mmu);
        let tileset = self.get_tileset_address(mmu);

        // Calculate the tile coordinates in the background map
        let bg_map_x = (((x as u8).wrapping_add(scx)) / 8) as u16;
        let bg_map_y = (((y as u8).wrapping_add(scy)) / 8) as u16;
        let tile_number = mmu.read_unchecked((tilemap + (bg_map_y * 32)) + bg_map_x);

        // Calculate the address of the tile data
        let tile_addr = if tileset == TILESET_0_ADDRESS {
            tileset + ((tile_number as u16) * 16)
        } else {
            tileset.wrapping_add_signed((tile_number as i8 as i16 + 128) * 16)
        };
        let tile = Tile::from_background_addr(mmu, tile_addr, false);

        // Calculate the pixel coordinates in the tile
        let tile_x = ((x as u8).wrapping_add(scx)) % 8;
        let tile_y = ((y as u8).wrapping_add(scy)) % 8;

        // Get the color of the pixel
        tile.pixels[tile_y as usize][tile_x as usize]
    }

    fn fetch_oams(&self, mmu: &Mmu, sprite_height: usize) -> Vec<Oam> {
        let mut oams: Vec<Oam> = Vec::new();

        for i in 0..40 {
            let sprite = Sprite::from_oam(mmu, i);

            if sprite_height == 16 {
                // 16px sprite
                let tile_index_top = sprite.tile_index & 0b1111_1110;
                let tile_index_bot = tile_index_top + 1;

                let tile_addr_top = TILESET_0_ADDRESS + (tile_index_top as u16) * 16;
                let tile_addr_bot = TILESET_0_ADDRESS + (tile_index_bot as u16) * 16;

                let tile_top = Tile::from_sprite_addr(mmu, tile_addr_top, &sprite);
                let tile_bot = Tile::from_sprite_addr(mmu, tile_addr_bot, &sprite);

                oams.push(Oam {
                    sprite,
                    tile1: tile_top,
                    tile2: Some(tile_bot),
                });
            } else {
                // 8px sprite
                let tile_addr = TILESET_0_ADDRESS + (sprite.tile_index as u16) * 16;
                let tile = Tile::from_sprite_addr(mmu, tile_addr, &sprite);

                oams.push(Oam {
                    sprite,
                    tile1: tile,
                    tile2: None,
                });
            }
        }

        oams
    }

    fn fetch_sprite_pixel(&self, oams: &Vec<Oam>, x: usize, y: usize, sprite_height: usize) -> Option<(Sprite, Palette)> {
        let mut sprites: Vec<(Sprite, Palette)> = Vec::new();

        for oam in oams {
            let sprite = &oam.sprite;

            let sprite_y = sprite.y.wrapping_sub(16);
            let sprite_x = sprite.x.wrapping_sub(8);

            if x >= sprite_x as usize && x < (sprite_x as usize + 8) && y >= sprite_y as usize && y < (sprite_y as usize + sprite_height) {
                if sprite_height == 16 {
                    // 16px sprite
                    let tile_top = &oam.tile1;
                    let tile_bot = oam.tile2.as_ref().unwrap();

                    let mut tile_x = (x - sprite_x as usize) as u8;
                    let mut tile_y = (y - sprite_y as usize) as u8;

                    if sprite.attributes.contains(SpriteAttributes::FLIP_X) {
                        tile_x = 7 - tile_x;
                    }

                    if sprite.attributes.contains(SpriteAttributes::FLIP_Y) {
                        tile_y = 15 - tile_y;
                    }

                    let color = if tile_y < 8 {
                        tile_top.pixels[tile_y as usize][tile_x as usize]
                    } else {
                        tile_bot.pixels[(tile_y - 8) as usize][tile_x as usize]
                    };

                    if !color.is_transparent() {
                        sprites.push((sprite.clone(), color));
                    }
                } else {
                    // 8px sprite
                    let tile = &oam.tile1;

                    let mut tile_x = (x - sprite_x as usize) as u8;
                    let mut tile_y = (y - sprite_y as usize) as u8;

                    if sprite.attributes.contains(SpriteAttributes::FLIP_X) {
                        tile_x = 7 - tile_x;
                    }

                    if sprite.attributes.contains(SpriteAttributes::FLIP_Y) {
                        tile_y = 7 - tile_y;
                    }

                    let color = tile.pixels[tile_y as usize][tile_x as usize];

                    if !color.is_transparent() {
                        sprites.push((sprite.clone(), color));
                    }
                };
            }
        }

        // Return sprite pixel
        sprites.sort_by(|a, b| a.0.x.cmp(&b.0.x));
        if let Some((sprite, color)) = sprites.first() {
            return Some((sprite.clone(), *color));
        }

        None
    }

    fn fetch_window_pixel(&self, mmu: &Mmu, x: usize, y: usize) -> Palette {
        if !mmu
            .read_as_unchecked::<LcdControl>(LCD_CONTROL_REGISTER)
            .contains(LcdControl::BG_DISPLAY)
            || !mmu
                .read_as_unchecked::<LcdControl>(LCD_CONTROL_REGISTER)
                .contains(LcdControl::WINDOW_DISPLAY)
        {
            return Palette::Transparent;
        }

        // Read renderer values from memory
        let wy = mmu.read_unchecked(WINDOW_Y_REGISTER);
        let wx = mmu.read_unchecked(WINDOW_X_REGISTER);

        // Return transparent color if renderer is disabled or not on screen yet
        if y < wy as usize || x + 7 < wx as usize {
            return Palette::Transparent;
        }

        // Adjust the coordinates based on renderer position
        let window_x = x as u8 + 7 - wx;
        let window_y = y as u8 - wy;

        // Read the renderer map and tile data addresses from memory
        let tilemap = self.get_window_tilemap_address(mmu);
        let tileset = self.get_tileset_address(mmu);

        // Calculate the tile coordinates in the renderer map
        let win_map_x = (window_x / 8) as u16;
        let win_map_y = (window_y / 8) as u16;
        let tile_number = mmu.read_unchecked((tilemap + (win_map_y * 32)) + win_map_x);

        // Calculate the address of the tile data
        let tile_addr = if tileset == TILESET_0_ADDRESS {
            tileset + ((tile_number as u16) * 16)
        } else {
            tileset.wrapping_add_signed((tile_number as i8 as i16 + 128) * 16)
        };
        let tile = Tile::from_background_addr(mmu, tile_addr, true);

        // Calculate the pixel coordinates in the tile
        let tile_x = window_x % 8;
        let tile_y = window_y % 8;

        // Get the color of the pixel
        tile.pixels[tile_y as usize][tile_x as usize]
    }

    fn get_background_tilemap_address(&self, mmu: &Mmu) -> u16 {
        if !mmu
            .read_as_unchecked::<LcdControl>(LCD_CONTROL_REGISTER)
            .contains(LcdControl::BG_TILE_MAP)
        {
            TILEMAP_0_ADDRESS
        } else {
            TILEMAP_1_ADDRESS
        }
    }

    fn get_window_tilemap_address(&self, mmu: &Mmu) -> u16 {
        if !mmu
            .read_as_unchecked::<LcdControl>(LCD_CONTROL_REGISTER)
            .contains(LcdControl::WINDOW_TILE_MAP)
        {
            TILEMAP_0_ADDRESS
        } else {
            TILEMAP_1_ADDRESS
        }
    }

    fn get_tileset_address(&self, mmu: &Mmu) -> u16 {
        if !mmu
            .read_as_unchecked::<LcdControl>(LCD_CONTROL_REGISTER)
            .contains(LcdControl::BG_TILE_DATA)
        {
            TILESET_1_ADDRESS
        } else {
            TILESET_0_ADDRESS
        }
    }
}
