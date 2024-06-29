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
