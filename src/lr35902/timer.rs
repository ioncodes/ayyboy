use crate::memory::mmu::Mmu;
use crate::memory::DIV_REGISTER;

pub struct Timer {}

impl Timer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn tick(&mut self, mmu: &mut Mmu) {
        mmu.write_unchecked(DIV_REGISTER, mmu.read_unchecked(DIV_REGISTER).wrapping_add(1));
    }

    pub fn reset_divider(&mut self, mmu: &mut Mmu) {
        mmu.write_unchecked(DIV_REGISTER, 0);
    }
}
