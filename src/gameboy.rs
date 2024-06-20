use crate::lr35902::cpu::Cpu;
use crate::memory::mmu::Mmu;
use crate::video::ppu::Ppu;

pub struct GameBoy {
    cpu: Cpu,
    mmu: Mmu,
    ppu: Ppu,
}

impl GameBoy {
    pub fn new(bootrom: Vec<u8>, cartridge: Vec<u8>) -> GameBoy {
        let cpu = Cpu::new();
        let mmu = Mmu::new(bootrom, cartridge);
        let ppu = Ppu::new();

        GameBoy { cpu, mmu, ppu }
    }

    pub fn tick(&mut self) {
        self.cpu.tick(&mut self.mmu);
    }
}
