use crate::error::AyyError;
use crate::lr35902::handlers::Handlers;
use crate::lr35902::irq::{Ime, Vector};
use crate::lr35902::registers::{Flags, Registers};
use crate::lr35902::sm83::{Opcode, Register, Sm83};
use crate::lr35902::timer::Timer;
use crate::memory::mmu::Mmu;
use crate::memory::registers::{InterruptEnable, InterruptFlags};
use crate::memory::{DIV_REGISTER, INTERRUPT_ENABLE_REGISTER, INTERRUPT_FLAGS_REGISTER};
use log::{debug, trace};

#[derive(Clone)]
pub struct Cpu {
    sm83: Sm83,
    registers: Registers,
    cycles: usize,
    ime: Ime,
    div_cycles: usize,
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
            div_cycles: 0,
            halted: false,
        }
    }

    pub fn tick(&mut self, mmu: &mut Mmu, timer: &mut Timer) -> Result<usize, AyyError> {
        self.handle_interrupts(mmu)?;

        if self.halted {
            self.cycles += 4;
            return Ok(4);
        }

        let instruction = self.sm83.decode(mmu, self.registers.pc)?;
        let instruction_bytes = (0..instruction.length)
            .map(|i| mmu.read_unchecked(self.registers.pc + i as u16))
            .collect::<Vec<u8>>();

        trace!(
            "[{:04x}] {:<12} {:<20} [{}  (SP): ${:02x}  IME: {}  ROM Bank: {}  RAM Bank: {}]",
            self.registers.pc,
            format!("{:02x?}", instruction_bytes),
            format!("{}", instruction),
            self,
            mmu.read(self.registers.sp)?,
            self.ime.enabled,
            mmu.current_rom_bank(),
            mmu.current_ram_bank()
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
            Opcode::Stop => {
                timer.reset_divider(mmu);
                Ok(4)
            }
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
            _ => Err(AyyError::UnimplementedInstruction {
                instruction: format!("{}", instruction),
                cpu: format!("{}", self),
            }),
        }?;

        self.cycles += cycles;
        self.div_cycles += cycles;

        self.tick_div(mmu);

        Ok(cycles)
    }

    #[inline]
    pub fn tick_div(&mut self, mmu: &mut Mmu) {
        if self.div_cycles >= 256 {
            let div = mmu.read_unchecked(DIV_REGISTER).wrapping_add(1);
            mmu.write_unchecked(DIV_REGISTER, div);
            self.div_cycles -= 256;
        }
    }

    #[inline]
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

    #[inline]
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

    #[inline]
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

    #[inline]
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

    #[inline]
    pub fn update_flag(&mut self, flag: Flags, value: bool) {
        if value {
            self.set_flag(flag);
        } else {
            self.clear_flag(flag);
        }
    }

    #[inline]
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

    #[inline]
    pub fn push_stack(&mut self, mmu: &mut Mmu, value: u16) -> Result<(), AyyError> {
        self.registers.sp -= 2;
        mmu.write16(self.registers.sp, value)?;
        Ok(())
    }

    #[inline]
    pub fn pop_stack(&mut self, mmu: &Mmu) -> Result<u16, AyyError> {
        let value = mmu.read16(self.registers.sp)?;
        self.registers.sp += 2;
        Ok(value)
    }

    #[inline]
    pub fn enable_interrupts(&mut self, delayed: bool) {
        if delayed {
            self.ime.enable_pending = true;
        } else {
            self.ime.enabled = true;
        }
    }

    #[inline]
    pub fn disable_interrupts(&mut self) {
        self.ime.enabled = false;
    }

    #[inline]
    #[cfg(test)]
    pub fn interrupt_master_raised(&self) -> bool {
        self.ime.enabled
    }

    #[inline]
    pub fn elapsed_cycles(&self) -> usize {
        self.cycles
    }

    #[inline]
    pub fn reset_cycles(&mut self, to: usize) {
        self.cycles = to;
    }

    fn handle_interrupts(&mut self, mmu: &mut Mmu) -> Result<(), AyyError> {
        // "EI instruction enables IME the following cycle to its execution."
        //   - TCAGBD.pdf, chapter 3.3
        if self.ime.enable_pending {
            self.ime.enabled = true;
            self.ime.enable_pending = false;

            debug!("IME pending, enabled");
        }

        let interrupt_enable = mmu.read_as::<InterruptEnable>(INTERRUPT_ENABLE_REGISTER)?;
        let interrupt_flags = mmu.read_as::<InterruptFlags>(INTERRUPT_FLAGS_REGISTER)?;

        if interrupt_enable.bits() & interrupt_flags.bits() != 0 {
            if self.ime.enabled {
                // handle interrupt vector
                let vector = Vector::from_flags(&interrupt_flags);
                debug!("Handling interrupt: {} => ${:04x}", vector, vector.to_address());

                // save $pc, jump to interrupt vector
                self.push_stack(mmu, self.registers.pc)?;
                self.registers.pc = vector.to_address();

                // clear interrupt flag
                match vector {
                    Vector::VBlank => mmu.write(INTERRUPT_FLAGS_REGISTER, interrupt_flags.bits() & !InterruptFlags::VBLANK.bits())?,
                    Vector::Stat => mmu.write(INTERRUPT_FLAGS_REGISTER, interrupt_flags.bits() & !InterruptFlags::STAT.bits())?,
                    Vector::Timer => mmu.write(INTERRUPT_FLAGS_REGISTER, interrupt_flags.bits() & !InterruptFlags::TIMER.bits())?,
                    Vector::Serial => mmu.write(INTERRUPT_FLAGS_REGISTER, interrupt_flags.bits() & !InterruptFlags::SERIAL.bits())?,
                    Vector::Joypad => mmu.write(INTERRUPT_FLAGS_REGISTER, interrupt_flags.bits() & !InterruptFlags::JOYPAD.bits())?,
                }
                self.ime.enabled = false;
            }

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
