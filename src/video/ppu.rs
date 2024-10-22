use std::collections::HashMap;

use log::trace;

use crate::gameboy::Mode;
use crate::memory::mmu::Mmu;
use crate::memory::registers::{InterruptFlags, LcdControl, LcdStatus};
use crate::memory::INTERRUPT_FLAGS_REGISTER;
use crate::video::oam::Oam;
use crate::video::palette::Palette;
use crate::video::sprite::{Sprite, SpriteAttributes};
use crate::video::tile::Tile;
use crate::video::{
    LCD_CONTROL_REGISTER, LCD_STATUS_REGISTER, SCANLINE_Y_COMPARE_REGISTER, SCANLINE_Y_REGISTER, SCREEN_HEIGHT,
    SCREEN_WIDTH, SCROLL_X_REGISTER, SCROLL_Y_REGISTER, TILEMAP_0_ADDRESS, TILEMAP_1_ADDRESS, TILESET_0_ADDRESS,
    TILESET_1_ADDRESS, WINDOW_X_REGISTER, WINDOW_Y_REGISTER,
};

use super::state::State;
use super::tile::TileAttributes;
use super::{BACKGROUND_MAP_SIZE, TILESET_SIZE};

pub struct Ppu {
    pub state: State,
    cycles: usize,
    emulated_frame: [[Palette; SCREEN_WIDTH]; SCREEN_HEIGHT],
    window_line_counter: usize,
    mode: Mode,
}

impl Ppu {
    pub fn new(mode: Mode) -> Ppu {
        Ppu {
            state: State::OamScan,
            cycles: 0,
            emulated_frame: [[Palette::default(); SCREEN_WIDTH]; SCREEN_HEIGHT],
            window_line_counter: 0,
            mode,
        }
    }

    pub fn tick(&mut self, mmu: &mut Mmu) {
        if !mmu
            .read_as_unchecked::<LcdControl>(LCD_CONTROL_REGISTER)
            .contains(LcdControl::LCD_DISPLAY)
        {
            return;
        }

        self.handle_window_line_counter(mmu);
        self.render_scanline(mmu);
        self.progress_scanline(mmu);
        self.handle_interrupts(mmu);
    }

    pub fn reset_state(&mut self) {
        self.state = State::OamScan;
        self.cycles = 0;
    }

    pub fn tick_state(&mut self, mmu: &mut Mmu, cycles: usize) {
        if !mmu
            .read_as_unchecked::<LcdControl>(LCD_CONTROL_REGISTER)
            .contains(LcdControl::LCD_DISPLAY)
        {
            return;
        }

        self.cycles += cycles;

        match self.state {
            State::OamScan if self.cycles >= 80 => {
                // OAM scan is done, we can start the drawing period. Just do nothing for now.
                // TODO: Realistically, writes to the OAM should be blocked during this period
                self.cycles -= 80;
                self.state = State::Drawing;
            }
            State::Drawing if self.cycles >= 172 => {
                // Drawing is done, we can start the HBlank period. Just do nothing for now.
                // TODO: Realistically, writes to the OAM should be blocked during this period
                self.cycles -= 172;
                self.state = State::HBlank;

                let lcd_status = mmu.read_as_unchecked::<LcdStatus>(LCD_STATUS_REGISTER);
                let interrupt_flags = mmu.read_as_unchecked::<InterruptFlags>(INTERRUPT_FLAGS_REGISTER);
                if lcd_status.contains(LcdStatus::MODE_0_CONDITION) {
                    trace!("Triggering STAT for Mode 0");
                    mmu.write_unchecked(
                        INTERRUPT_FLAGS_REGISTER,
                        (interrupt_flags | InterruptFlags::STAT).bits(),
                    );
                }
            }
            State::HBlank if self.cycles >= 204 => {
                self.cycles -= 204;
                if mmu.read_unchecked(SCANLINE_Y_REGISTER) == 144 {
                    // We finished the HBlank period of the last scanline, so we can start the VBlank period
                    self.state = State::VBlank;

                    let lcd_status = mmu.read_as_unchecked::<LcdStatus>(LCD_STATUS_REGISTER);
                    let mut interrupt_flags = mmu.read_as_unchecked::<InterruptFlags>(INTERRUPT_FLAGS_REGISTER);
                    if lcd_status.contains(LcdStatus::MODE_1_CONDITION) {
                        trace!("Triggering STAT for Mode 1");
                        interrupt_flags |= InterruptFlags::STAT;
                    }

                    mmu.write_unchecked(
                        INTERRUPT_FLAGS_REGISTER,
                        (interrupt_flags | InterruptFlags::VBLANK).bits(),
                    );
                } else {
                    // We finished the HBlank period but we aren't ready for VBlank yet,
                    // so we can start a new scanline
                    // Handle internal line counter, render the current scanline,
                    // increment scanline and check for interrupts
                    self.state = State::OamScan;

                    let lcd_status = mmu.read_as_unchecked::<LcdStatus>(LCD_STATUS_REGISTER);
                    let interrupt_flags = mmu.read_as_unchecked::<InterruptFlags>(INTERRUPT_FLAGS_REGISTER);
                    if lcd_status.contains(LcdStatus::MODE_2_CONDITION) {
                        trace!("Triggering STAT for Mode 2");
                        mmu.write_unchecked(
                            INTERRUPT_FLAGS_REGISTER,
                            (interrupt_flags | InterruptFlags::STAT).bits(),
                        );
                    }
                }
            }
            State::VBlank if self.cycles >= 456 => {
                // We are currently in the VBlank period, do nothing except handling internal window
                // line counter and incrementing the scanline
                // We need to check for interrupts at the end of the VBlank period due to LY=LYC and LY=153 quirk
                self.cycles -= 456;

                if mmu.read_unchecked(SCANLINE_Y_REGISTER) == 0 {
                    // We finished the VBlank period of the last (non-visible) scanline, so we can start a new frame
                    self.state = State::OamScan;

                    let lcd_status = mmu.read_as_unchecked::<LcdStatus>(LCD_STATUS_REGISTER);
                    let interrupt_flags = mmu.read_as_unchecked::<InterruptFlags>(INTERRUPT_FLAGS_REGISTER);
                    if lcd_status.contains(LcdStatus::MODE_2_CONDITION) {
                        trace!("Triggering STAT for Mode 2");
                        mmu.write_unchecked(
                            INTERRUPT_FLAGS_REGISTER,
                            (interrupt_flags | InterruptFlags::STAT).bits(),
                        );
                    }
                }
            }
            _ => {}
        }
    }

    pub fn handle_window_line_counter(&mut self, mmu: &mut Mmu) {
        let scanline = mmu.read_unchecked(SCANLINE_Y_REGISTER);

        // Reset window line counter if we start a new frame
        if scanline == 0 {
            self.window_line_counter = 0;
        }

        let wx = mmu.read_unchecked(WINDOW_X_REGISTER);
        let wy = mmu.read_unchecked(WINDOW_Y_REGISTER);
        let lcdc = mmu.read_as_unchecked::<LcdControl>(LCD_CONTROL_REGISTER);

        // Check if the window is enabled and the scanline is within valid range
        if lcdc.contains(LcdControl::WINDOW_DISPLAY) && wx <= 166 && wy <= 143 {
            if scanline == wy {
                // Reset window line counter if scanline matches WY
                self.window_line_counter = 0;
            } else if scanline > wy {
                // Increment window line counter if scanline is greater than WY
                self.window_line_counter += 1;
            }
        }
    }

    pub fn render_scanline(&mut self, mmu: &Mmu) {
        let scanline = mmu.read_unchecked(SCANLINE_Y_REGISTER) as usize;
        if scanline >= SCREEN_HEIGHT {
            return;
        }

        let lcdc = mmu.read_as_unchecked::<LcdControl>(LCD_CONTROL_REGISTER);
        if !lcdc.contains(LcdControl::LCD_DISPLAY) {
            for x in 0..SCREEN_WIDTH {
                self.emulated_frame[scanline][x] = Palette::White(0);
            }
            return;
        }

        let sprite_height = if lcdc.contains(LcdControl::OBJ_SIZE) { 16 } else { 8 };
        let oams = self.fetch_oams(mmu, sprite_height);

        // Track visited OAMs for current scanline
        // Key: sprite address (as OAM identifier), Value: (x coordinate, pixel color)
        let mut visited_oams: HashMap<u16, Vec<(usize, Palette)>> = HashMap::new();

        for x in 0..SCREEN_WIDTH {
            let (background_color, bg_tile) = self.fetch_background_pixel(mmu, x, scanline);
            self.emulated_frame[scanline][x] = background_color;

            let (window_color, win_tile) = self.fetch_window_pixel(mmu, x, scanline);
            if !window_color.is_transparent() {
                self.emulated_frame[scanline][x] = window_color;
            }

            if visited_oams.len() <= 10
                && mmu
                    .read_as_unchecked::<LcdControl>(LCD_CONTROL_REGISTER)
                    .contains(LcdControl::OBJ_DISPLAY)
                && let Some((sprite, sprite_color)) = self.fetch_sprite_pixel(&oams, x, scanline, sprite_height)
            {
                let is_bg_visible = !background_color.is_color(0);
                let is_win_visible = !window_color.is_color(0) && !window_color.is_transparent();

                if sprite.attributes.contains(SpriteAttributes::PRIORITY) && (is_bg_visible || is_win_visible) {
                    continue;
                }

                // Are background and window tiles deprioritized?
                let cgb_sprite_prio = self.mode == Mode::Cgb && !lcdc.contains(LcdControl::BG_AND_WIN_DISPLAY);

                // Do the background or window tiles have priority while being visible?
                let cgb_master_prio = self.mode == Mode::Cgb
                    && ((bg_tile.attributes.contains(TileAttributes::PRIORITY) && is_bg_visible)
                        || (win_tile.attributes.contains(TileAttributes::PRIORITY) && is_win_visible));

                if !cgb_sprite_prio && cgb_master_prio {
                    continue;
                }

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

    pub fn render_tileset(&mut self, mmu: &Mmu, vram_source: u8) -> Vec<Tile> {
        let mut tiles: Vec<Tile> = Vec::new();

        let tileset_addr = self.get_tileset_address(mmu);

        for tile_nr in 0..TILESET_SIZE {
            let addr = tileset_addr + (tile_nr as u16 * 16);

            // Fake attributes to select the correct bank
            let mut attributes = TileAttributes::empty();
            attributes.set(TileAttributes::BANK, vram_source == 1);

            let tile = Tile::from(mmu, addr, &self.mode, attributes);
            tiles.push(tile);
        }

        tiles
    }

    pub fn render_background_tilemap(&mut self, mmu: &Mmu) -> Vec<Tile> {
        let mut tiles: Vec<Tile> = Vec::new();

        let tileset_addr = self.get_tileset_address(mmu);
        let tilemap_addr = self.get_background_tilemap_address(mmu);

        for idx in 0..BACKGROUND_MAP_SIZE {
            let tile_nr = mmu.read_from_vram(tilemap_addr + idx as u16, 0);
            let addr = if tileset_addr == TILESET_0_ADDRESS {
                tileset_addr + ((tile_nr as u16) * 16)
            } else {
                tileset_addr.wrapping_add_signed((tile_nr as i8 as i16 + 128) * 16)
            };
            let attributes = if self.mode == Mode::Cgb {
                TileAttributes::from_bits_truncate(mmu.read_from_vram(tilemap_addr + idx as u16, 1))
            } else {
                TileAttributes::empty()
            };
            let tile = Tile::from(mmu, addr, &self.mode, attributes);
            tiles.push(tile);
        }

        tiles
    }

    pub fn render_window_tilemap(&mut self, mmu: &Mmu) -> Vec<Tile> {
        let mut tiles: Vec<Tile> = Vec::new();

        let tileset_addr = self.get_tileset_address(mmu);
        let tilemap_addr = self.get_window_tilemap_address(mmu);

        for idx in 0..BACKGROUND_MAP_SIZE {
            let tile_nr = mmu.read_from_vram(tilemap_addr + idx as u16, 0);
            let addr = if tileset_addr == TILESET_0_ADDRESS {
                tileset_addr + ((tile_nr as u16) * 16)
            } else {
                tileset_addr.wrapping_add_signed((tile_nr as i8 as i16 + 128) * 16)
            };
            let tile = Tile::from(mmu, addr, &self.mode, TileAttributes::empty());
            tiles.push(tile);
        }

        tiles
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
        // if scanline == 144 {
        //     interrupt_flags |= InterruptFlags::VBLANK;
        // }

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

    fn fetch_background_pixel(&self, mmu: &Mmu, x: usize, y: usize) -> (Palette, Tile) {
        // Handle case where background is disabled
        if !mmu
            .read_as_unchecked::<LcdControl>(LCD_CONTROL_REGISTER)
            .contains(LcdControl::BG_AND_WIN_DISPLAY)
            && self.mode == Mode::Dmg
        {
            return (
                Palette::from_background(0, mmu, &self.mode, &TileAttributes::empty()),
                Tile::default(),
            );
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
        let bg_map_addr = (tilemap + (bg_map_y * 32)) + bg_map_x;
        let tile_number = mmu.read_from_vram(bg_map_addr, 0);

        // Calculate the address of the tile data
        let tile_addr = if tileset == TILESET_0_ADDRESS {
            tileset + ((tile_number as u16) * 16)
        } else {
            tileset.wrapping_add_signed((tile_number as i8 as i16 + 128) * 16)
        };

        let attributes = if self.mode == Mode::Cgb {
            TileAttributes::from_bits_truncate(mmu.read_from_vram(bg_map_addr, 1))
        } else {
            TileAttributes::empty()
        };
        let tile = Tile::from(mmu, tile_addr, &self.mode, attributes);

        // Calculate the pixel coordinates in the tile
        let mut tile_x = ((x as u8).wrapping_add(scx)) % 8;
        let mut tile_y = ((y as u8).wrapping_add(scy)) % 8;

        // Flip tiles if we're in CGB mode and the tile attributes require it
        if self.mode == Mode::Cgb {
            if tile.attributes.contains(TileAttributes::FLIP_X) {
                tile_x = 7 - tile_x;
            }

            if tile.attributes.contains(TileAttributes::FLIP_Y) {
                tile_y = 7 - tile_y;
            }
        }

        // Get the color of the pixel
        (tile.pixels[tile_y as usize][tile_x as usize], tile)
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

                let tile_top = Tile::from_sprite(mmu, tile_addr_top, &sprite, &self.mode);
                let tile_bot = Tile::from_sprite(mmu, tile_addr_bot, &sprite, &self.mode);

                oams.push(Oam {
                    sprite,
                    tile1: tile_top,
                    tile2: Some(tile_bot),
                });
            } else {
                // 8px sprite
                let tile_addr = TILESET_0_ADDRESS + (sprite.tile_index as u16) * 16;
                let tile = Tile::from_sprite(mmu, tile_addr, &sprite, &self.mode);

                oams.push(Oam {
                    sprite,
                    tile1: tile,
                    tile2: None,
                });
            }
        }

        oams
    }

    fn fetch_sprite_pixel(
        &self, oams: &Vec<Oam>, x: usize, y: usize, sprite_height: usize,
    ) -> Option<(Sprite, Palette)> {
        let mut sprites: Vec<(Sprite, Palette)> = Vec::new();

        for oam in oams {
            let sprite = &oam.sprite;

            let sprite_y = sprite.y as i32 - 16;
            let sprite_x = sprite.x as i32 - 8;

            if x >= sprite_x.max(0) as usize
                && x < (sprite_x + 8).min(SCREEN_WIDTH as i32) as usize
                && y >= sprite_y.max(0) as usize
                && y < (sprite_y + sprite_height as i32).min(SCREEN_HEIGHT as i32) as usize
            {
                if sprite_height == 16 {
                    // 16px sprite
                    let tile_top = &oam.tile1;
                    let tile_bot = oam.tile2.as_ref().unwrap();

                    let mut tile_x = (x as i32 - sprite_x).max(0) as u8; // Ensure we don't go below zero
                    let mut tile_y = (y as i32 - sprite_y).max(0) as u8; // Ensure we don't go below zero

                    if sprite.attributes.contains(SpriteAttributes::FLIP_X) {
                        tile_x = 7u8.saturating_sub(tile_x);
                    }

                    if sprite.attributes.contains(SpriteAttributes::FLIP_Y) {
                        tile_y = 15u8.saturating_sub(tile_y);
                    }

                    // Ensure tile_x and tile_y are within bounds
                    if tile_x < 8 && tile_y < 16 {
                        let color = if tile_y < 8 {
                            tile_top.pixels[tile_y as usize][tile_x as usize]
                        } else {
                            tile_bot.pixels[(tile_y - 8) as usize][tile_x as usize]
                        };

                        if !color.is_transparent() {
                            sprites.push((sprite.clone(), color));
                        }
                    }
                } else {
                    // 8px sprite
                    let tile = &oam.tile1;

                    let mut tile_x = (x as i32 - sprite_x).max(0) as u8; // Ensure we don't go below zero
                    let mut tile_y = (y as i32 - sprite_y).max(0) as u8; // Ensure we don't go below zero

                    if sprite.attributes.contains(SpriteAttributes::FLIP_X) {
                        tile_x = 7u8.saturating_sub(tile_x);
                    }

                    if sprite.attributes.contains(SpriteAttributes::FLIP_Y) {
                        tile_y = 7u8.saturating_sub(tile_y);
                    }

                    // Ensure tile_x and tile_y are within bounds
                    if tile_x < 8 && tile_y < 8 {
                        let color = tile.pixels[tile_y as usize][tile_x as usize];
                        if !color.is_transparent() {
                            sprites.push((sprite.clone(), color));
                        }
                    }
                };
            }
        }

        // Sort sprites by x coordinate if we're in DMG mode
        if self.mode == Mode::Dmg {
            sprites.sort_by(|a, b| a.0.x.cmp(&b.0.x));
        }

        // Return sprite pixel with highest priority
        if let Some((sprite, color)) = sprites.first() {
            return Some((sprite.clone(), *color));
        }

        None
    }

    fn fetch_window_pixel(&self, mmu: &Mmu, x: usize, y: usize) -> (Palette, Tile) {
        if !mmu
            .read_as_unchecked::<LcdControl>(LCD_CONTROL_REGISTER)
            .contains(LcdControl::BG_AND_WIN_DISPLAY)
            || !mmu
                .read_as_unchecked::<LcdControl>(LCD_CONTROL_REGISTER)
                .contains(LcdControl::WINDOW_DISPLAY)
        {
            return (Palette::Transparent(0), Tile::default());
        }

        // Read renderer values from memory
        let wy = mmu.read_unchecked(WINDOW_Y_REGISTER);
        let wx = mmu.read_unchecked(WINDOW_X_REGISTER);

        // Return transparent color if window is not on screen
        if y < wy as usize || x + 7 < wx as usize {
            return (Palette::Transparent(0), Tile::default());
        }

        // Adjust the coordinates based on renderer position
        let window_x = x.wrapping_add(7).wrapping_sub(wx as usize);
        let window_y = self.window_line_counter;

        // Read the renderer map and tile data addresses from memory
        let tilemap = self.get_window_tilemap_address(mmu);
        let tileset = self.get_tileset_address(mmu);

        // Calculate the tile coordinates in the window map
        let win_map_x = (window_x / 8) as u16;
        let win_map_y = (window_y / 8) as u16;
        let win_map_addr = (tilemap + (win_map_y * 32)) + win_map_x;
        let tile_number = mmu.read_from_vram(win_map_addr, 0);

        // Calculate the address of the tile data
        let tile_addr = if tileset == TILESET_0_ADDRESS {
            tileset + ((tile_number as u16) * 16)
        } else {
            tileset.wrapping_add_signed((tile_number as i8 as i16 + 128) * 16)
        };

        let attributes = if self.mode == Mode::Cgb {
            TileAttributes::from_bits_truncate(mmu.read_from_vram(win_map_addr, 1))
        } else {
            TileAttributes::empty()
        };
        let tile = Tile::from(mmu, tile_addr, &self.mode, attributes);

        // Calculate the pixel coordinates in the tile
        let mut tile_x = window_x % 8;
        let mut tile_y = window_y % 8;

        // Flip tiles if we're in CGB mode and the tile attributes require it
        if self.mode == Mode::Cgb {
            if tile.attributes.contains(TileAttributes::FLIP_X) {
                tile_x = 7 - tile_x;
            }

            if tile.attributes.contains(TileAttributes::FLIP_Y) {
                tile_y = 7 - tile_y;
            }
        }

        // Get the color of the pixel
        (tile.pixels[tile_y as usize][tile_x as usize], tile)
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
            .contains(LcdControl::BG_AND_WIN_TILE_DATA)
        {
            TILESET_1_ADDRESS
        } else {
            TILESET_0_ADDRESS
        }
    }
}
