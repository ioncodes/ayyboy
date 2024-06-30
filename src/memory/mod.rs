pub mod mapper;
pub mod mmu;
pub mod registers;

pub const INTERRUPT_ENABLE_REGISTER: u16 = 0xffff;
pub const INTERRUPT_FLAGS_REGISTER: u16 = 0xff0f;
pub const BOOTROM_MAPPER_REGISTER: u16 = 0xff50;
pub const OAM_DMA_REGISTER: u16 = 0xff46;
pub const JOYPAD_REGISTER: u16 = 0xff00;
