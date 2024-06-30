use crate::error::AyyError;
use crate::memory::mapper::Mapper;
use crate::memory::{BOOTROM_MAPPER_REGISTER, JOYPAD_REGISTER, OAM_DMA_REGISTER};
use log::error;

// The last instruction unmaps the boot ROM. Execution continues normally,
// thus entering cartridge entrypoint at $100
const BOOTROM_SIZE: u16 = 0xff;

#[derive(Clone)]
pub struct Mmu {
    cartridge: Box<dyn Mapper>,
    memory: Vec<u8>,
    bootrom: Vec<u8>,
}

impl Mmu {
    pub fn new(bootrom: Vec<u8>, cartridge: Box<dyn Mapper>) -> Mmu {
        Mmu {
            //cartridge: Box::new(Rom::new(cartridge)),
            cartridge,
            memory: vec![0; 0x10000],
            bootrom,
        }
    }

    pub fn read(&self, addr: u16) -> Result<u8, AyyError> {
        // if joypad is read, spoof no buttons pressed
        // THIS MAY CAUSE ISSUES WITH THE UNIT TESTS
        if addr == JOYPAD_REGISTER {
            return Ok(self.memory[addr as usize] | 0xf);
        }

        match addr {
            0x0000..=BOOTROM_SIZE if self.is_bootrom_mapped() => Ok(self.bootrom[addr as usize]),
            0x0000..=0x7fff => self.cartridge.read(addr),
            _ => Ok(self.memory[addr as usize]),
        }
    }

    pub fn read_as<T>(&self, addr: u16) -> Result<T, AyyError>
    where
        T: From<u8>,
    {
        Ok(T::from(self.read(addr)?))
    }

    pub fn write(&mut self, addr: u16, data: u8) -> Result<(), AyyError> {
        match addr {
            OAM_DMA_REGISTER => error!("OAM DMA not implemented!"),
            0x0000..=BOOTROM_SIZE if self.is_bootrom_mapped() => self.bootrom[addr as usize] = data,
            0x0000..=0x7fff => self.cartridge.write(addr, data)?,
            _ => self.memory[addr as usize] = data,
        }

        Ok(())
    }

    pub fn read16(&self, addr: u16) -> Result<u16, AyyError> {
        let lo = self.read(addr)? as u16;
        let hi = self.read(addr.wrapping_add(1))? as u16;
        Ok((hi << 8) | lo)
    }

    pub fn write16(&mut self, addr: u16, data: u16) -> Result<(), AyyError> {
        let lo = data as u8;
        let hi = (data >> 8) as u8;
        self.write(addr, lo)?;
        self.write(addr.wrapping_add(1), hi)?;
        Ok(())
    }

    pub fn read_unchecked(&self, addr: u16) -> u8 {
        self.read(addr).unwrap()
    }

    pub fn write_unchecked(&mut self, addr: u16, data: u8) {
        self.write(addr, data).unwrap();
    }

    pub fn read_as_unchecked<T>(&self, addr: u16) -> T
    where
        T: From<u8>,
    {
        self.read_as(addr).unwrap()
    }

    pub fn read16_unchecked(&self, addr: u16) -> u16 {
        self.read16(addr).unwrap()
    }

    pub fn write16_unchecked(&mut self, addr: u16, data: u16) {
        self.write16(addr, data).unwrap();
    }

    pub fn is_bootrom_mapped(&self) -> bool {
        self.read(BOOTROM_MAPPER_REGISTER).unwrap() == 0x00
    }

    pub fn current_rom_bank(&self) -> u8 {
        self.cartridge.current_rom_bank()
    }

    #[cfg(test)]
    pub fn resize_memory(&mut self, size: usize) {
        self.memory.resize(size, 0);
    }

    #[cfg(test)]
    pub fn unmap_bootrom(&mut self) {
        let _ = self.write(BOOTROM_MAPPER_REGISTER, 0x69);
    }
}
