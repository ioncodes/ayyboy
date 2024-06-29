mod mapper;
pub mod mmu;
pub mod registers;

pub const INTERRUPT_ENABLE_REGISTER: u16 = 0xffff;
pub const INTERRUPT_FLAGS_REGISTER: u16 = 0xff0f;
pub const BOOTROM_MAPPER_REGISTER: u16 = 0xff50;
