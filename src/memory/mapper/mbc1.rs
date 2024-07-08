use crate::error::AyyError;
use crate::memory::mapper::Mapper;
use crate::memory::{EXTERNAL_RAM_END, EXTERNAL_RAM_START};
use log::{debug, warn};

const RAM_ENABLE_RANGE: std::ops::RangeInclusive<u16> = 0x0000..=0x1fff;
const SECONDARY_BANK_REGISTER: std::ops::RangeInclusive<u16> = 0x4000..=0x5fff;
const ROM_BANK_RANGE: std::ops::RangeInclusive<u16> = 0x2000..=0x3fff;
const ROM_SLOT_0_RANGE: std::ops::RangeInclusive<u16> = 0x0000..=0x3fff;
const ROM_SLOT_1_RANGE: std::ops::RangeInclusive<u16> = 0x4000..=0x7fff;
const BANKING_MODE_REGISTER: std::ops::RangeInclusive<u16> = 0x6000..=0x7fff;

#[derive(Clone)]
pub struct Mbc1 {
    rom: Vec<u8>,
    rom_bank: u8,
    ram: Vec<u8>,
    ram_bank: u8,
    ram_enabled: bool,
    banking_mode: bool,
    secondary_banking_allowed: bool,
}

impl Mbc1 {
    pub fn new(memory: Vec<u8>) -> Mbc1 {
        // If the cart is not large enough to use the 2-bit register
        // (≤ 8 KiB RAM and ≤ 512 KiB ROM) this mode select has no observable effect.
        let secondary_banking_allowed = if memory.len() > 0x80000 { true } else { false };

        Mbc1 {
            rom: memory,
            rom_bank: 1,
            ram: vec![0; 0x2000],
            ram_bank: 0,
            ram_enabled: false,
            banking_mode: false,
            secondary_banking_allowed,
        }
    }
}

impl Mapper for Mbc1 {
    #[inline]
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

    #[inline]
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
            addr if SECONDARY_BANK_REGISTER.contains(&addr) && !self.banking_mode => {
                // This 5-bit register (range $01-$1F) selects the ROM bank number for the 4000–7FFF region.
                // Higher bits are discarded — writing $E1 (binary 11100001) to this register would select bank $01.
                if self.secondary_banking_allowed {
                    self.ram_bank = data & 0b11;
                    debug!("MBC1: Switched to RAM bank {}", self.rom_bank);
                } else {
                    warn!("MBC1: Attempted to switch to RAM bank, but not allowed");
                }
            }
            addr if SECONDARY_BANK_REGISTER.contains(&addr) && self.banking_mode => {
                // or to specify the upper two bits (bits 5-6) of the ROM Bank number (1 MiB ROM or larger carts only).
                // If neither ROM nor RAM is large enough, setting this register does nothing.
                if self.secondary_banking_allowed {
                    self.rom_bank = (self.rom_bank & 0b0001_1111) | ((data & 0b11) << 5);
                    debug!("MBC1: Switched to ROM bank {}", self.ram_bank);
                } else {
                    warn!("MBC1: Attempted to switch to ROM bank, but not allowed");
                }
            }
            addr if BANKING_MODE_REGISTER.contains(&addr) => {
                self.banking_mode = data & 0b0000_0001 == 1;
                debug!("MBC1: Switched to banking mode: {}", self.banking_mode);
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

    fn dump_ram(&self) -> Vec<u8> {
        self.ram.clone()
    }

    fn load_ram(&mut self, ram: Vec<u8>) {
        self.ram = ram;
    }

    #[inline]
    fn current_rom_bank(&self) -> u8 {
        self.rom_bank
    }

    #[inline]
    fn current_ram_bank(&self) -> u8 {
        self.ram_bank
    }

    #[inline]
    fn name(&self) -> String {
        String::from("MBC1")
    }
}
