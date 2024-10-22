pub mod apu;
mod channels;
mod stereo;

// The audio sample rate
pub const SAMPLE_RATE: usize = 48_000;

// The size of the audio sample buffer
pub const BUFFER_SIZE: usize = 1024;

// The rate at which the CPU is ticked
pub const CPU_CLOCK: usize = 4194304;

// APU registers
pub const NR10: u16 = 0xff10;
pub const NR11: u16 = 0xff11;
pub const NR12: u16 = 0xff12;
pub const NR13: u16 = 0xff13;
pub const NR14: u16 = 0xff14;
pub const NR21: u16 = 0xff16;
pub const NR22: u16 = 0xff17;
pub const NR23: u16 = 0xff18;
pub const NR24: u16 = 0xff19;
pub const NR30: u16 = 0xff1a;
pub const NR31: u16 = 0xff1b;
pub const NR32: u16 = 0xff1c;
pub const NR33: u16 = 0xff1d;
pub const NR34: u16 = 0xff1e;
pub const NR41: u16 = 0xff20;
pub const NR42: u16 = 0xff21;
pub const NR43: u16 = 0xff22;
pub const NR44: u16 = 0xff23;
pub const NR50: u16 = 0xff24;
pub const NR51: u16 = 0xff25;
pub const NR52: u16 = 0xff26;
pub const WAVE_PATTERN_RAM_START: u16 = 0xff30;
pub const WAVE_PATTERN_RAM_END: u16 = 0xff3f;
