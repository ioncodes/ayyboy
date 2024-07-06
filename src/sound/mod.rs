pub mod apu;
mod channels;

pub const SAMPLE_RATE: u32 = 44_100;

pub const MASTER_VOLUME_REGISTER: u16 = 0xff24; // NR50
pub const MASTER_CONTROL_REGISTER: u16 = 0xff26; // NR52

pub const NR21: u16 = 0xFF16;
pub const NR22: u16 = 0xFF17;
pub const NR23: u16 = 0xFF18;
pub const NR24: u16 = 0xFF19;
