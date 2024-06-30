use crate::memory::mapper::Mapper;
use log::{debug, warn};

#[derive(Clone)]
pub struct Mbc1 {
    rom: Vec<u8>,
    rom_bank: u8,
}

impl Mbc1 {
    pub fn new(memory: Vec<u8>) -> Mbc1 {
        Mbc1 { rom: memory, rom_bank: 1 }
    }
}

impl Mapper for Mbc1 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3fff => self.rom[addr as usize],
            0x4000..=0x7fff => {
                let addr = (addr as usize % 0x4000) + (self.rom_bank as usize * 0x4000);
                self.rom[addr]
            }
            _ => panic!("Invalid read address: {:04x}", addr),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        if addr >= 0x2000 && addr <= 0x3fff {
            // This 5-bit register (range $01-$1F) selects the ROM bank number for the 4000–7FFF region.
            // Higher bits are discarded — writing $E1 (binary 11100001) to this register would select bank $01.
            self.rom_bank = data & 0b11111;
            if self.rom_bank == 0 {
                self.rom_bank = 1;
            }
            debug!("MBC1: Switched to ROM bank {}", self.rom_bank);
        } else {
            warn!("MBC1: Attempting to write to read-only memory at {:04x} with {:02x}", addr, data);
        }
    }

    fn current_rom_bank(&self) -> u8 {
        self.rom_bank
    }

    fn name(&self) -> String {
        String::from("MBC1")
    }
}
