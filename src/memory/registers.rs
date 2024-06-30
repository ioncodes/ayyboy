use crate::error::AyyError;
use bitflags::bitflags;

bitflags! {
    pub struct InterruptFlags: u8 {
        const VBLANK    = 0b00001;
        const LCD_STAT  = 0b00010;
        const TIMER     = 0b00100;
        const SERIAL    = 0b01000;
        const JOYPAD    = 0b10000;
    }
}

bitflags! {
    pub struct InterruptEnable: u8 {
        const VBLANK    = 0b00001;
        const LCD_STAT  = 0b00010;
        const TIMER     = 0b00100;
        const SERIAL    = 0b01000;
        const JOYPAD    = 0b10000;
    }
}

bitflags! {
    pub struct LcdControl: u8 {
        const BG_DISPLAY = 0b0000_0001;
        const OBJ_DISPLAY = 0b0000_0010;
        const OBJ_SIZE = 0b0000_0100;
        const BG_TILE_MAP = 0b0000_1000;
        const BG_TILE_DATA = 0b0001_0000;
        const WINDOW_DISPLAY = 0b0010_0000;
        const WINDOW_TILE_MAP = 0b0100_0000;
        const LCD_DISPLAY = 0b1000_0000;
    }
}

impl From<u8> for InterruptFlags {
    fn from(byte: u8) -> Self {
        Self::from_bits_truncate(byte)
    }
}

impl From<u8> for InterruptEnable {
    fn from(byte: u8) -> Self {
        Self::from_bits_truncate(byte)
    }
}

impl From<u8> for LcdControl {
    fn from(byte: u8) -> Self {
        Self::from_bits_truncate(byte)
    }
}

impl InterruptFlags {
    pub fn to_vector(&self) -> Result<u16, AyyError> {
        if self.contains(InterruptFlags::VBLANK) {
            Ok(0x0040)
        } else if self.contains(InterruptFlags::LCD_STAT) {
            Ok(0x0048)
        } else if self.contains(InterruptFlags::TIMER) {
            Ok(0x0050)
        } else if self.contains(InterruptFlags::SERIAL) {
            Ok(0x0058)
        } else if self.contains(InterruptFlags::JOYPAD) {
            Ok(0x0060)
        } else {
            Err(AyyError::UnknownIrqVector { vector: self.bits() })
        }
    }
}
