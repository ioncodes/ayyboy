use crate::memory::mmu::Mmu;

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

const BACKGROUND_WIDTH: usize = 256;
const BACKGROUND_HEIGHT: usize = 256;

const BACKGROUND_0_ADDRESS: u16 = 0x9800;
const BACKGROUND_1_ADDRESS: u16 = 0x9c00;

const CONTROL_REGISTER: u16 = 0xff40;
const STATUS_REGISTER: u16 = 0xff41;
const SCROLL_Y_REGISTER: u16 = 0xff42;
const SCROLL_X_REGISTER: u16 = 0xff43;
const SCANLINE_Y_REGISTER: u16 = 0xff44;
const SCANLINE_Y_COMPARE_REGISTER: u16 = 0xff45;

pub struct Ppu {}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {}
    }

    pub fn tick(&mut self, mmu: &mut Mmu) {
        let ly = mmu.read(SCANLINE_Y_REGISTER);
        mmu.write(SCANLINE_Y_REGISTER, ly.wrapping_add(1));
    }

    pub fn render_background(&self, mmu: &Mmu) -> Vec<u8> {
        let mut tile_numbers = vec![0u8; 32 * 32];

        for y in 0..32 {
            for x in 0..32 {
                let addr = BACKGROUND_0_ADDRESS + (y * 32 + x) as u16;
                tile_numbers.push(mmu.read(addr));
            }
        }

        tile_numbers
    }
}
