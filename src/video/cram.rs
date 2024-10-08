use log::error;

use crate::memory::addressable::Addressable;
use crate::memory::{
    BACKGROUND_PALETTE_DATA_REGISTER, BACKGROUND_PALETTE_INDEX_REGISTER, OBJECT_PALETTE_DATA_REGISTER,
    OBJECT_PALETTE_INDEX_REGISTER,
};

pub struct Cram {
    background_palette: [u8; 64],
    object_palette: [u8; 64],
    auto_increment: bool,
    obj_address: u8,
    bg_address: u8,
}

impl Cram {
    pub fn new() -> Cram {
        Cram {
            background_palette: [0; 64],
            object_palette: [0; 64],
            auto_increment: false,
            obj_address: 0,
            bg_address: 0,
        }
    }

    pub fn fetch_bg(&self, slot: u8, index: u8) -> u16 {
        (self.background_palette[((slot * 8) + index + 1) as usize] as u16) << 8
            | self.background_palette[((slot * 8) + index) as usize] as u16
    }

    pub fn fetch_obj(&self, slot: u8, index: u8) -> u16 {
        (self.object_palette[((slot * 8) + index + 1) as usize] as u16) << 8
            | self.object_palette[((slot * 8) + index) as usize] as u16
    }
}

impl Addressable for Cram {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            BACKGROUND_PALETTE_INDEX_REGISTER => {
                (((self.auto_increment as u16) << 7) as u8) | (self.bg_address & 0b0011_1111)
            }
            OBJECT_PALETTE_INDEX_REGISTER => {
                (((self.auto_increment as u16) << 7) as u8) | (self.obj_address & 0b0011_1111)
            }
            BACKGROUND_PALETTE_DATA_REGISTER => self.background_palette[self.bg_address as usize],
            OBJECT_PALETTE_DATA_REGISTER => self.object_palette[self.obj_address as usize],
            _ => {
                error!("Unmapped read from CRAM address {:04x}", addr);
                0xff
            }
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            BACKGROUND_PALETTE_INDEX_REGISTER => {
                self.auto_increment = data & 0b1000_0000 != 0;
                self.bg_address = data & 0b0011_1111;
            }
            OBJECT_PALETTE_INDEX_REGISTER => {
                self.auto_increment = data & 0b1000_0000 != 0;
                self.obj_address = data & 0b0011_1111;
            }
            BACKGROUND_PALETTE_DATA_REGISTER => {
                self.background_palette[self.bg_address as usize] = data;
                if self.auto_increment {
                    self.bg_address = (self.bg_address.wrapping_add(1)) & 0b0011_1111;
                }
            }
            OBJECT_PALETTE_DATA_REGISTER => {
                self.object_palette[self.obj_address as usize] = data;
                if self.auto_increment {
                    self.obj_address = (self.obj_address.wrapping_add(1)) & 0b0011_1111;
                }
            }
            _ => error!("Unmapped write to CRAM address {:04x} with data {:02x}", addr, data),
        }
    }
}
