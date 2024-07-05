use bitflags::bitflags;

bitflags! {
    pub struct InterruptFlags: u8 {
        const VBLANK = 0b00001;
        const STAT   = 0b00010;
        const TIMER  = 0b00100;
        const SERIAL = 0b01000;
        const JOYPAD = 0b10000;
    }
}

bitflags! {
    pub struct InterruptEnable: u8 {
        const VBLANK = 0b00001;
        const STAT   = 0b00010;
        const TIMER  = 0b00100;
        const SERIAL = 0b01000;
        const JOYPAD = 0b10000;
    }
}

bitflags! {
    pub struct LcdControl: u8 {
        const BG_DISPLAY      = 0b0000_0001;
        const OBJ_DISPLAY     = 0b0000_0010;
        const OBJ_SIZE        = 0b0000_0100;
        const BG_TILE_MAP     = 0b0000_1000;
        const BG_TILE_DATA    = 0b0001_0000;
        const WINDOW_DISPLAY  = 0b0010_0000;
        const WINDOW_TILE_MAP = 0b0100_0000;
        const LCD_DISPLAY     = 0b1000_0000;
    }
}

bitflags! {
    pub struct LcdStatus: u8 {
        const PPU_MODE            = 0b0000_0011;
        const LYC_EQ_LY_INTERRUPT = 0b0000_0100;
        const MODE_0_CONDITION    = 0b0000_1000;
        const MODE_1_CONDITION    = 0b0001_0000;
        const MODE_2_CONDITION    = 0b0010_0000;
        const LYC_EQ_LY_ENABLE    = 0b0100_0000;
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

impl From<u8> for LcdStatus {
    fn from(byte: u8) -> Self {
        Self::from_bits_truncate(byte)
    }
}
