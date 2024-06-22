use crate::memory::mapper::rom::Rom;
use crate::memory::mapper::Mapper;

// The last instruction unmaps the boot ROM. Execution continues normally,
// thus entering cartridge entrypoint at $100
const BOOTROM_SIZE: u16 = 0xff;

#[derive(Clone)]
pub struct Mmu {
    cartridge: Box<dyn Mapper>,
    memory: Vec<u8>,
    bootrom_mapped: bool,
    bootrom: Vec<u8>,
}

impl Mmu {
    pub fn new(bootrom: Vec<u8>, cartridge: Vec<u8>) -> Mmu {
        Mmu {
            cartridge: Box::new(Rom::new(cartridge)),
            memory: Vec::<u8>::with_capacity(0xffff),
            bootrom_mapped: true,
            bootrom,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=BOOTROM_SIZE if self.bootrom_mapped => self.bootrom[addr as usize],
            0x0000..=0x7fff => self.cartridge.read(addr),
            _ => self.memory[addr as usize],
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=BOOTROM_SIZE if self.bootrom_mapped => self.bootrom[addr as usize] = data,
            0x0000..=0x7fff => self.cartridge.write(addr, data),
            _ => self.memory[addr as usize] = data,
        }
    }

    pub fn read16(&self, addr: u16) -> u16 {
        let lo = self.read(addr) as u16;
        let hi = self.read(addr + 1) as u16;
        (hi << 8) | lo
    }

    pub fn write16(&mut self, addr: u16, data: u16) {
        let lo = data as u8;
        let hi = (data >> 8) as u8;
        self.write(addr, lo);
        self.write(addr + 1, hi);
    }

    pub fn unmap_bootrom(&mut self) {
        self.bootrom_mapped = false;
    }

    pub fn resize_memory(&mut self, size: usize) {
        self.memory.resize(size, 0);
    }
}
