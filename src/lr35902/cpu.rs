use crate::error::AyyError;
use crate::lr35902::handlers::Handlers;
use crate::lr35902::sm83::{Opcode, Register, Sm83};
use crate::memory::mmu::Mmu;
use crate::memory::registers::{InterruptEnable, InterruptFlags};
use crate::memory::{INTERRUPT_ENABLE_REGISTER, INTERRUPT_FLAGS_REGISTER};
use bitflags::bitflags;
use log::{debug, trace};

#[derive(Clone)]
pub struct Cpu {
    sm83: Sm83,
    registers: Registers,
    cycles: usize,
    ime: Ime,
    pub halted: bool,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            sm83: Sm83::new(),
            registers: Registers::default(),
            cycles: 0,
            ime: Ime {
                enabled: false,
                enable_pending: false,
            },
            halted: false,
        }
    }

    pub fn tick(&mut self, mmu: &mut Mmu) -> Result<(), AyyError> {
        self.handle_interrupts(mmu)?;

        if self.halted {
            self.cycles += 4;
            return Ok(());
        }

        let instruction = self.sm83.decode(mmu, self.registers.pc)?;

        trace!(
            "[{:04x}] {:<20} [{}  (SP): ${:02x}  IME: {}  ROM Bank: {}]",
            self.registers.pc,
            format!("{}", instruction),
            self,
            mmu.read(self.registers.sp),
            self.ime.enabled,
            mmu.current_rom_bank()
        );

        self.registers.pc = self.registers.pc.wrapping_add(instruction.length as u16);

        let cycles = match instruction.opcode {
            Opcode::Ld | Opcode::Ldh => Handlers::load(self, mmu, &instruction),
            Opcode::Push => Handlers::push(self, mmu, &instruction),
            Opcode::Pop => Handlers::pop(self, mmu, &instruction),
            Opcode::Ei | Opcode::Di => Handlers::handle_interrupt(self, mmu, &instruction),
            Opcode::Nop => Handlers::nop(self, mmu, &instruction),
            Opcode::Cp => Handlers::compare(self, mmu, &instruction),
            Opcode::Add => Handlers::add(self, mmu, &instruction),
            Opcode::Sub => Handlers::sub(self, mmu, &instruction),
            Opcode::Adc => Handlers::add_with_carry(self, mmu, &instruction),
            Opcode::Sbc => Handlers::sub_with_carry(self, mmu, &instruction),
            Opcode::Inc => Handlers::increment(self, mmu, &instruction),
            Opcode::Dec => Handlers::decrement(self, mmu, &instruction),
            Opcode::Xor => Handlers::xor(self, mmu, &instruction),
            Opcode::And => Handlers::and(self, mmu, &instruction),
            Opcode::Or => Handlers::or(self, mmu, &instruction),
            Opcode::Daa => Handlers::decimal_adjust_accumulator(self, mmu, &instruction),
            Opcode::Halt => Handlers::halt(self, mmu, &instruction),
            Opcode::Jp | Opcode::Jr | Opcode::Call => Handlers::jump(self, mmu, &instruction),
            Opcode::Rst => Handlers::restart(self, mmu, &instruction),
            Opcode::Ret | Opcode::Reti => Handlers::ret(self, mmu, &instruction),
            Opcode::Cpl | Opcode::Scf | Opcode::Ccf => Handlers::complement(self, mmu, &instruction),
            Opcode::Bit => Handlers::test_bit(self, mmu, &instruction),
            Opcode::Rl | Opcode::Rla | Opcode::Rlc | Opcode::Rlca => Handlers::rotate_left(self, mmu, &instruction),
            Opcode::Rr | Opcode::Rra | Opcode::Rrc | Opcode::Rrca => Handlers::rotate_right(self, mmu, &instruction),
            Opcode::Sla => Handlers::shift_left(self, mmu, &instruction),
            Opcode::Sra | Opcode::Srl => Handlers::shift_right(self, mmu, &instruction),
            Opcode::Swap => Handlers::swap(self, mmu, &instruction),
            Opcode::Res => Handlers::reset_bit(self, mmu, &instruction),
            Opcode::Set => Handlers::set_bit(self, mmu, &instruction),
            _ => {
                return Err(AyyError::UnimplementedInstruction {
                    instruction: format!("{}", instruction),
                    cpu: format!("{}", self),
                })
            }
        }?;

        self.cycles += cycles;

        Ok(())
    }

    pub fn read_register(&self, register: &Register) -> u8 {
        match register {
            Register::A => self.registers.a,
            Register::F => self.registers.f.bits(),
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
            Register::AF => (self.registers.a as u16) << 8 | self.registers.f.bits() as u16,
            Register::BC => (self.registers.b as u16) << 8 | self.registers.c as u16,
            Register::DE => (self.registers.d as u16) << 8 | self.registers.e as u16,
            Register::HL => (self.registers.h as u16) << 8 | self.registers.l as u16,
            Register::SP => self.registers.sp,
            Register::PC => self.registers.pc,
            _ => panic!("Invalid register: {:?}", register),
        }
    }

    pub fn write_register(&mut self, register: &Register, data: u8) {
        match register {
            Register::A => self.registers.a = data,
            Register::F => self.registers.f = Flags::from_bits_truncate(data),
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
        let lo = (value & 0xff) as u8;
        let hi = (value >> 8) as u8;

        match register {
            Register::AF => {
                self.registers.a = hi;
                self.registers.f = Flags::from_bits_truncate(lo);
            }
            Register::BC => {
                self.registers.b = hi;
                self.registers.c = lo;
            }
            Register::DE => {
                self.registers.d = hi;
                self.registers.e = lo;
            }
            Register::HL => {
                self.registers.h = hi;
                self.registers.l = lo;
            }
            Register::SP => self.registers.sp = value,
            Register::PC => self.registers.pc = value,
            _ => panic!("Invalid register: {:?}", register),
        }
    }

    pub fn update_flag(&mut self, flag: Flags, value: bool) {
        if value {
            self.set_flag(flag);
        } else {
            self.clear_flag(flag);
        }
    }

    pub fn read_flag(&self, flag: Flags) -> bool {
        self.registers.f.contains(flag)
    }

    #[inline]
    pub fn set_flag(&mut self, flag: Flags) {
        self.registers.f |= flag;
    }

    #[inline]
    pub fn clear_flag(&mut self, flag: Flags) {
        self.registers.f &= !flag;
    }

    pub fn push_stack(&mut self, mmu: &mut Mmu, value: u16) {
        self.registers.sp -= 2;
        mmu.write16(self.registers.sp, value);
    }

    pub fn pop_stack(&mut self, mmu: &Mmu) -> u16 {
        let value = mmu.read16(self.registers.sp);
        self.registers.sp += 2;
        value
    }

    pub fn enable_interrupts(&mut self, delayed: bool) {
        if delayed {
            self.ime.enable_pending = true;
        } else {
            self.ime.enabled = true;
        }
    }

    pub fn disable_interrupts(&mut self) {
        self.ime.enabled = false;
    }

    pub fn interrupt_master_raised(&self) -> bool {
        self.ime.enabled
    }

    pub fn elapsed_cycles(&self) -> usize {
        self.cycles
    }

    pub fn reset_cycles(&mut self) {
        self.cycles = 0;
    }

    fn handle_interrupts(&mut self, mmu: &mut Mmu) -> Result<(), AyyError> {
        // "EI instruction enables IME the following cycle to its execution."
        //   - TCAGBD.pdf, chapter 3.3
        if self.ime.enable_pending {
            self.ime.enabled = true;
            self.ime.enable_pending = false;

            debug!("IME pending, enabled");
        }

        let interrupt_enable = mmu.read_as::<InterruptEnable>(INTERRUPT_ENABLE_REGISTER);
        let interrupt_flags = mmu.read_as::<InterruptFlags>(INTERRUPT_FLAGS_REGISTER);

        if self.ime.enabled && interrupt_enable.bits() & interrupt_flags.bits() != 0 {
            // handle interrupt vector
            self.push_stack(mmu, self.registers.pc);
            let vector = interrupt_flags.to_vector()?;
            debug!("Handling interrupt vector: ${:04x}", vector);
            self.registers.pc = vector;

            // clear interrupt flag
            match vector {
                0x0040 => mmu.write(INTERRUPT_FLAGS_REGISTER, interrupt_flags.bits() & !InterruptFlags::VBLANK.bits()),
                0x0048 => mmu.write(INTERRUPT_FLAGS_REGISTER, interrupt_flags.bits() & !InterruptFlags::LCD_STAT.bits()),
                0x0050 => mmu.write(INTERRUPT_FLAGS_REGISTER, interrupt_flags.bits() & !InterruptFlags::TIMER.bits()),
                0x0058 => mmu.write(INTERRUPT_FLAGS_REGISTER, interrupt_flags.bits() & !InterruptFlags::SERIAL.bits()),
                0x0060 => mmu.write(INTERRUPT_FLAGS_REGISTER, interrupt_flags.bits() & !InterruptFlags::JOYPAD.bits()),
                _ => unreachable!(),
            }
            self.ime.enabled = false;

            // unhalt the CPU
            self.halted = false;

            // "The entire process lasts 5 M-cycles."
            //   - Pandocs
            self.cycles += 20;
        }

        Ok(())
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

#[derive(Clone)]
pub struct Ime {
    pub enabled: bool,
    pub enable_pending: bool,
}

bitflags! {
    #[derive(Clone)]
    pub struct Flags: u8 {
        const ZERO       = 0b1000_0000;
        const SUBTRACT   = 0b0100_0000;
        const HALF_CARRY = 0b0010_0000;
        const CARRY      = 0b0001_0000;
    }
}

#[derive(Clone)]
struct Registers {
    a: u8,
    f: Flags,
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
            f: Flags::empty(),
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
