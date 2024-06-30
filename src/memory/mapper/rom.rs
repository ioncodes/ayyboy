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
    fn read(&self, addr: u16) -> Result<u8, AyyError> {
        Ok(self.memory[addr as usize])
    }

    fn write(&mut self, addr: u16, data: u8) -> Result<(), AyyError> {
        // We simply only have a ROM. Writing to it is not allowed.
        Err(AyyError::WriteToReadOnlyMemory { address: addr, data })
    }

    fn current_rom_bank(&self) -> u8 {
        0
    }

    fn name(&self) -> String {
        String::from("ROM")
    }
}
