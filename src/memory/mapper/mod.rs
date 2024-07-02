use crate::error::AyyError;
use dyn_clone::DynClone;

pub mod mbc1;
pub mod rom;

pub trait Mapper: DynClone {
    fn read(&self, addr: u16) -> Result<u8, AyyError>;
    fn write(&mut self, addr: u16, data: u8) -> Result<(), AyyError>;
    fn current_rom_bank(&self) -> u8;
    fn current_ram_bank(&self) -> u8;
    fn name(&self) -> String;

    fn read16(&self, addr: u16) -> Result<u16, AyyError> {
        let lo = self.read(addr)? as u16;
        let hi = self.read(addr + 1)? as u16;
        Ok((hi << 8) | lo)
    }

    fn write16(&mut self, addr: u16, data: u16) -> Result<(), AyyError> {
        let lo = data as u8;
        let hi = (data >> 8) as u8;
        self.write(addr, lo)?;
        self.write(addr + 1, hi)?;
        Ok(())
    }
}

dyn_clone::clone_trait_object!(Mapper);
