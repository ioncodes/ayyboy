use crate::lr35902::cpu::Cpu;
use crate::lr35902::sm83::Register;
use crate::memory::mmu::Mmu;
use crate::rhai_engine::RhaiEngine;
use crate::video::ppu::{Ppu, Tile};
use std::path::PathBuf;

pub struct GameBoy<'a> {
    cpu: Cpu,
    mmu: Mmu,
    ppu: Ppu,
    cycles: usize,
    cpu_breakpoints: Vec<u16>,
    rhai: Option<RhaiEngine<'a>>,
}

impl<'a> GameBoy<'a> {
    pub fn new(bootrom: Vec<u8>, cartridge: Vec<u8>) -> GameBoy<'a> {
        let cpu = Cpu::new();
        let mmu = Mmu::new(bootrom, cartridge);
        let ppu = Ppu::new();

        GameBoy {
            cpu,
            mmu,
            ppu,
            cycles: 0,
            cpu_breakpoints: Vec::new(),
            rhai: None,
        }
    }

    pub fn with_rhai(bootrom: Vec<u8>, cartridge: Vec<u8>, path: PathBuf) -> GameBoy<'a> {
        let mut gb = GameBoy::new(bootrom, cartridge);
        gb.rhai = Some(RhaiEngine::new(path));
        gb
    }

    pub fn tick(&mut self) -> Option<Vec<Tile>> {
        loop {
            self.try_rhai_script();
            self.cpu.tick(&mut self.mmu);

            if self.cpu.read_register16(&Register::PC) == 0x100 {
                for i in 0..0x2000 {
                    if i % 16 == 0 {
                        println!();
                        print!("{:02x}: ", 0x8000 + i);
                    }
                    print!("{:02x} ", self.mmu.read(0x8000 + i));
                }

                return Some(self.ppu.render_background_map(&self.mmu));
            }

            // Each scanline takes exactly 456 dots, or 114 cycles.
            // Mode 2 also takes a constant amount of time (20 cycles) HBlank's length varies wildly,
            // and will often be nearly as long as or longer than the drawing phase.
            if self.cpu.current_cycles() - self.cycles >= 114 {
                break;
            }
        }

        self.ppu.tick(&mut self.mmu); // "does a scanline"
        self.cycles = self.cpu.current_cycles();

        None
    }

    pub fn install_breakpoints(&mut self, breakpoints: Vec<u16>) {
        self.cpu_breakpoints = breakpoints;
    }

    fn is_breakpoint_hit(&self) -> bool {
        self.cpu_breakpoints
            .iter()
            .any(|bp| *bp == self.cpu.read_register16(&Register::PC))
    }

    fn try_rhai_script(&mut self) {
        if self.rhai.is_none() || !self.is_breakpoint_hit() {
            return;
        }

        if let Some(rhai) = &mut self.rhai {
            rhai.prepare_scope(&self.cpu, &self.mmu);
            rhai.execute_script();

            let (cpu, mmu) = rhai.get_hw_from_scope();
            self.cpu = cpu;
            self.mmu = mmu;
        }
    }
}
