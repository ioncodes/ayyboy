use crate::memory::mmu::Mmu;

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

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
}
