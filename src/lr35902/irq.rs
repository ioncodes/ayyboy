use crate::memory::registers::InterruptFlags;

#[derive(Clone)]
pub struct Ime {
    pub enabled: bool,
    pub enable_pending: bool,
}

pub enum Vector {
    VBlank,
    Stat,
    Timer,
    Serial,
    Joypad,
}

impl Vector {
    pub fn from_flags(flags: &InterruptFlags) -> Vector {
        if flags.contains(InterruptFlags::VBLANK) {
            return Vector::VBlank;
        } else if flags.contains(InterruptFlags::STAT) {
            return Vector::Stat;
        } else if flags.contains(InterruptFlags::TIMER) {
            return Vector::Timer;
        } else if flags.contains(InterruptFlags::SERIAL) {
            return Vector::Serial;
        } else if flags.contains(InterruptFlags::JOYPAD) {
            return Vector::Joypad;
        }

        unreachable!();
    }

    pub fn to_address(&self) -> u16 {
        match self {
            Vector::VBlank => 0x0040,
            Vector::Stat => 0x0048,
            Vector::Timer => 0x0050,
            Vector::Serial => 0x0058,
            Vector::Joypad => 0x0060,
        }
    }
}

impl std::fmt::Display for Vector {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Vector::VBlank => write!(f, "VBLANK"),
            Vector::Stat => write!(f, "STAT"),
            Vector::Timer => write!(f, "TIMER"),
            Vector::Serial => write!(f, "SERIAL"),
            Vector::Joypad => write!(f, "JOYPAD"),
        }
    }
}
