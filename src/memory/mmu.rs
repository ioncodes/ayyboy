use crate::memory::mapper::rom::Rom;
use crate::memory::mapper::Mapper;

pub struct Mmu {
    cartridge: Box<dyn Mapper>,
}

impl Mmu {
    pub fn new(rom: Vec<u8>) -> Mmu {
        Mmu {
            cartridge: Box::new(Rom::new(rom)),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.cartridge.read(addr)
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.cartridge.write(addr, data);
    }

    pub fn read16(&self, addr: u16) -> u16 {
        self.cartridge.read16(addr)
    }

    pub fn write16(&mut self, addr: u16, data: u16) {
        self.cartridge.write16(addr, data);
    }
}
