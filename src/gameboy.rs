use crate::error::AyyError;
use crate::lr35902::cpu::Cpu;
use crate::lr35902::sm83::Register;
use crate::lr35902::timer::Timer;
use crate::memory::mapper::mbc1::Mbc1;
use crate::memory::mapper::mbc3::Mbc3;
use crate::memory::mapper::mbc5::Mbc5;
use crate::memory::mapper::rom::Rom;
use crate::memory::mapper::Mapper;
use crate::memory::mmu::Mmu;
use crate::video::ppu::Ppu;
use crate::video::tile::Tile;
use crate::video::SCANLINE_Y_REGISTER;
use log::{error, info, warn};

const BOOTROM_DMG: &[u8] = include_bytes!("../external/roms/boot/bootix_dmg.bin");
const BOOTROM_CGB: &[u8] = include_bytes!("../external/roms/boot/sameboy_cgb.bin");

#[derive(PartialEq, Clone)]
pub enum Mode {
    Dmg,
    Cgb,
}

pub struct GameBoy {
    pub cpu: Cpu,
    pub mmu: Mmu,
    pub ppu: Ppu,
    pub timer: Timer,
    pub mode: Mode,
}

impl GameBoy {
    pub fn new(bootrom: Option<Vec<u8>>, cartridge: Vec<u8>) -> GameBoy {
        let title = cartridge[0x0134..=0x0142]
            .iter()
            .take_while(|&&c| c != 0)
            .map(|&c| c as char)
            .collect::<String>();
        info!("ROM Title: {}", title);

        let mode = match cartridge[0x0143] {
            0xc0 => Mode::Cgb,
            0x80 => Mode::Dmg, // TODO: CGB enhancements, but backwards compatible with DMG
            _ => Mode::Dmg,
        };
        info!(
            "Emulating GameBoy: {}",
            if mode == Mode::Dmg { "DMG" } else { "CGB" }
        );

        let cartridge: Box<dyn Mapper> = match cartridge[0x0147] {
            0x00 => Box::new(Rom::new(cartridge)),
            0x01 | 0x02 | 0x03 => Box::new(Mbc1::new(cartridge)),
            0x0f | 0x10 | 0x11 | 0x12 | 0x13 => Box::new(Mbc3::new(cartridge)),
            0x19 | 0x1a | 0x1b | 0x1c | 0x1d | 0x1e => Box::new(Mbc5::new(cartridge)),
            _ => panic!("Unsupported cartridge type: {:02x}", cartridge[0x0147]),
        };
        info!("Cartridge type: {}", cartridge.name());

        let bootrom = bootrom.unwrap_or_else(|| match mode {
            Mode::Dmg => BOOTROM_DMG.to_vec(),
            Mode::Cgb => BOOTROM_CGB.to_vec(),
        });

        let cpu = Cpu::new();
        let mmu = Mmu::new(bootrom, cartridge, mode.clone());
        let ppu = Ppu::new();
        let timer = Timer::new();

        GameBoy {
            cpu,
            mmu,
            ppu,
            timer,
            mode,
        }
    }

    pub fn run_frame(&mut self) {
        loop {
            loop {
                // TODO: instead of relying on cycles being return after tick, we should
                //       track total cycles before tick and then after tick subtract
                let cycles = match self.cpu.tick(&mut self.mmu, &mut self.timer) {
                    Ok(cycles) => cycles,
                    Err(AyyError::WriteToReadOnlyMemory { address, data }) => {
                        warn!(
                            "PC @ {:04x} => Attempted to write {:02x} to unmapped read-only memory at {:04x}",
                            self.cpu.read_register16(&Register::PC),
                            data,
                            address
                        );
                        0
                    }
                    Err(AyyError::OutOfBoundsMemoryAccess { address }) => {
                        warn!(
                            "PC @ {:04x} => Attempted to read out-of-bounds memory at {:04x}",
                            self.cpu.read_register16(&Register::PC),
                            address
                        );
                        0
                    }
                    Err(AyyError::WriteToDisabledExternalRam { address, data }) => {
                        error!(
                            "PC @ {:04x} => Attempted to write {:02x} to disabled external RAM at {:04x}",
                            self.cpu.read_register16(&Register::PC),
                            data,
                            address
                        );
                        0
                    }
                    Err(e) => panic!("{}", e),
                };

                self.mmu.apu.tick(cycles);
                self.timer.tick(&mut self.mmu, cycles);

                if self.cpu.elapsed_cycles() >= 456 {
                    self.cpu.reset_cycles(self.cpu.elapsed_cycles() - 456);
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

    pub fn dbg_render_tileset(&mut self) -> Vec<Tile> {
        self.ppu.render_tileset(&self.mmu)
    }

    pub fn dbg_render_background_tilemap(&mut self) -> Vec<Tile> {
        self.ppu.render_background_tilemap(&self.mmu)
    }

    pub fn dbg_render_window_tilemap(&mut self) -> Vec<Tile> {
        self.ppu.render_window_tilemap(&self.mmu)
    }
}
