use crate::memory::mapper::Mapper;
use log::warn;

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
    fn read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn write(&mut self, addr: u16, data: u8) {
        // We simply only have a ROM. Writing to it is not allowed.
        warn!("Attempted to write to ROM at address {:04x}", addr);
    }

    fn current_rom_bank(&self) -> u8 {
        0
    }

    fn name(&self) -> String {
        String::from("ROM")
    }
}
