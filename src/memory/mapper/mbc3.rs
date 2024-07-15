use log::{error, trace};

use crate::memory::mapper::Mapper;

#[derive(Clone)]
pub struct Mbc3 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank: u16,
    ram_bank: u8,
    ram_enabled: bool,
    rtc_mapped: bool, // TODO: fake
}

impl Mbc3 {
    pub fn new(memory: Vec<u8>) -> Mbc3 {
        Mbc3 {
            rom: memory,
            ram: vec![0; 0x8000],
            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,
            rtc_mapped: false,
        }
    }
}

impl Mapper for Mbc3 {
    #[inline]
    fn read(&self, addr: u16) -> Result<u8, crate::error::AyyError> {
        match addr {
            0x0000..=0x3fff => Ok(self.rom[addr as usize]),
            0x4000..=0x7fff => {
                let addr = (addr as usize % 0x4000) + (self.rom_bank as usize * 0x4000);
                Ok(self.rom[addr])
            }
            0xa000..=0xbfff if self.rtc_mapped => {
                // TODO: This needs precedence over RAM
                error!("MBC3: Faking unmapped RTC register read");
                Ok(0x00)
            }
            0xa000..=0xbfff if self.ram_enabled => {
                let base_addr = (addr - 0xa000) as usize;
                let addr = base_addr + (self.ram_bank as usize * 0x2000);
                Ok(self.ram[addr])
            }
            _ => {
                error!("MBC3: Unmapped read from address {:04x}", addr);
                Ok(0x00)
            }
        }
    }

    #[inline]
    fn write(&mut self, addr: u16, data: u8) -> Result<(), crate::error::AyyError> {
        match addr {
            0x0000..=0x1fff => {
                self.ram_enabled = data & 0x0f == 0x0a;
                // TODO: enable RTC
                trace!("MBC3: RAM access toggled to {}", self.ram_enabled);
                Ok(())
            }
            0x2000..=0x3fff => {
                self.rom_bank = (data & 0b0111_1111) as u16;
                if self.rom_bank == 0 {
                    self.rom_bank = 1;
                }
                trace!("MBC3: Switched to ROM bank {}", self.rom_bank);
                Ok(())
            }
            0x4000..=0x5fff if data <= 0x03 => {
                // only RAM bank 1-3 allowed, rest goes to RTC
                self.rtc_mapped = false;
                self.ram_bank = data & 0x0f;
                trace!("MBC3: Switched to RAM bank {}", self.ram_bank);
                Ok(())
            }
            0x4000..=0x5fff if data > 0x03 => {
                error!("MBC3: Faking unmapped RTC register select {}", data);
                self.rtc_mapped = true;
                Ok(())
            }
            0xa000..=0xbfff => {
                if self.ram_enabled {
                    let base_addr = (addr - 0xa000) as usize;
                    let addr = base_addr + (self.ram_bank as usize * 0x2000);
                    self.ram[addr] = data;
                } else {
                    error!(
                        "MBC3: Attempted write to RAM bank {} while RAM is disabled",
                        self.ram_bank
                    );
                }
                Ok(())
            }
            _ => {
                error!("MBC3: Unmapped write to address {:04x} with data {:02x}", addr, data);
                Ok(())
            }
        }
    }

    fn dump_ram(&self) -> Vec<u8> {
        self.ram.clone()
    }

    fn load_ram(&mut self, ram: Vec<u8>) {
        self.ram = ram;
    }

    #[inline]
    fn current_rom_bank(&self) -> u16 {
        self.rom_bank
    }

    #[inline]
    fn current_ram_bank(&self) -> u8 {
        self.ram_bank
    }

    #[inline]
    fn name(&self) -> String {
        String::from("MBC3")
    }
}
