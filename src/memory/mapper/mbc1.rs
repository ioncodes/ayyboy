use crate::error::AyyError;
use crate::memory::mapper::Mapper;
use crate::memory::{EXTERNAL_RAM_END, EXTERNAL_RAM_START};
use log::{debug, warn};

const RAM_ENABLE_START: u16 = 0x0000;
const RAM_ENABLE_END: u16 = 0x1fff;
const SECONDARY_BANK_REGISTER_START: u16 = 0x4000;
const SECONDARY_BANK_REGISTER_END: u16 = 0x5fff;
const ROM_BANK_START: u16 = 0x2000;
const ROM_BANK_END: u16 = 0x3fff;
const ROM_SLOT_0_START: u16 = 0x0000;
const ROM_SLOT_0_END: u16 = 0x3fff;
const ROM_SLOT_1_START: u16 = 0x4000;
const ROM_SLOT_1_END: u16 = 0x7fff;
const BANKING_MODE_START: u16 = 0x6000;
const BANKING_MODE_END: u16 = 0x7fff;

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
        let secondary_banking_allowed = memory.len() > 0x80000; // 512 KiB ROM

        Mbc1 {
            rom: memory,
            rom_bank: 1,
            ram: vec![0; 0x8000],
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
            ROM_SLOT_0_START..=ROM_SLOT_0_END => Ok(self.rom[addr as usize]),
            ROM_SLOT_1_START..=ROM_SLOT_1_END => {
                let rom_addr = (addr as usize % 0x4000) + (self.rom_bank as usize * 0x4000);
                if rom_addr < self.rom.len() {
                    Ok(self.rom[rom_addr])
                } else {
                    Err(AyyError::OutOfBoundsMemoryAccess { address: addr })
                }
            }
            EXTERNAL_RAM_START..=EXTERNAL_RAM_END if self.ram_enabled => {
                let base_addr = (addr - EXTERNAL_RAM_START) as usize;
                let ram_addr = base_addr + (self.ram_bank as usize * 0x2000);
                Ok(self.ram[ram_addr])
            }
            _ => Err(AyyError::OutOfBoundsMemoryAccess { address: addr }),
        }
    }

    #[inline]
    fn write(&mut self, addr: u16, data: u8) -> Result<(), AyyError> {
        match addr {
            RAM_ENABLE_START..=RAM_ENABLE_END => {
                self.ram_enabled = (data & 0x0f) == 0x0a;
                debug!("MBC1: RAM enabled: {}", self.ram_enabled);
            }
            ROM_BANK_START..=ROM_BANK_END => {
                self.rom_bank = data & 0b0001_1111;
                if self.rom_bank == 0 {
                    self.rom_bank = 1;
                }
                debug!("MBC1: Switched to ROM bank {}", self.rom_bank);
            }
            SECONDARY_BANK_REGISTER_START..=SECONDARY_BANK_REGISTER_END if self.banking_mode => {
                if self.secondary_banking_allowed {
                    self.rom_bank = (self.rom_bank & 0b0001_1111) | ((data & 0b11) << 5);
                    debug!("MBC1: Switched to ROM bank {}", self.rom_bank);
                } else {
                    warn!("MBC1: Attempted to switch to ROM bank, but not allowed");
                }
            }
            SECONDARY_BANK_REGISTER_START..=SECONDARY_BANK_REGISTER_END if !self.banking_mode => {
                self.ram_bank = data & 0b11;
                debug!("MBC1: Switched to RAM bank {}", self.ram_bank);
            }
            BANKING_MODE_START..=BANKING_MODE_END => {
                self.banking_mode = data & 0b0000_0001 == 1;
                debug!("MBC1: Switched to banking mode: {}", self.banking_mode);
            }
            EXTERNAL_RAM_START..=EXTERNAL_RAM_END => {
                if self.ram_enabled {
                    let base_addr = (addr - EXTERNAL_RAM_START) as usize;
                    let ram_addr = base_addr + (self.ram_bank as usize * 0x2000);
                    self.ram[ram_addr] = data;
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
