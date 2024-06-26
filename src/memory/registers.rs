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
