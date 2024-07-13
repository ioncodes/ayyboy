pub mod addressable;
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
pub const VRAM_BANK_SELECT_REGISTER: u16 = 0xff4f;
pub const WRAM_BANK_SELECT_REGISTER: u16 = 0xff70;

pub const ROM_START: u16 = 0x0000;
pub const ROM_END: u16 = 0x7fff;
pub const EXTERNAL_RAM_START: u16 = 0xa000;
pub const EXTERNAL_RAM_END: u16 = 0xbfff;
pub const VRAM_START: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9fff;
pub const WRAM_BANK1_START: u16 = 0xd000;
pub const WRAM_BANK1_END: u16 = 0xdfff;
