use crate::error::AyyError;
use crate::memory::mapper::Mapper;
use crate::memory::{BOOTROM_MAPPER_REGISTER, JOYPAD_REGISTER, OAM_DMA_REGISTER};
use log::debug;

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
            cartridge,
            memory: vec![0; 0x10000],
            bootrom,
        }
    }

    #[inline]
    pub fn read(&self, addr: u16) -> Result<u8, AyyError> {
        // if joypad is read, spoof no buttons pressed
        // THIS MAY CAUSE ISSUES WITH THE UNIT TESTS

        match addr {
            0x0000..=BOOTROM_SIZE if self.is_bootrom_mapped() => Ok(self.bootrom[addr as usize]),
            0x0000..=0x7fff => self.cartridge.read(addr),
            JOYPAD_REGISTER => Ok(self.memory[addr as usize] | 0xf),
            _ => Ok(self.memory[addr as usize]),
        }
    }

    #[inline]
    pub fn read_as<T>(&self, addr: u16) -> Result<T, AyyError>
    where
        T: From<u8>,
    {
        Ok(T::from(self.read(addr)?))
    }

    #[inline]
    pub fn read16(&self, addr: u16) -> Result<u16, AyyError> {
        let lo = self.read(addr)? as u16;
        let hi = self.read(addr.wrapping_add(1))? as u16;
        Ok((hi << 8) | lo)
    }

    #[inline]
    pub fn read_unchecked(&self, addr: u16) -> u8 {
        self.read(addr).unwrap()
    }

    #[inline]
    pub fn read_as_unchecked<T>(&self, addr: u16) -> T
    where
        T: From<u8>,
    {
        self.read_as(addr).unwrap()
    }

    #[inline]
    pub fn read16_unchecked(&self, addr: u16) -> u16 {
        self.read16(addr).unwrap()
    }

    #[inline]
    pub fn write16_unchecked(&mut self, addr: u16, data: u16) {
        self.write16(addr, data).unwrap();
    }

    #[inline]
    pub fn write(&mut self, addr: u16, data: u8) -> Result<(), AyyError> {
        match addr {
            0x0000..=BOOTROM_SIZE if self.is_bootrom_mapped() => self.bootrom[addr as usize] = data,
            0x0000..=0x7fff => self.cartridge.write(addr, data)?,
            OAM_DMA_REGISTER => self.start_dma_transfer(data)?,
            _ => self.memory[addr as usize] = data,
        }

        Ok(())
    }

    #[inline]
    pub fn write16(&mut self, addr: u16, data: u16) -> Result<(), AyyError> {
        let lo = data as u8;
        let hi = (data >> 8) as u8;
        self.write(addr, lo)?;
        self.write(addr.wrapping_add(1), hi)?;
        Ok(())
    }

    #[inline]
    pub fn write_unchecked(&mut self, addr: u16, data: u8) {
        self.write(addr, data).unwrap();
    }

    pub fn is_bootrom_mapped(&self) -> bool {
        self.read(BOOTROM_MAPPER_REGISTER).unwrap() == 0x00
    }

    pub fn current_rom_bank(&self) -> u8 {
        self.cartridge.current_rom_bank()
    }

    fn start_dma_transfer(&mut self, data: u8) -> Result<(), AyyError> {
        let src_addr = (data as u16) << 8;
        debug!("OAM DMA transfer from ${:04x}", src_addr);

        // TODO: Is this range correct?
        // TODO: Add cycles
        for i in 0..0xa0 {
            let byte = self.read(src_addr + i)?;
            self.write(0xfe00 + i, byte)?;
        }

        Ok(())
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
