pub mod palette;
pub mod ppu;
mod sprite;
pub mod tile;

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

pub const BACKGROUND_WIDTH: usize = 256;
pub const BACKGROUND_HEIGHT: usize = 256;

pub const TILESET_0_ADDRESS: u16 = 0x8000;
pub const TILESET_1_ADDRESS: u16 = 0x8800;
pub const TILEMAP_0_ADDRESS: u16 = 0x9800;
pub const TILEMAP_1_ADDRESS: u16 = 0x9c00;
pub const OAM_ADDRESS: u16 = 0xfe00;

pub const BACKGROUND_MAP_SIZE: usize = 32 * 32;

pub const LCD_CONTROL_REGISTER: u16 = 0xff40;
pub const LCD_STATUS_REGISTER: u16 = 0xff41;
pub const SCROLL_Y_REGISTER: u16 = 0xff42;
pub const SCROLL_X_REGISTER: u16 = 0xff43;
pub const SCANLINE_Y_REGISTER: u16 = 0xff44;
pub const SCANLINE_Y_COMPARE_REGISTER: u16 = 0xff45;
pub const BG_PALETTE_REGISTER: u16 = 0xff47;
pub const OBJ0_PALETTE_REGISTER: u16 = 0xff48;
pub const OBJ1_PALETTE_REGISTER: u16 = 0xff49;
pub const WINDOW_X_REGISTER: u16 = 0xff4b;
pub const WINDOW_Y_REGISTER: u16 = 0xff4a;
