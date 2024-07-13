use log::error;

use crate::memory::addressable::Addressable;
use crate::memory::{
    BACKGROUND_PALETTE_DATA_REGISTER, BACKGROUND_PALETTE_INDEX_REGISTER,
    OBJECT_PALETTE_DATA_REGISTER, OBJECT_PALETTE_INDEX_REGISTER,
};

pub struct Cram {
    pub background_palette: [u8; 64],
    pub object_palette: [u8; 64],
    auto_increment: bool,
    address: u8,
}

impl Cram {
    pub fn new() -> Cram {
        Cram {
            background_palette: [0; 64],
            object_palette: [0; 64],
            auto_increment: false,
            address: 0,
        }
    }
}

impl Addressable for Cram {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            BACKGROUND_PALETTE_INDEX_REGISTER => self.background_palette[self.address as usize],
            OBJECT_PALETTE_INDEX_REGISTER => self.object_palette[self.address as usize],
            _ => {
                error!("Unmapped read from CRAM address {:04x}", addr);
                0xff
            }
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            BACKGROUND_PALETTE_INDEX_REGISTER | OBJECT_PALETTE_INDEX_REGISTER => {
                self.auto_increment = data & 0b1000_0000 != 0;
                self.address = data & 0b0011_1111;
            }
            BACKGROUND_PALETTE_DATA_REGISTER => {
                self.background_palette[self.address as usize] = data;
                if self.auto_increment {
                    self.address = (self.address.wrapping_add(1)) & 0b0011_1111;
                }
            }
            OBJECT_PALETTE_DATA_REGISTER => {
                self.object_palette[self.address as usize] = data;
                if self.auto_increment {
                    self.address = (self.address.wrapping_add(1)) & 0b0011_1111;
                }
            }
            _ => error!(
                "Unmapped write to CRAM address {:04x} with data {:02x}",
                addr, data
            ),
        }
    }
}
