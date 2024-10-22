use crate::error::AyyError;
use crate::error::AyyError::{InvalidHandler, UnresolvedTarget};
use crate::lr35902::cpu::Cpu;
use crate::lr35902::registers::Flags;
use crate::lr35902::sm83::{AddressingMode, Condition, Instruction, Opcode, Operand, Register};
use crate::memory::mmu::Mmu;

use super::timer::Timer;

macro_rules! invalid_handler {
    ($instruction:expr) => {
        Err(InvalidHandler {
            instruction: $instruction.clone(),
        })
    };
}

macro_rules! ensure {
    (lhs => $instr:expr) => {
        #[cfg(debug_assertions)]
        if $instr.lhs.is_none() {
            return Err(InvalidHandler {
                instruction: $instr.clone(),
            });
        }
    };
    (lhs_rhs => $instr:expr) => {
        #[cfg(debug_assertions)]
        if $instr.lhs.is_none() || $instr.rhs.is_none() {
            return Err(InvalidHandler {
                instruction: $instr.clone(),
            });
        }
    };
}

pub struct Handlers {}

#[allow(unused_variables)]
impl Handlers {
    #[inline]
    pub fn load(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs_rhs => instruction);

        // In case of LDH we need to make sure to add 0xff00 to dst/src
        let is_ldh_instruction = instruction.opcode == Opcode::Ldh;
        let src = instruction.rhs.as_ref().unwrap();
        let src = Handlers::resolve_operand(cpu, mmu, src, is_ldh_instruction)?;

        match instruction {
            Instruction {
                opcode: Opcode::Ld,
                lhs: Some(Operand::Reg16(reg, _)),
                rhs: Some(Operand::DisplacedReg16(Register::SP, offset, _)),
                ..
            } => {
                let sp = cpu.read_register16(&Register::SP);
                let addr = sp.wrapping_add_signed(*offset as i16);
                cpu.write_register16(reg, addr);

                let imm_signed = *offset as i16;
                let effective_address = (addr as i32).wrapping_add(imm_signed as i32) as u16;
                let sp_low = sp as u8;
                let imm_low = *offset as u8;

                cpu.update_flag(Flags::ZERO, false);
                cpu.update_flag(Flags::SUBTRACT, false);
                cpu.update_flag(Flags::HALF_CARRY, (sp_low & 0x0f) + (imm_low & 0x0f) > 0x0f);
                cpu.update_flag(Flags::CARRY, (sp_low as u16) + (imm_low as u16) > 0xff);
            }
            Instruction {
                opcode: Opcode::Ld,
                lhs: Some(Operand::Reg8(reg, mode)),
                ..
            } => {
                if !mode.contains(AddressingMode::Indirect) {
                    cpu.write_register(reg, src as u8);
                } else {
                    let addr = 0xff00 + cpu.read_register(reg) as u16;
                    mmu.write(addr, src as u8)?;
                }
            }
            Instruction {
                opcode: Opcode::Ld,
                lhs: Some(Operand::Reg16(Register::HL, mode)),
                ..
            } if mode.contains(AddressingMode::Increment) => {
                let addr = cpu.read_register16(&Register::HL);
                mmu.write(addr, src as u8)?;
                cpu.write_register16(&Register::HL, addr.wrapping_add(1));
            }
            Instruction {
                opcode: Opcode::Ld,
                lhs: Some(Operand::Reg16(Register::HL, mode)),
                ..
            } if mode.contains(AddressingMode::Decrement) => {
                let addr = cpu.read_register16(&Register::HL);
                mmu.write(addr, src as u8)?;
                cpu.write_register16(&Register::HL, addr.wrapping_sub(1));
            }
            Instruction {
                opcode: Opcode::Ld,
                lhs: Some(Operand::Reg16(reg, mode)),
                ..
            } if !mode.contains(AddressingMode::Indirect) => {
                cpu.write_register16(reg, src as u16);
            }
            Instruction {
                opcode: Opcode::Ld,
                lhs: Some(Operand::Reg16(reg, mode)),
                ..
            } if mode.contains(AddressingMode::Indirect) => {
                let addr = cpu.read_register16(reg);
                mmu.write(addr, src as u8)?;
            }
            Instruction {
                opcode: Opcode::Ld,
                lhs: Some(Operand::Imm16(addr, _)),
                rhs: Some(Operand::Reg16(reg, _)),
                ..
            } => {
                let value = cpu.read_register16(reg);
                mmu.write16(*addr, value)?;
            }
            Instruction {
                opcode: Opcode::Ld,
                lhs: Some(Operand::Imm16(addr, _)),
                rhs: Some(Operand::Reg8(reg, _)),
                ..
            } => {
                let value = cpu.read_register(reg);
                mmu.write(*addr, value)?;
            }
            Instruction {
                opcode: Opcode::Ldh,
                lhs: Some(Operand::Imm8(addr, _)),
                ..
            } => {
                mmu.write(0xff00 + *addr as u16, src as u8)?;
            }
            Instruction {
                opcode: Opcode::Ldh,
                lhs: Some(Operand::Reg8(reg, _)),
                rhs: Some(Operand::Imm8(addr, _)),
                ..
            } => {
                let value = mmu.read(0xff00 + *addr as u16)?;
                cpu.write_register(reg, value);
            }
            _ => return invalid_handler!(instruction),
        }

        Ok(instruction.cycles.0)
    }

    #[inline]
    pub fn nop(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        Ok(instruction.cycles.0)
    }

    #[inline]
    pub fn xor(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs_rhs => instruction);

        let x = Handlers::resolve_operand(cpu, mmu, instruction.lhs.as_ref().unwrap(), false)? as u8;
        let y = Handlers::resolve_operand(cpu, mmu, instruction.rhs.as_ref().unwrap(), false)? as u8;

        let result = x ^ y;
        cpu.write_register(&Register::A, result);

        cpu.update_flag(Flags::ZERO, result == 0);
        cpu.update_flag(Flags::SUBTRACT, false);
        cpu.update_flag(Flags::HALF_CARRY, false);
        cpu.update_flag(Flags::CARRY, false);

        Ok(instruction.cycles.0)
    }

    #[inline]
    pub fn complement(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        match instruction {
            Instruction {
                opcode: Opcode::Cpl, ..
            } => {
                let value = cpu.read_register(&Register::A);
                let result = !value;
                cpu.write_register(&Register::A, result);

                cpu.update_flag(Flags::SUBTRACT, true);
                cpu.update_flag(Flags::HALF_CARRY, true);

                Ok(instruction.cycles.0)
            }
            Instruction {
                opcode: Opcode::Ccf, ..
            } => {
                let carry = cpu.read_flag(Flags::CARRY);
                cpu.update_flag(Flags::SUBTRACT, false);
                cpu.update_flag(Flags::HALF_CARRY, false);
                cpu.update_flag(Flags::CARRY, !carry);

                Ok(instruction.cycles.0)
            }
            Instruction {
                opcode: Opcode::Scf, ..
            } => {
                cpu.update_flag(Flags::SUBTRACT, false);
                cpu.update_flag(Flags::HALF_CARRY, false);
                cpu.update_flag(Flags::CARRY, true);

                Ok(instruction.cycles.0)
            }
            _ => invalid_handler!(instruction),
        }
    }

    #[inline]
    pub fn decimal_adjust_accumulator(
        cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction,
    ) -> Result<usize, AyyError> {
        let mut a = cpu.read_register(&Register::A);
        let mut adjust = 0;
        let mut carry = cpu.read_flag(Flags::CARRY) as u8;

        if cpu.read_flag(Flags::HALF_CARRY) || (!cpu.read_flag(Flags::SUBTRACT) && (a & 0x0f) > 9) {
            adjust |= 0x06;
        }

        if cpu.read_flag(Flags::CARRY) || (!cpu.read_flag(Flags::SUBTRACT) && a > 0x99) {
            adjust |= 0x60;
            carry = 1;
        }

        if cpu.read_flag(Flags::SUBTRACT) {
            a = a.wrapping_sub(adjust);
        } else {
            a = a.wrapping_add(adjust);
        }

        cpu.write_register(&Register::A, a);

        cpu.update_flag(Flags::ZERO, a == 0);
        cpu.update_flag(Flags::HALF_CARRY, false);
        cpu.update_flag(Flags::CARRY, carry == 1);

        Ok(instruction.cycles.0)
    }

    #[inline]
    pub fn add(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs_rhs => instruction);

        match instruction.lhs.as_ref().unwrap() {
            Operand::Reg16(reg, _) => {
                let x = Handlers::resolve_operand(cpu, mmu, instruction.lhs.as_ref().unwrap(), false)? as u16;

                if reg == &Register::SP {
                    let y = Handlers::resolve_operand(cpu, mmu, instruction.rhs.as_ref().unwrap(), false)? as i16;
                    let result = x.wrapping_add_signed(y);
                    cpu.write_register16(reg, result);

                    cpu.update_flag(Flags::ZERO, false);
                    cpu.update_flag(Flags::SUBTRACT, false);
                    cpu.update_flag(Flags::HALF_CARRY, (x & 0x0f).wrapping_add_signed(y & 0x0f) > 0x0f);
                    cpu.update_flag(Flags::CARRY, (x & 0xff).wrapping_add_signed(y & 0xff) > 0xff);
                } else {
                    let y = Handlers::resolve_operand(cpu, mmu, instruction.rhs.as_ref().unwrap(), false)? as u16;
                    let result = x.wrapping_add(y);
                    cpu.write_register16(reg, result);

                    cpu.update_flag(Flags::SUBTRACT, false);
                    cpu.update_flag(Flags::HALF_CARRY, (x & 0x0fff) + (y & 0x0fff) > 0x0fff);
                    cpu.update_flag(Flags::CARRY, result < x);
                }
            }
            _ => {
                let x = Handlers::resolve_operand(cpu, mmu, instruction.lhs.as_ref().unwrap(), false)? as u8;
                let y = Handlers::resolve_operand(cpu, mmu, instruction.rhs.as_ref().unwrap(), false)? as u8;
                let result = x.wrapping_add(y);
                cpu.write_register(&Register::A, result);

                cpu.update_flag(Flags::ZERO, result == 0);
                cpu.update_flag(Flags::SUBTRACT, false);
                cpu.update_flag(Flags::HALF_CARRY, (x & 0x0f) + (y & 0x0f) > 0x0f);
                cpu.update_flag(Flags::CARRY, result < x);
            }
        };

        Ok(instruction.cycles.0)
    }

    #[inline]
    pub fn sub(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs_rhs => instruction);

        match instruction.lhs.as_ref().unwrap() {
            Operand::Reg16(Register::HL, _) => {
                let x = Handlers::resolve_operand(cpu, mmu, instruction.lhs.as_ref().unwrap(), false)? as u16;
                let y = Handlers::resolve_operand(cpu, mmu, instruction.rhs.as_ref().unwrap(), false)? as u16;

                let result = x.wrapping_sub(y);
                cpu.write_register16(&Register::HL, result);

                cpu.update_flag(Flags::SUBTRACT, true);
                cpu.update_flag(Flags::HALF_CARRY, (x & 0x0fff) < (y & 0x0fff));
                cpu.update_flag(Flags::CARRY, result > x);
            }
            _ => {
                let x = Handlers::resolve_operand(cpu, mmu, instruction.lhs.as_ref().unwrap(), false)? as u8;
                let y = Handlers::resolve_operand(cpu, mmu, instruction.rhs.as_ref().unwrap(), false)? as u8;
                let result = x.wrapping_sub(y);
                cpu.write_register(&Register::A, result);

                cpu.update_flag(Flags::ZERO, result == 0);
                cpu.update_flag(Flags::SUBTRACT, true);
                cpu.update_flag(Flags::HALF_CARRY, (x & 0x0f) < (y & 0x0f));
                cpu.update_flag(Flags::CARRY, result > x);
            }
        };

        Ok(instruction.cycles.0)
    }

    #[inline]
    pub fn and(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs_rhs => instruction);

        let x = Handlers::resolve_operand(cpu, mmu, instruction.lhs.as_ref().unwrap(), false)? as u8;
        let y = Handlers::resolve_operand(cpu, mmu, instruction.rhs.as_ref().unwrap(), false)? as u8;

        let result = x & y;
        cpu.write_register(&Register::A, result);

        cpu.update_flag(Flags::ZERO, result == 0);
        cpu.update_flag(Flags::SUBTRACT, false);
        cpu.update_flag(Flags::HALF_CARRY, true);
        cpu.update_flag(Flags::CARRY, false);

        Ok(instruction.cycles.0)
    }

    #[inline]
    pub fn or(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs_rhs => instruction);

        let x = Handlers::resolve_operand(cpu, mmu, instruction.lhs.as_ref().unwrap(), false)? as u8;
        let y = Handlers::resolve_operand(cpu, mmu, instruction.rhs.as_ref().unwrap(), false)? as u8;

        let result = x | y;
        cpu.write_register(&Register::A, result);

        cpu.update_flag(Flags::ZERO, result == 0);
        cpu.update_flag(Flags::SUBTRACT, false);
        cpu.update_flag(Flags::HALF_CARRY, false);
        cpu.update_flag(Flags::CARRY, false);

        Ok(instruction.cycles.0)
    }

    #[inline]
    pub fn rotate_left(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        match instruction {
            Instruction {
                opcode: Opcode::Rl,
                lhs: Some(Operand::Reg8(reg, _)),
                ..
            } => {
                let value = cpu.read_register(reg);
                let carry = cpu.read_flag(Flags::CARRY) as u8;
                let result = (value << 1) | carry;
                cpu.write_register(reg, result);

                cpu.update_flag(Flags::ZERO, result == 0);
                cpu.update_flag(Flags::SUBTRACT, false);
                cpu.update_flag(Flags::HALF_CARRY, false);
                cpu.update_flag(Flags::CARRY, value & 0x80 != 0);

                Ok(instruction.cycles.0)
            }
            Instruction {
                opcode: Opcode::Rl,
                lhs: Some(Operand::Reg16(Register::HL, _)),
                ..
            } => {
                let addr = cpu.read_register16(&Register::HL);
                let value = mmu.read(addr)?;
                let carry = cpu.read_flag(Flags::CARRY) as u8;
                let result = (value << 1) | carry;
                mmu.write(addr, result)?;

                cpu.update_flag(Flags::ZERO, result == 0);
                cpu.update_flag(Flags::SUBTRACT, false);
                cpu.update_flag(Flags::HALF_CARRY, false);
                cpu.update_flag(Flags::CARRY, value & 0x80 != 0);

                Ok(instruction.cycles.0)
            }
            Instruction {
                opcode: Opcode::Rla, ..
            } => {
                let value = cpu.read_register(&Register::A);
                let carry = cpu.read_flag(Flags::CARRY) as u8;
                let result = (value << 1) | carry;
                cpu.write_register(&Register::A, result);

                cpu.update_flag(Flags::ZERO, false);
                cpu.update_flag(Flags::SUBTRACT, false);
                cpu.update_flag(Flags::HALF_CARRY, false);
                cpu.update_flag(Flags::CARRY, value & 0x80 != 0);

                Ok(instruction.cycles.0)
            }
            Instruction {
                opcode: Opcode::Rlc,
                lhs: Some(Operand::Reg8(reg, _)),
                ..
            } => {
                let value = cpu.read_register(reg);
                let result = (value << 1) | (value >> 7);
                cpu.write_register(reg, result);

                cpu.update_flag(Flags::ZERO, result == 0);
                cpu.update_flag(Flags::SUBTRACT, false);
                cpu.update_flag(Flags::HALF_CARRY, false);
                cpu.update_flag(Flags::CARRY, value & 0x80 != 0);

                Ok(instruction.cycles.0)
            }
            Instruction {
                opcode: Opcode::Rlc,
                lhs: Some(Operand::Reg16(Register::HL, _)),
                ..
            } => {
                let addr = cpu.read_register16(&Register::HL);
                let value = mmu.read(addr)?;
                let result = (value << 1) | (value >> 7);
                mmu.write(addr, result)?;

                cpu.update_flag(Flags::ZERO, result == 0);
                cpu.update_flag(Flags::SUBTRACT, false);
                cpu.update_flag(Flags::HALF_CARRY, false);
                cpu.update_flag(Flags::CARRY, value & 0x80 != 0);

                Ok(instruction.cycles.0)
            }
            Instruction {
                opcode: Opcode::Rlca, ..
            } => {
                let value = cpu.read_register(&Register::A);
                let result = (value << 1) | (value >> 7);
                cpu.write_register(&Register::A, result);

                cpu.update_flag(Flags::ZERO, false);
                cpu.update_flag(Flags::SUBTRACT, false);
                cpu.update_flag(Flags::HALF_CARRY, false);
                cpu.update_flag(Flags::CARRY, value & 0x80 != 0);

                Ok(instruction.cycles.0)
            }
            _ => invalid_handler!(instruction),
        }
    }

    #[inline]
    pub fn rotate_right(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        match instruction {
            Instruction {
                opcode: Opcode::Rr,
                lhs: Some(Operand::Reg8(reg, _)),
                ..
            } => {
                let value = cpu.read_register(reg);
                let carry = cpu.read_flag(Flags::CARRY) as u8;
                let result = (value >> 1) | (carry << 7);
                cpu.write_register(reg, result);

                cpu.update_flag(Flags::ZERO, result == 0);
                cpu.update_flag(Flags::SUBTRACT, false);
                cpu.update_flag(Flags::HALF_CARRY, false);
                cpu.update_flag(Flags::CARRY, value & 0x01 != 0);

                Ok(instruction.cycles.0)
            }
            Instruction {
                opcode: Opcode::Rr,
                lhs: Some(Operand::Reg16(Register::HL, _)),
                ..
            } => {
                let addr = cpu.read_register16(&Register::HL);
                let value = mmu.read(addr)?;
                let carry = cpu.read_flag(Flags::CARRY) as u8;
                let result = (value >> 1) | (carry << 7);
                mmu.write(addr, result)?;

                cpu.update_flag(Flags::ZERO, result == 0);
                cpu.update_flag(Flags::SUBTRACT, false);
                cpu.update_flag(Flags::HALF_CARRY, false);
                cpu.update_flag(Flags::CARRY, value & 0x01 != 0);

                Ok(instruction.cycles.0)
            }
            Instruction {
                opcode: Opcode::Rra, ..
            } => {
                let value = cpu.read_register(&Register::A);
                let carry = cpu.read_flag(Flags::CARRY) as u8;
                let result = (value >> 1) | (carry << 7);
                cpu.write_register(&Register::A, result);

                cpu.update_flag(Flags::ZERO, false);
                cpu.update_flag(Flags::SUBTRACT, false);
                cpu.update_flag(Flags::HALF_CARRY, false);
                cpu.update_flag(Flags::CARRY, value & 0x01 != 0);

                Ok(instruction.cycles.0)
            }
            Instruction {
                opcode: Opcode::Rrc,
                lhs: Some(Operand::Reg8(reg, _)),
                ..
            } => {
                let value = cpu.read_register(reg);
                let result = (value >> 1) | (value << 7);
                cpu.write_register(reg, result);

                cpu.update_flag(Flags::ZERO, result == 0);
                cpu.update_flag(Flags::SUBTRACT, false);
                cpu.update_flag(Flags::HALF_CARRY, false);
                cpu.update_flag(Flags::CARRY, value & 0x01 != 0);

                Ok(instruction.cycles.0)
            }
            Instruction {
                opcode: Opcode::Rrc,
                lhs: Some(Operand::Reg16(Register::HL, _)),
                ..
            } => {
                let addr = cpu.read_register16(&Register::HL);
                let value = mmu.read(addr)?;
                let result = (value >> 1) | (value << 7);
                mmu.write(addr, result)?;

                cpu.update_flag(Flags::ZERO, result == 0);
                cpu.update_flag(Flags::SUBTRACT, false);
                cpu.update_flag(Flags::HALF_CARRY, false);
                cpu.update_flag(Flags::CARRY, value & 0x01 != 0);

                Ok(instruction.cycles.0)
            }
            Instruction {
                opcode: Opcode::Rrca, ..
            } => {
                let value = cpu.read_register(&Register::A);
                let result = (value >> 1) | (value << 7);
                cpu.write_register(&Register::A, result);

                cpu.update_flag(Flags::ZERO, false);
                cpu.update_flag(Flags::SUBTRACT, false);
                cpu.update_flag(Flags::HALF_CARRY, false);
                cpu.update_flag(Flags::CARRY, value & 0x01 != 0);

                Ok(instruction.cycles.0)
            }
            _ => invalid_handler!(instruction),
        }
    }

    #[inline]
    pub fn shift_left(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs => instruction);

        match instruction {
            Instruction {
                opcode: Opcode::Sla,
                lhs: Some(Operand::Reg8(reg, _)),
                ..
            } => {
                let value = cpu.read_register(reg);
                let result = value << 1;
                cpu.write_register(reg, result);

                cpu.update_flag(Flags::ZERO, result == 0);
                cpu.update_flag(Flags::SUBTRACT, false);
                cpu.update_flag(Flags::HALF_CARRY, false);
                cpu.update_flag(Flags::CARRY, value & 0x80 != 0);

                Ok(instruction.cycles.0)
            }
            Instruction {
                opcode: Opcode::Sla,
                lhs: Some(Operand::Reg16(Register::HL, AddressingMode::Indirect)),
                ..
            } => {
                let addr = cpu.read_register16(&Register::HL);
                let value = mmu.read(addr)?;
                let result = value << 1;
                mmu.write(addr, result)?;

                cpu.update_flag(Flags::ZERO, result == 0);
                cpu.update_flag(Flags::SUBTRACT, false);
                cpu.update_flag(Flags::HALF_CARRY, false);
                cpu.update_flag(Flags::CARRY, value & 0x80 != 0);

                Ok(instruction.cycles.0)
            }
            _ => invalid_handler!(instruction),
        }
    }

    #[inline]
    pub fn shift_right(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs => instruction);

        match instruction {
            Instruction {
                opcode: Opcode::Sra,
                lhs: Some(Operand::Reg8(reg, _)),
                ..
            } => {
                let value = cpu.read_register(reg);
                let result = (value >> 1) | (value & 0x80);
                cpu.write_register(reg, result);

                cpu.update_flag(Flags::ZERO, result == 0);
                cpu.update_flag(Flags::SUBTRACT, false);
                cpu.update_flag(Flags::HALF_CARRY, false);
                cpu.update_flag(Flags::CARRY, value & 0x01 != 0);

                Ok(instruction.cycles.0)
            }
            Instruction {
                opcode: Opcode::Sra,
                lhs: Some(Operand::Reg16(Register::HL, AddressingMode::Indirect)),
                ..
            } => {
                let addr = cpu.read_register16(&Register::HL);
                let value = mmu.read(addr)?;
                let result = (value >> 1) | (value & 0x80);
                mmu.write(addr, result)?;

                cpu.update_flag(Flags::ZERO, result == 0);
                cpu.update_flag(Flags::SUBTRACT, false);
                cpu.update_flag(Flags::HALF_CARRY, false);
                cpu.update_flag(Flags::CARRY, value & 0x01 != 0);

                Ok(instruction.cycles.0)
            }
            Instruction {
                opcode: Opcode::Srl,
                lhs: Some(Operand::Reg8(reg, _)),
                ..
            } => {
                let value = cpu.read_register(reg);
                let result = value >> 1;
                cpu.write_register(reg, result);

                cpu.update_flag(Flags::ZERO, result == 0);
                cpu.update_flag(Flags::SUBTRACT, false);
                cpu.update_flag(Flags::HALF_CARRY, false);
                cpu.update_flag(Flags::CARRY, value & 0x01 != 0);

                Ok(instruction.cycles.0)
            }
            Instruction {
                opcode: Opcode::Srl,
                lhs: Some(Operand::Reg16(Register::HL, AddressingMode::Indirect)),
                ..
            } => {
                let addr = cpu.read_register16(&Register::HL);
                let value = mmu.read(addr)?;
                let result = value >> 1;
                mmu.write(addr, result)?;

                cpu.update_flag(Flags::ZERO, result == 0);
                cpu.update_flag(Flags::SUBTRACT, false);
                cpu.update_flag(Flags::HALF_CARRY, false);
                cpu.update_flag(Flags::CARRY, value & 0x01 != 0);

                Ok(instruction.cycles.0)
            }
            _ => invalid_handler!(instruction),
        }
    }

    #[inline]
    pub fn swap(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs => instruction);

        let result = match instruction {
            Instruction {
                opcode: Opcode::Swap,
                lhs: Some(Operand::Reg8(reg, _)),
                ..
            } => {
                let value = cpu.read_register(reg);
                let result = (value >> 4) | (value << 4);
                cpu.write_register(reg, result);
                result
            }
            Instruction {
                opcode: Opcode::Swap,
                lhs: Some(Operand::Reg16(Register::HL, AddressingMode::Indirect)),
                ..
            } => {
                let addr = cpu.read_register16(&Register::HL);
                let value = mmu.read(addr)?;
                let result = (value >> 4) | (value << 4);
                mmu.write(addr, result)?;
                result
            }
            _ => return invalid_handler!(instruction),
        };

        cpu.update_flag(Flags::ZERO, result == 0);
        cpu.update_flag(Flags::SUBTRACT, false);
        cpu.update_flag(Flags::HALF_CARRY, false);
        cpu.update_flag(Flags::CARRY, false);

        Ok(instruction.cycles.0)
    }

    #[inline]
    pub fn reset_bit(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs_rhs => instruction);

        match instruction {
            Instruction {
                lhs: Some(Operand::Bit(bit)),
                rhs: Some(Operand::Reg8(register, AddressingMode::Direct)),
                ..
            } => {
                let value = cpu.read_register(register);
                let result = value & !(1 << *bit);
                cpu.write_register(register, result);

                Ok(instruction.cycles.0)
            }
            Instruction {
                lhs: Some(Operand::Bit(bit)),
                rhs: Some(Operand::Reg16(register, AddressingMode::Indirect)),
                ..
            } => {
                let addr = cpu.read_register16(register);
                let value = mmu.read(addr)?;
                let result = value & !(1 << *bit);
                mmu.write(addr, result)?;

                Ok(instruction.cycles.0)
            }
            _ => invalid_handler!(instruction),
        }
    }

    #[inline]
    pub fn set_bit(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs_rhs => instruction);

        match instruction {
            Instruction {
                lhs: Some(Operand::Bit(bit)),
                rhs: Some(Operand::Reg8(register, AddressingMode::Direct)),
                ..
            } => {
                let value = cpu.read_register(register);
                let result = value | (1 << *bit);
                cpu.write_register(register, result);

                Ok(instruction.cycles.0)
            }
            Instruction {
                lhs: Some(Operand::Bit(bit)),
                rhs: Some(Operand::Reg16(register, AddressingMode::Indirect)),
                ..
            } => {
                let addr = cpu.read_register16(register);
                let value = mmu.read(addr)?;
                let result = value | (1 << *bit);
                mmu.write(addr, result)?;

                Ok(instruction.cycles.0)
            }
            _ => invalid_handler!(instruction),
        }
    }

    #[inline]
    pub fn compare(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs_rhs => instruction);

        let x = Handlers::resolve_operand(cpu, mmu, instruction.lhs.as_ref().unwrap(), false)? as u8;
        let y = Handlers::resolve_operand(cpu, mmu, instruction.rhs.as_ref().unwrap(), false)? as u8;

        let result = x.wrapping_sub(y);
        cpu.update_flag(Flags::ZERO, result == 0);
        cpu.update_flag(Flags::SUBTRACT, true);
        cpu.update_flag(Flags::HALF_CARRY, (x & 0x0f) < (y & 0x0f));
        cpu.update_flag(Flags::CARRY, result > x);

        Ok(instruction.cycles.0)
    }

    #[inline]
    pub fn test_bit(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs_rhs => instruction);

        let register = Handlers::resolve_operand(cpu, mmu, instruction.rhs.as_ref().unwrap(), false)? as u8;
        let bit = Handlers::resolve_operand(cpu, mmu, instruction.lhs.as_ref().unwrap(), false)? as u8;

        let result = register & (1 << bit);
        cpu.update_flag(Flags::ZERO, result == 0);
        cpu.update_flag(Flags::SUBTRACT, false);
        cpu.update_flag(Flags::HALF_CARRY, true);

        Ok(instruction.cycles.0)
    }

    #[inline]
    pub fn halt(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        cpu.halted = true;

        Ok(instruction.cycles.0)
    }

    #[inline]
    pub fn stop(cpu: &mut Cpu, mmu: &mut Mmu, timer: &mut Timer, instruction: &Instruction) -> Result<usize, AyyError> {
        timer.reset_divider(mmu);
        mmu.enable_pending_speed_switch();
        Ok(instruction.cycles.0)
    }

    #[inline]
    pub fn jump(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs_rhs => instruction);

        match instruction.opcode {
            Opcode::Jp => {
                if let Some(Operand::Conditional(cond)) = instruction.lhs.as_ref() {
                    return if Handlers::check_condition(cpu, cond) {
                        let addr =
                            Handlers::resolve_operand(cpu, mmu, instruction.rhs.as_ref().unwrap(), false)? as u16;
                        cpu.write_register16(&Register::PC, addr);
                        Ok(instruction.cycles.0)
                    } else {
                        Ok(instruction.cycles.1.unwrap())
                    };
                }
            }
            Opcode::Jr => {
                if let Some(Operand::Conditional(cond)) = instruction.lhs.as_ref() {
                    return if Handlers::check_condition(cpu, cond) {
                        let offset =
                            Handlers::resolve_operand(cpu, mmu, instruction.rhs.as_ref().unwrap(), false)? as i8;
                        let pc = cpu.read_register16(&Register::PC);
                        cpu.write_register16(&Register::PC, pc.wrapping_add_signed(offset as i16));
                        Ok(instruction.cycles.0)
                    } else {
                        Ok(instruction.cycles.1.unwrap())
                    };
                }
            }
            Opcode::Call => {
                if let Some(Operand::Conditional(cond)) = instruction.lhs.as_ref() {
                    return if Handlers::check_condition(cpu, cond) {
                        let addr =
                            Handlers::resolve_operand(cpu, mmu, instruction.rhs.as_ref().unwrap(), false)? as u16;
                        let pc = cpu.read_register16(&Register::PC);
                        // We already increased the PC by 3, so we need to push the current PC + 3
                        cpu.push_stack(mmu, pc)?;
                        cpu.write_register16(&Register::PC, addr);
                        Ok(instruction.cycles.0)
                    } else {
                        Ok(instruction.cycles.1.unwrap())
                    };
                }
            }
            _ => return invalid_handler!(instruction),
        }

        invalid_handler!(instruction)
    }

    #[inline]
    pub fn restart(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs => instruction);

        let addr = Handlers::resolve_operand(cpu, mmu, instruction.lhs.as_ref().unwrap(), false)? as u16;
        let pc = cpu.read_register16(&Register::PC);
        cpu.push_stack(mmu, pc)?;
        cpu.write_register16(&Register::PC, addr);

        Ok(instruction.cycles.0)
    }

    #[inline]
    pub fn ret(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        match instruction.opcode {
            Opcode::Ret => {
                ensure!(lhs => instruction);
                if let Some(Operand::Conditional(cond)) = instruction.lhs.as_ref() {
                    if Handlers::check_condition(cpu, cond) {
                        let addr = cpu.pop_stack(mmu)?;
                        cpu.write_register16(&Register::PC, addr);
                    }
                    Ok(instruction.cycles.0)
                } else {
                    Ok(instruction.cycles.1.unwrap())
                }
            }
            Opcode::Reti => {
                let addr = cpu.pop_stack(mmu)?;
                cpu.write_register16(&Register::PC, addr);
                cpu.enable_interrupts(false);
                Ok(instruction.cycles.0)
            }
            _ => invalid_handler!(instruction),
        }
    }

    #[inline]
    pub fn push(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs => instruction);

        let operand = instruction.lhs.as_ref().unwrap();
        match operand {
            Operand::Reg16(reg, _) => {
                let value = cpu.read_register16(reg);
                cpu.push_stack(mmu, value)?;
            }
            _ => return invalid_handler!(instruction),
        }

        Ok(instruction.cycles.0)
    }

    #[inline]
    pub fn pop(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs => instruction);

        let operand = instruction.lhs.as_ref().unwrap();
        match operand {
            Operand::Reg16(reg, _) => {
                let value = cpu.pop_stack(mmu)?;
                cpu.write_register16(reg, value);
            }
            _ => return invalid_handler!(instruction),
        }

        Ok(instruction.cycles.0)
    }

    #[inline]
    pub fn increment(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs => instruction);

        let operand = instruction.lhs.as_ref().unwrap();
        match operand {
            Operand::Reg8(reg, _) => {
                let value = cpu.read_register(reg);
                let result = value.wrapping_add(1);
                cpu.write_register(reg, result);

                cpu.update_flag(Flags::ZERO, result == 0);
                cpu.update_flag(Flags::SUBTRACT, false);
                cpu.update_flag(Flags::HALF_CARRY, (value & 0x0f) == 0x0f);
            }
            Operand::Reg16(reg, mode) => {
                if mode.contains(AddressingMode::Indirect) {
                    let addr = cpu.read_register16(reg);
                    let value = mmu.read(addr)?;
                    let result = value.wrapping_add(1);
                    mmu.write(addr, result)?;

                    cpu.update_flag(Flags::ZERO, result == 0);
                    cpu.update_flag(Flags::SUBTRACT, false);
                    cpu.update_flag(Flags::HALF_CARRY, (value & 0x0f) == 0x0f);
                } else {
                    let value = cpu.read_register16(reg);
                    let result = value.wrapping_add(1);
                    cpu.write_register16(reg, result);
                }
            }
            _ => return invalid_handler!(instruction),
        }

        Ok(instruction.cycles.0)
    }

    #[inline]
    pub fn decrement(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs => instruction);

        let operand = instruction.lhs.as_ref().unwrap();
        match operand {
            Operand::Reg8(reg, _) => {
                let value = cpu.read_register(reg);
                let result = value.wrapping_sub(1);
                cpu.write_register(reg, result);

                cpu.update_flag(Flags::ZERO, result == 0);
                cpu.update_flag(Flags::SUBTRACT, true);
                cpu.update_flag(Flags::HALF_CARRY, (value & 0x0f) == 0);
            }
            Operand::Reg16(reg, mode) => {
                if mode.contains(AddressingMode::Indirect) {
                    let addr = cpu.read_register16(reg);
                    let value = mmu.read(addr)?;
                    let result = value.wrapping_sub(1);
                    mmu.write(addr, result)?;

                    cpu.update_flag(Flags::ZERO, result == 0);
                    cpu.update_flag(Flags::SUBTRACT, true);
                    cpu.update_flag(Flags::HALF_CARRY, (value & 0x0f) == 0);
                } else {
                    let value = cpu.read_register16(reg);
                    let result = value.wrapping_sub(1);
                    cpu.write_register16(reg, result);
                }
            }
            _ => return invalid_handler!(instruction),
        }

        Ok(instruction.cycles.0)
    }

    #[inline]
    pub fn add_with_carry(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs_rhs => instruction);

        let x = Handlers::resolve_operand(cpu, mmu, instruction.lhs.as_ref().unwrap(), false)? as u8;
        let y = Handlers::resolve_operand(cpu, mmu, instruction.rhs.as_ref().unwrap(), false)? as u8;
        let carry = cpu.read_flag(Flags::CARRY) as u8;

        let result = x.wrapping_add(y).wrapping_add(carry);
        cpu.write_register(&Register::A, result);

        cpu.update_flag(Flags::ZERO, result == 0);
        cpu.update_flag(Flags::SUBTRACT, false);
        cpu.update_flag(Flags::HALF_CARRY, (x & 0x0f) + (y & 0x0f) + carry > 0x0f);
        cpu.update_flag(Flags::CARRY, (x as u16) + (y as u16) + (carry as u16) > 0xff);

        Ok(instruction.cycles.0)
    }

    #[inline]
    pub fn sub_with_carry(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs_rhs => instruction);

        let x = Handlers::resolve_operand(cpu, mmu, instruction.lhs.as_ref().unwrap(), false)? as u8;
        let y = Handlers::resolve_operand(cpu, mmu, instruction.rhs.as_ref().unwrap(), false)? as u8;
        let carry = cpu.read_flag(Flags::CARRY) as u8;

        let result = x.wrapping_sub(y).wrapping_sub(carry);
        cpu.write_register(&Register::A, result);

        cpu.update_flag(Flags::ZERO, result == 0);
        cpu.update_flag(Flags::SUBTRACT, true);
        cpu.update_flag(Flags::HALF_CARRY, (x & 0x0f) < (y & 0x0f) + carry);
        cpu.update_flag(Flags::CARRY, (x as u16) < (y as u16) + (carry as u16));

        Ok(instruction.cycles.0)
    }

    #[inline]
    pub fn handle_interrupt(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        if instruction.opcode == Opcode::Ei {
            cpu.enable_interrupts(true);
        } else {
            cpu.disable_interrupts();
        }

        Ok(instruction.cycles.0)
    }

    #[inline]
    fn resolve_operand(cpu: &mut Cpu, mmu: &Mmu, operand: &Operand, is_ldh: bool) -> Result<usize, AyyError> {
        match operand {
            Operand::Reg16(reg, mode) if mode.contains(AddressingMode::Direct) => {
                Ok(cpu.read_register16(&reg) as usize)
            }
            Operand::Reg16(reg, mode) if mode.contains(AddressingMode::Indirect) => {
                let addr = cpu.read_register16(&reg);
                Handlers::process_additional_address_mode(cpu, reg, addr, mode);
                Ok(mmu.read16(addr)? as usize)
            }
            Operand::Reg8(reg, mode) if mode.contains(AddressingMode::Direct) => Ok(cpu.read_register(&reg) as usize),
            Operand::Reg8(reg, mode) if mode.contains(AddressingMode::Indirect) => {
                // ld a, (c)
                let addr = cpu.read_register(&reg);
                Ok(mmu.read(0xff00 + addr as u16)? as usize)
            }
            Operand::Imm16(imm, mode) if mode.contains(AddressingMode::Direct) => Ok(*imm as usize),
            Operand::Imm16(imm, mode) if mode.contains(AddressingMode::Indirect) => Ok(mmu.read16(*imm)? as usize),
            Operand::Imm8(imm, mode) if mode.contains(AddressingMode::Direct) => Ok(*imm as usize),
            Operand::Imm8(imm, mode) if mode.contains(AddressingMode::Indirect) && is_ldh => {
                // ldh a, (imm)
                let addr = 0xff00 + *imm as u16;
                Ok(mmu.read(addr)? as usize)
            }
            Operand::Bit(bit) => Ok(*bit as usize),
            Operand::Offset(offset) => Ok(*offset as usize),
            Operand::DisplacedReg16(reg, offset, mode) if mode.contains(AddressingMode::Direct) => {
                Ok(cpu.read_register16(reg).wrapping_add_signed(*offset as i16) as usize)
            }
            _ => Err(UnresolvedTarget {
                target: operand.clone(),
            }),
        }
    }

    #[inline]
    fn process_additional_address_mode(cpu: &mut Cpu, reg: &Register, addr: u16, mode: &AddressingMode) {
        if mode.contains(AddressingMode::Increment) {
            cpu.write_register16(&reg, addr.wrapping_add(1));
        } else if mode.contains(AddressingMode::Decrement) {
            cpu.write_register16(&reg, addr.wrapping_sub(1));
        }
    }

    #[inline]
    fn check_condition(cpu: &Cpu, condition: &Condition) -> bool {
        match condition {
            Condition::Z => cpu.read_flag(Flags::ZERO),
            Condition::NZ => !cpu.read_flag(Flags::ZERO),
            Condition::C => cpu.read_flag(Flags::CARRY),
            Condition::NC => !cpu.read_flag(Flags::CARRY),
            Condition::None => true,
        }
    }
}
