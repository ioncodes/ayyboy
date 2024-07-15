use log::error;

use super::Mapper;

#[derive(Clone)]
pub struct Mbc5 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank: u16,
    ram_bank: u8,
    ram_enabled: bool,
}

impl Mbc5 {
    pub fn new(memory: Vec<u8>) -> Mbc5 {
        Mbc5 {
            rom: memory,
            ram: vec![0; 0x8000],
            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,
        }
    }
}

impl Mapper for Mbc5 {
    #[inline]
    fn read(&self, addr: u16) -> Result<u8, crate::error::AyyError> {
        match addr {
            0x0000..=0x3fff => Ok(self.rom[addr as usize]),
            0x4000..=0x7fff => {
                let addr = (addr as usize % 0x4000) + (self.rom_bank as usize * 0x4000);
                Ok(self.rom[addr])
            }
            0xa000..=0xbfff if self.ram_enabled => {
                let base_addr = (addr - 0xa000) as usize;
                let addr = base_addr + (self.ram_bank as usize * 0x2000);
                Ok(self.ram[addr])
            }
            0xa000..=0xbfff if !self.ram_enabled => {
                error!(
                    "MBC5: Attempted read from RAM bank {} while RAM is disabled",
                    self.ram_bank
                );
                Ok(0)
            }
            _ => {
                error!("MBC5: Unmapped read from address {:04x}", addr);
                Ok(0)
            }
        }
    }

    #[inline]
    fn write(&mut self, addr: u16, data: u8) -> Result<(), crate::error::AyyError> {
        match addr {
            0x0000..=0x1fff => {
                self.ram_enabled = data & 0x0f == 0x0a;
                Ok(())
            }
            0x2000..=0x2fff => {
                self.rom_bank = (self.rom_bank & 0x100) | data as u16;
                Ok(())
            }
            0x3000..=0x3fff => {
                self.rom_bank = (self.rom_bank & 0xff) | ((data as u16 & 0x1) << 8);
                Ok(())
            }
            0x4000..=0x5fff => {
                self.ram_bank = data & 0x0f;
                Ok(())
            }
            0xa000..=0xbfff if self.ram_enabled => {
                let base_addr = (addr - 0xa000) as usize;
                let addr = base_addr + (self.ram_bank as usize * 0x2000);
                self.ram[addr] = data;
                Ok(())
            }
            0xa000..=0xbfff if !self.ram_enabled => {
                error!(
                    "MBC5: Attempted write to RAM bank {} while RAM is disabled",
                    self.ram_bank
                );
                Ok(())
            }
            _ => {
                error!("MBC5: Unmapped write to address {:04x}", addr);
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
        String::from("MBC5")
    }
}
