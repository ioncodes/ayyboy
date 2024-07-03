use crate::error::AyyError;
use crate::error::AyyError::WriteToReadOnlyMemory;
use crate::lr35902::cpu::Cpu;
use crate::lr35902::sm83::Register;
use crate::lr35902::timer::Timer;
use crate::memory::mapper::mbc1::Mbc1;
use crate::memory::mapper::rom::Rom;
use crate::memory::mapper::Mapper;
use crate::memory::mmu::Mmu;
use crate::rhai_engine::RhaiEngine;
use crate::video::palette::Palette;
use crate::video::ppu::Ppu;
use crate::video::tile::Tile;
use crate::video::{SCANLINE_Y_REGISTER, SCREEN_HEIGHT, SCREEN_WIDTH};
use eframe::egui::Key;
use log::{error, info, warn};
use std::path::PathBuf;

pub struct GameBoy<'a> {
    cpu: Cpu,
    mmu: Mmu,
    ppu: Ppu,
    timer: Timer,
    cpu_breakpoints: Vec<u16>,
    rhai: Option<RhaiEngine<'a>>,
}

impl<'a> GameBoy<'a> {
    pub fn new(bootrom: Vec<u8>, cartridge: Vec<u8>) -> GameBoy<'a> {
        let cartridge: Box<dyn Mapper> = match cartridge[0x0147] {
            0x00 => Box::new(Rom::new(cartridge)),
            0x01 | 0x02 | 0x03 => Box::new(Mbc1::new(cartridge)), // TODO: RAM + BATTERY is not supported
            _ => panic!("Unsupported cartridge type: {:02x}", cartridge[0x0147]),
        };
        info!("Cartridge type: {}", cartridge.name());

        let cpu = Cpu::new();
        let mmu = Mmu::new(bootrom, cartridge);
        let ppu = Ppu::new();
        let timer = Timer::new();

        GameBoy {
            cpu,
            mmu,
            ppu,
            timer,
            cpu_breakpoints: Vec::new(),
            rhai: None,
        }
    }

    pub fn with_rhai(bootrom: Vec<u8>, cartridge: Vec<u8>, path: PathBuf) -> GameBoy<'a> {
        let mut gb = GameBoy::new(bootrom, cartridge);
        gb.rhai = Some(RhaiEngine::new(path));
        gb
    }

    pub fn run_frame(&mut self) {
        loop {
            loop {
                self.try_rhai_script();
                match self.cpu.tick(&mut self.mmu, &mut self.timer) {
                    Ok(_) => {}
                    Err(WriteToReadOnlyMemory { address, data }) => {
                        warn!(
                            "PC @ {:04x} => Attempted to write {:02x} to unmapped read-only memory at {:04x}",
                            self.cpu.read_register16(&Register::PC),
                            data,
                            address
                        );
                    }
                    Err(AyyError::OutOfBoundsMemoryAccess { address }) => {
                        warn!(
                            "PC @ {:04x} => Attempted to read out-of-bounds memory at {:04x}",
                            self.cpu.read_register16(&Register::PC),
                            address
                        );
                    }
                    Err(AyyError::WriteToDisabledExternalRam { address, data }) => {
                        error!(
                            "PC @ {:04x} => Attempted to write {:02x} to disabled external RAM at {:04x}",
                            self.cpu.read_register16(&Register::PC),
                            data,
                            address
                        );
                    }
                    Err(e) => panic!("{}", e),
                };

                self.timer.tick(&mut self.mmu, self.cpu.elapsed_cycles());

                if self.cpu.elapsed_cycles() % 456 == 0 {
                    break;
                }
            }

            // H-Blank (Mode 0)
            // This mode takes up the remainder of the scanline after the Drawing Mode finishes,
            // more or less “padding” the duration of the scanline to a total of 456 T-Cycles.
            // The PPU effectively pauses during this mode.
            self.ppu.tick(&mut self.mmu); // "does a scanline"

            // Do we have a frame to render?
            if self.mmu.read_unchecked(SCANLINE_Y_REGISTER) == 0 {
                break;
            }
        }
    }

    pub fn update_button(&mut self, key: Key, pressed: bool) {
        self.mmu.joypad.update_button(key, pressed);
    }

    pub fn render_tilemap(&mut self) -> Vec<Tile> {
        self.ppu.render_tilemap(&self.mmu)
    }

    pub fn render_background(&mut self) -> [[Palette; SCREEN_WIDTH]; SCREEN_HEIGHT] {
        self.ppu.render_background(&self.mmu)
    }

    pub fn install_breakpoints(&mut self, breakpoints: Vec<u16>) {
        self.cpu_breakpoints = breakpoints;
    }

    pub fn emulated_frame(&self) -> [[Palette; SCREEN_WIDTH]; SCREEN_HEIGHT] {
        self.ppu.pull_frame()
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
