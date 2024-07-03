pub mod mapper;
pub mod mmu;
pub mod registers;

pub const INTERRUPT_ENABLE_REGISTER: u16 = 0xffff;
pub const INTERRUPT_FLAGS_REGISTER: u16 = 0xff0f;
pub const BOOTROM_MAPPER_REGISTER: u16 = 0xff50;
pub const OAM_DMA_REGISTER: u16 = 0xff46;
pub const JOYPAD_REGISTER: u16 = 0xff00;
pub const DIV_REGISTER: u16 = 0xff04;
pub const TIMA_REGISTER: u16 = 0xff05;
pub const TMA_REGISTER: u16 = 0xff06;
pub const TAC_REGISTER: u16 = 0xff07;

pub const ROM_START: u16 = 0x0000;
pub const ROM_END: u16 = 0x7fff;
pub const EXTERNAL_RAM_START: u16 = 0xa000;
pub const EXTERNAL_RAM_END: u16 = 0xbfff;
