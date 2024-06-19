use crate::lr35902::cpu::Cpu;
use crate::memory::mmu::Mmu;

pub struct GameBoy {
    cpu: Cpu,
    mmu: Mmu,
}

impl GameBoy {
    pub fn new(rom: Vec<u8>) -> GameBoy {
        GameBoy {
            cpu: Cpu::new(),
            mmu: Mmu::new(rom),
        }
    }

    pub fn tick(&mut self) {
        self.cpu.tick(&mut self.mmu);
    }
}
