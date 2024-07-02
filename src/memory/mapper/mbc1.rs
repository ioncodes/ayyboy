use crate::error::AyyError;
use crate::memory::mapper::Mapper;
use crate::memory::{EXTERNAL_RAM_END, EXTERNAL_RAM_START};
use log::debug;

const RAM_ENABLE_RANGE: std::ops::RangeInclusive<u16> = 0x0000..=0x1fff;
const RAM_BANK_RANGE: std::ops::RangeInclusive<u16> = 0x4000..=0x5fff;
const ROM_BANK_RANGE: std::ops::RangeInclusive<u16> = 0x2000..=0x3fff;
const ROM_SLOT_0_RANGE: std::ops::RangeInclusive<u16> = 0x0000..=0x3fff;
const ROM_SLOT_1_RANGE: std::ops::RangeInclusive<u16> = 0x4000..=0x7fff;

#[derive(Clone)]
pub struct Mbc1 {
    rom: Vec<u8>,
    rom_bank: u8,
    ram: Vec<u8>,
    ram_bank: u8,
    ram_enabled: bool,
}

impl Mbc1 {
    pub fn new(memory: Vec<u8>) -> Mbc1 {
        Mbc1 {
            rom: memory,
            rom_bank: 1,
            ram: vec![0; 0x2000],
            ram_bank: 0,
            ram_enabled: false,
        }
    }
}

impl Mapper for Mbc1 {
    fn read(&self, addr: u16) -> Result<u8, AyyError> {
        match addr {
            addr if ROM_SLOT_0_RANGE.contains(&addr) => Ok(self.rom[addr as usize]),
            addr if ROM_SLOT_1_RANGE.contains(&addr) => {
                let addr = (addr as usize % 0x4000) + (self.rom_bank as usize * 0x4000);
                Ok(self.rom[addr])
            }
            addr if addr >= EXTERNAL_RAM_START && addr <= EXTERNAL_RAM_END => {
                if self.ram_enabled {
                    let base_addr = (addr - EXTERNAL_RAM_START) as usize;
                    let addr = base_addr + (self.ram_bank as usize * 0x2000);
                    Ok(self.ram[addr])
                } else {
                    Err(AyyError::OutOfBoundsMemoryAccess { address: addr })
                }
            }
            _ => Err(AyyError::OutOfBoundsMemoryAccess { address: addr }),
        }
    }

    fn write(&mut self, addr: u16, data: u8) -> Result<(), AyyError> {
        match addr {
            addr if RAM_ENABLE_RANGE.contains(&addr) => {
                self.ram_enabled = (data & 0x0f) == 0x0a;
                debug!("MBC1: RAM enabled: {}", self.ram_enabled);
            }
            addr if ROM_BANK_RANGE.contains(&addr) => {
                // This 5-bit register (range $01-$1F) selects the ROM bank number for the 4000–7FFF region.
                // Higher bits are discarded — writing $E1 (binary 11100001) to this register would select bank $01.
                self.rom_bank = data & 0b0001_1111;
                if self.rom_bank == 0 {
                    self.rom_bank = 1;
                }
                debug!("MBC1: Switched to RAM bank {}", self.ram_bank);
            }
            addr if RAM_BANK_RANGE.contains(&addr) => {
                // This 5-bit register (range $01-$1F) selects the ROM bank number for the 4000–7FFF region.
                // Higher bits are discarded — writing $E1 (binary 11100001) to this register would select bank $01.
                self.ram_bank = data & 0b11;
                debug!("MBC1: Switched to ROM bank {}", self.rom_bank);
            }
            addr if addr >= EXTERNAL_RAM_START && addr <= EXTERNAL_RAM_END => {
                if self.ram_enabled {
                    let base_addr = (addr - EXTERNAL_RAM_START) as usize;
                    let addr = base_addr + (self.ram_bank as usize * 0x2000);
                    self.ram[addr] = data;
                } else {
                    return Err(AyyError::WriteToDisabledExternalRam { address: addr, data });
                }
            }
            _ => return Err(AyyError::WriteToReadOnlyMemory { address: addr, data }),
        }

        Ok(())
    }

    fn current_rom_bank(&self) -> u8 {
        self.rom_bank
    }

    fn current_ram_bank(&self) -> u8 {
        self.ram_bank
    }

    fn name(&self) -> String {
        String::from("MBC1")
    }
}
