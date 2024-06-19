use crate::lr35902::handlers::Handlers;
use crate::lr35902::sm83::{Opcode, Register, Sm83};
use crate::memory::mmu::Mmu;

pub struct Cpu {
    sm83: Sm83,
    registers: Registers,
    cycles: usize,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            sm83: Sm83::new(),
            registers: Registers::default(),
            cycles: 0,
        }
    }

    pub fn tick(&mut self, mmu: &mut Mmu) {
        let instruction = if let Ok(instruction) = self.sm83.decode(mmu, self.registers.pc) {
            println!("[{:#04x}] {}", self.registers.pc, instruction);
            instruction
        } else {
            panic!("Failed to decode instruction at address: {:#04x}\n{}", self.registers.pc, self);
        };

        let cycles = match instruction.opcode {
            Opcode::Ld => Handlers::load(self, mmu, &instruction),
            _ => panic!("Unimplemented instruction: {}\n{}", instruction, self),
        };

        if let Ok(cycles) = cycles {
            self.registers.pc += instruction.length as u16;
            self.cycles += cycles;
        } else {
            panic!("Failed to execute instruction: {}\n{}", instruction, self);
        }
    }

    pub fn read_register(&self, register: &Register) -> u8 {
        match register {
            Register::A => self.registers.a,
            Register::B => self.registers.b,
            Register::C => self.registers.c,
            Register::D => self.registers.d,
            Register::E => self.registers.e,
            Register::H => self.registers.h,
            Register::L => self.registers.l,
            _ => panic!("Invalid register: {:?}", register),
        }
    }

    pub fn read_register16(&self, register: &Register) -> u16 {
        match register {
            Register::AF => u16::from_le_bytes([self.registers.a, self.registers.f]),
            Register::BC => u16::from_le_bytes([self.registers.b, self.registers.c]),
            Register::DE => u16::from_le_bytes([self.registers.d, self.registers.e]),
            Register::HL => u16::from_le_bytes([self.registers.h, self.registers.l]),
            Register::SP => self.registers.sp,
            Register::PC => self.registers.pc,
            _ => panic!("Invalid register: {:?}", register),
        }
    }

    pub fn write_register(&mut self, register: &Register, data: u8) {
        match register {
            Register::A => self.registers.a = data,
            Register::B => self.registers.b = data,
            Register::C => self.registers.c = data,
            Register::D => self.registers.d = data,
            Register::E => self.registers.e = data,
            Register::H => self.registers.h = data,
            Register::L => self.registers.l = data,
            _ => panic!("Invalid register: {:?}", register),
        }
    }

    pub fn write_register16(&mut self, register: &Register, value: u16) {
        let [high, low] = value.to_le_bytes();
        match register {
            Register::AF => {
                self.registers.a = high;
                self.registers.f = low;
            }
            Register::BC => {
                self.registers.b = high;
                self.registers.c = low;
            }
            Register::DE => {
                self.registers.d = high;
                self.registers.e = low;
            }
            Register::HL => {
                self.registers.h = high;
                self.registers.l = low;
            }
            Register::SP => self.registers.sp = value,
            Register::PC => self.registers.pc = value,
            _ => panic!("Invalid register: {:?}", register),
        }
    }
}

impl std::fmt::Display for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "A: ${:02x}  F: ${:02x}  B: ${:02x}  C: ${:02x}  D: ${:02x}  E: ${:02x}  H: ${:02x}  L: ${:02x}  SP: ${:04x}  PC: ${:04x}",
            self.registers.a,
            self.registers.f,
            self.registers.b,
            self.registers.c,
            self.registers.d,
            self.registers.e,
            self.registers.h,
            self.registers.l,
            self.registers.sp,
            self.registers.pc
        )
    }
}

struct Registers {
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
}

impl Default for Registers {
    fn default() -> Registers {
        Registers {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
        }
    }
}
