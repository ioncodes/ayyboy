use crate::memory::mmu::Mmu;

pub struct Apu {}

impl Apu {
    pub fn new() -> Apu {
        Apu {}
    }

    pub fn tick(&mut self, mmu: &mut Mmu, cycles: usize) {}

    pub fn sample(&self) -> f32 {
        0.0
    }
}
