use crate::memory::mmu::Mmu;

pub struct Apu {}

impl Apu {
    pub fn new() -> Apu {
        Apu {}
    }

    pub fn tick(&mut self, _mmu: &mut Mmu, _cycles: usize) {}

    pub fn _sample(&self) -> f32 {
        0.0
    }
}
