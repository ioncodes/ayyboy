use crate::error::AyyError;
use crate::memory::mapper::Mapper;

#[derive(Clone)]
pub struct Rom {
    memory: Vec<u8>,
}

impl Rom {
    pub fn new(memory: Vec<u8>) -> Rom {
        Rom { memory }
    }
}

impl Mapper for Rom {
    #[inline]
    fn read(&self, addr: u16) -> Result<u8, AyyError> {
        Ok(self.memory[addr as usize])
    }

    #[inline]
    fn write(&mut self, addr: u16, data: u8) -> Result<(), AyyError> {
        // We simply only have a ROM. Writing to it is not allowed.
        Err(AyyError::WriteToReadOnlyMemory { address: addr, data })
    }

    fn dump_ram(&self) -> Vec<u8> {
        Vec::new()
    }

    fn load_ram(&mut self, _ram: Vec<u8>) {}

    #[inline]
    fn current_rom_bank(&self) -> u16 {
        0
    }

    #[inline]
    fn current_ram_bank(&self) -> u8 {
        0
    }

    #[inline]
    fn name(&self) -> String {
        String::from("ROM")
    }
}
