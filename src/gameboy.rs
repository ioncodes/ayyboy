use crate::lr35902::cpu::Cpu;
use crate::memory::mmu::Mmu;
use crate::video::ppu::Ppu;

pub struct GameBoy {
    cpu: Cpu,
    mmu: Mmu,
    ppu: Ppu,
    cycles: usize,
}

impl GameBoy {
    pub fn new(bootrom: Vec<u8>, cartridge: Vec<u8>) -> GameBoy {
        let cpu = Cpu::new();
        let mmu = Mmu::new(bootrom, cartridge);
        let ppu = Ppu::new();

        GameBoy { cpu, mmu, ppu, cycles: 0 }
    }

    pub fn tick(&mut self) {
        loop {
            self.cpu.tick(&mut self.mmu);

            // Each scanline takes exactly 456 dots, or 114 cycles.
            // Mode 2 also takes a constant amount of time (20 cycles) HBlank's length varies wildly,
            // and will often be nearly as long as or longer than the drawing phase.
            if self.cpu.current_cycles() - self.cycles >= 114 {
                break;
            }
        }

        self.ppu.tick(&mut self.mmu); // "does a scanline"
        self.cycles = self.cpu.current_cycles();
    }
}
