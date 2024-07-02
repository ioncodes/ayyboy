use crate::memory::mmu::Mmu;
use crate::memory::registers::{InterruptFlags, LcdControl, LcdStatus};
use crate::memory::INTERRUPT_FLAGS_REGISTER;
use crate::video::palette::Palette;
use crate::video::sprite::Sprite;
use crate::video::tile::Tile;
use crate::video::{
    BACKGROUND_HEIGHT, BACKGROUND_MAP_SIZE, BACKGROUND_WIDTH, LCD_CONTROL_REGISTER, LCD_STATUS_REGISTER, SCANLINE_Y_COMPARE_REGISTER,
    SCANLINE_Y_REGISTER, SCREEN_HEIGHT, SCREEN_WIDTH, SCROLL_X_REGISTER, SCROLL_Y_REGISTER, TILEMAP_0_ADDRESS, TILEMAP_1_ADDRESS,
    TILESET_0_ADDRESS, TILESET_1_ADDRESS, WINDOW_X_REGISTER, WINDOW_Y_REGISTER,
};

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
        // Render the scanline
        self.render_scanline(mmu);

        // Increment scanline register
        let mut scanline = mmu.read_unchecked(SCANLINE_Y_REGISTER);
        scanline += 1;
        if scanline >= 154 {
            scanline = 0;
        }
        mmu.write_unchecked(SCANLINE_Y_REGISTER, scanline);

        // Emulate the LY == 153 bug, however, let's not write this back to the register
        // if scanline == 153 {
        //     scanline = 0;
        // }

        // Raise interrupts
        let mut interrupt_flags = mmu.read_as_unchecked::<InterruptFlags>(INTERRUPT_FLAGS_REGISTER);

        // Raise VBLANK IRQ
        if scanline >= 144 {
            interrupt_flags |= InterruptFlags::VBLANK;
        }

        // Raise STAT IRQ
        let lcd_status = mmu.read_as_unchecked::<LcdStatus>(LCD_STATUS_REGISTER);
        let lyc = mmu.read_unchecked(SCANLINE_Y_COMPARE_REGISTER);
        if lcd_status.contains(LcdStatus::LYC_EQ_LY_ENABLE) && scanline == lyc {
            interrupt_flags |= InterruptFlags::STAT;
        }

        // Write back interrupt flags
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
            let background_color = self.fetch_background_pixel(mmu, x, scanline);
            let window_color = self.fetch_window_pixel(mmu, x, scanline);

            if visited_oams.len() <= 10
                && mmu
                    .read_as_unchecked::<LcdControl>(LCD_CONTROL_REGISTER)
                    .contains(LcdControl::OBJ_DISPLAY)
                && let Some((oam_id, sprite_color)) = self.fetch_sprite_pixel(mmu, x, scanline)
            {
                self.emulated_frame[scanline][x] = sprite_color;
                if !visited_oams.contains(&oam_id) {
                    visited_oams.push(oam_id);
                }
            } else {
                if mmu
                    .read_as_unchecked::<LcdControl>(LCD_CONTROL_REGISTER)
                    .contains(LcdControl::WINDOW_DISPLAY)
                    && !window_color.is_transparent()
                {
                    self.emulated_frame[scanline][x] = window_color;
                } else {
                    self.emulated_frame[scanline][x] = background_color;
                }
            }
        }
    }

    pub fn pull_frame(&self) -> [[Palette; SCREEN_WIDTH]; SCREEN_HEIGHT] {
        self.emulated_frame
    }

    pub fn render_tilemap(&mut self, mmu: &Mmu) -> Vec<Tile> {
        let mut tiles: Vec<Tile> = Vec::new();

        let tile_map_addr = if mmu.read_unchecked(LCD_CONTROL_REGISTER) & 0b0001_0000 == 0 {
            TILEMAP_1_ADDRESS
        } else {
            TILESET_0_ADDRESS
        };

        for tile_nr in 0..384 {
            let addr = tile_map_addr + (tile_nr as u16 * 16);
            let tile = Tile::from_background_addr(mmu, addr);
            tiles.push(tile);
        }

        tiles
    }

    pub fn render_backgroundmap(&self, mmu: &Mmu) -> Vec<Tile> {
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

        for idx in 0..BACKGROUND_MAP_SIZE {
            let tile_nr = mmu.read_unchecked(bg_map_addr + idx as u16);
            let addr = tile_map_addr + (tile_nr as u16 * 16);
            let tile = Tile::from_background_addr(mmu, addr);
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
        let tile = Tile::from_background_addr(mmu, tile_addr);

        // Calculate the pixel coordinates in the tile
        let tile_x = ((x as u8).wrapping_add(scx)) % 8;
        let tile_y = ((y as u8).wrapping_add(scy)) % 8;

        // Get the color of the pixel
        tile.pixels[tile_y as usize][tile_x as usize]
    }

    fn fetch_sprite_pixel(&self, mmu: &Mmu, x: usize, y: usize) -> Option<(u16, Palette)> {
        let lcdc = mmu.read_as_unchecked::<LcdControl>(LCD_CONTROL_REGISTER);
        let sprite_height = if lcdc.contains(LcdControl::OBJ_SIZE) { 16 } else { 8 };

        for i in 0..40 {
            let sprite = Sprite::from_oam(mmu, i);

            if sprite.is_visible_on_scanline(y) || true {
                let sprite_y = sprite.y.wrapping_sub(16);
                let sprite_x = sprite.x.wrapping_sub(8);

                if x >= sprite_x as usize
                    && x < (sprite_x as usize + 8)
                    && y >= sprite_y as usize
                    && y < (sprite_y as usize + sprite_height)
                {
                    let tile_index = if sprite_height == 16 {
                        if (y - sprite_y as usize) < 8 {
                            sprite.tile_index & 0b1111_1110 // top tile
                        } else {
                            sprite.tile_index | 0b0000_0001 // bottom tile
                        }
                    } else {
                        sprite.tile_index
                    };

                    let tile_addr = TILESET_0_ADDRESS + (tile_index as u16) * 16;
                    let tile = Tile::from_sprite_addr(mmu, tile_addr, &sprite);

                    let mut tile_x = (x - sprite_x as usize) as u8;
                    let mut tile_y = (y - sprite_y as usize) as u8;

                    if sprite_height == 16 && y - sprite_y as usize >= 8 {
                        tile_y -= 8;
                    }

                    let color = tile.pixels[tile_y as usize][tile_x as usize];

                    if !color.is_transparent() {
                        return Some((i, color));
                    }
                }
            }
        }

        None
    }

    fn fetch_window_pixel(&self, mmu: &Mmu, x: usize, y: usize) -> Palette {
        // Read renderer values from memory
        let wy = mmu.read_unchecked(WINDOW_Y_REGISTER);
        let wx = mmu.read_unchecked(WINDOW_X_REGISTER);

        // Return transparent color if renderer is disabled or not on screen yet
        if y < wy as usize || x + 7 < wx as usize {
            return Palette::White;
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
        let tile = Tile::from_background_addr(mmu, tile_addr);

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
