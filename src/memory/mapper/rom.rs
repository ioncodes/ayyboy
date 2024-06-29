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
    fn read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    fn current_rom_bank(&self) -> u8 {
        0
    }
}
