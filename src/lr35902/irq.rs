use crate::memory::registers::{InterruptEnable, InterruptFlags};

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
    pub fn from_flags(interrupt_enable: &InterruptEnable, interrupt_flags: &InterruptFlags) -> Vector {
        if interrupt_enable.contains(InterruptEnable::VBLANK) && interrupt_flags.contains(InterruptFlags::VBLANK) {
            return Vector::VBlank;
        }

        if interrupt_enable.contains(InterruptEnable::STAT) && interrupt_flags.contains(InterruptFlags::STAT) {
            return Vector::Stat;
        }

        if interrupt_enable.contains(InterruptEnable::TIMER) && interrupt_flags.contains(InterruptFlags::TIMER) {
            return Vector::Timer;
        }

        if interrupt_enable.contains(InterruptEnable::SERIAL) && interrupt_flags.contains(InterruptFlags::SERIAL) {
            return Vector::Serial;
        }

        if interrupt_enable.contains(InterruptEnable::JOYPAD) && interrupt_flags.contains(InterruptFlags::JOYPAD) {
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
