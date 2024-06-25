use crate::error::AyyError;
use crate::error::AyyError::{InvalidHandler, UnresolvedTarget};
use crate::lr35902::cpu::{Cpu, Flags};
use crate::lr35902::sm83::{AddressingMode, Condition, Instruction, Opcode, Operand, Register};
use crate::memory::mmu::Mmu;
use crate::memory::{INTERRUPT_ENABLE_REGISTER, INTERRUPT_FLAGS_REGISTER};

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
    pub fn load(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs_rhs => instruction);

        // In case of LDH we need to make sure to add 0xff00 to dst/src
        let is_ldh_instruction = instruction.opcode == Opcode::Ldh;
        let src = instruction.rhs.as_ref().unwrap();
        let src = Handlers::resolve_operand(cpu, mmu, src, is_ldh_instruction)?;

        match instruction.lhs.as_ref().unwrap() {
            Operand::Reg8(reg, mode) => {
                if mode.contains(AddressingMode::Indirect) {
                    // ld (c), a
                    let addr = cpu.read_register(reg);
                    mmu.write(0xff00 + addr as u16, src as u8);
                } else {
                    cpu.write_register(&reg, src as u8);
                }
            }
            Operand::Reg16(reg, mode) => {
                if mode.contains(AddressingMode::Indirect) {
                    let addr = cpu.read_register16(&reg);
                    Handlers::process_additional_address_mode(cpu, reg, addr, mode);
                    mmu.write(addr, src as u8);
                } else {
                    cpu.write_register16(&reg, src as u16);
                }
            }
            Operand::Imm8(imm, mode) if is_ldh_instruction && mode.contains(AddressingMode::Indirect) => {
                let addr = 0xff00 + *imm as u16;
                mmu.write(addr, src as u8);
            }
            Operand::Imm16(imm, mode) if mode.contains(AddressingMode::Indirect) => mmu.write16(*imm, src as u16),
            _ => return invalid_handler!(instruction),
        };

        Ok(instruction.cycles.0)
    }

    pub fn nop(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        Ok(instruction.cycles.0)
    }

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

    pub fn add(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs_rhs => instruction);

        let x = Handlers::resolve_operand(cpu, mmu, instruction.lhs.as_ref().unwrap(), false)? as u8;
        let y = Handlers::resolve_operand(cpu, mmu, instruction.rhs.as_ref().unwrap(), false)? as u8;

        let result = x.wrapping_add(y);
        cpu.write_register(&Register::A, result);

        cpu.update_flag(Flags::ZERO, result == 0);
        cpu.update_flag(Flags::SUBTRACT, false);
        cpu.update_flag(Flags::HALF_CARRY, (x & 0x0f) + (y & 0x0f) > 0x0f);
        cpu.update_flag(Flags::CARRY, result < x);

        Ok(instruction.cycles.0)
    }

    pub fn sub(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs_rhs => instruction);

        let x = Handlers::resolve_operand(cpu, mmu, instruction.lhs.as_ref().unwrap(), false)? as u8;
        let y = Handlers::resolve_operand(cpu, mmu, instruction.rhs.as_ref().unwrap(), false)? as u8;

        let result = x.wrapping_sub(y);
        cpu.write_register(&Register::A, result);

        cpu.update_flag(Flags::ZERO, result == 0);
        cpu.update_flag(Flags::SUBTRACT, true);
        cpu.update_flag(Flags::HALF_CARRY, (x & 0x0f) < (y & 0x0f));
        cpu.update_flag(Flags::CARRY, result > x);

        Ok(instruction.cycles.0)
    }

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

    pub fn rotate_left(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        let (reg, is_rla) = if instruction.opcode == Opcode::Rla {
            (&Register::A, true)
        } else if let Some(Operand::Reg8(reg, _)) = instruction.lhs.as_ref() {
            (reg, false)
        } else {
            return invalid_handler!(instruction);
        };

        let value = cpu.read_register(reg);
        let carry = cpu.read_flag(Flags::CARRY) as u8;
        let result = (value << 1) | carry;
        cpu.write_register(reg, result);

        cpu.update_flag(Flags::ZERO, if !is_rla { result == 0 } else { false });
        cpu.update_flag(Flags::SUBTRACT, false);
        cpu.update_flag(Flags::HALF_CARRY, false);
        cpu.update_flag(Flags::CARRY, value & 0x80 != 0);

        Ok(instruction.cycles.0)
    }

    pub fn reset_bit(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs_rhs => instruction);

        let register = Handlers::resolve_operand(cpu, mmu, instruction.rhs.as_ref().unwrap(), false)? as u8;
        let bit = Handlers::resolve_operand(cpu, mmu, instruction.lhs.as_ref().unwrap(), false)? as u8;

        let result = register & !(1 << bit);
        cpu.write_register(&Register::A, result);

        Ok(instruction.cycles.0)
    }

    pub fn set_bit(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs_rhs => instruction);

        let register = Handlers::resolve_operand(cpu, mmu, instruction.rhs.as_ref().unwrap(), false)? as u8;
        let bit = Handlers::resolve_operand(cpu, mmu, instruction.lhs.as_ref().unwrap(), false)? as u8;

        let result = register | (1 << bit);
        cpu.write_register(&Register::A, result);

        Ok(instruction.cycles.0)
    }

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

    pub fn halt(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        // HALT mode is exited when a flag in register IF is set and the corresponding flag in IE is also set,
        // regardless of the value of IME. The only difference is that IME='1' will make the CPU jump to the
        // interrupt vector (and clear the IF flag), while IME='0' will only make the CPU continue executing
        // instructions, but the jump won't be performed (and the IF flag won't be cleared).

        let interrupt_enable = mmu.read(INTERRUPT_ENABLE_REGISTER);
        let interrupt_flags = mmu.read(INTERRUPT_FLAGS_REGISTER);

        if interrupt_enable & interrupt_flags == 0 {
            // We need to set the PC back to HALT to make sure we land here again
            let addr_of_halt = cpu.read_register16(&Register::PC).wrapping_sub(instruction.length as u16);
            cpu.write_register16(&Register::PC, addr_of_halt);
        }

        Ok(instruction.cycles.0)
    }

    pub fn jump(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs_rhs => instruction);

        match instruction.opcode {
            Opcode::Jp => {
                if let Some(Operand::Conditional(cond)) = instruction.lhs.as_ref() {
                    return if Handlers::check_condition(cpu, cond) {
                        let addr = Handlers::resolve_operand(cpu, mmu, instruction.rhs.as_ref().unwrap(), false)? as u16;
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
                        let offset = Handlers::resolve_operand(cpu, mmu, instruction.rhs.as_ref().unwrap(), false)? as i8;
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
                        let addr = Handlers::resolve_operand(cpu, mmu, instruction.rhs.as_ref().unwrap(), false)? as u16;
                        let pc = cpu.read_register16(&Register::PC);
                        // We already increased the PC by 3, so we need to push the current PC + 3
                        cpu.push_stack(mmu, pc);
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

    pub fn ret(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs => instruction);

        if let Some(Operand::Conditional(cond)) = instruction.lhs.as_ref() {
            if Handlers::check_condition(cpu, cond) {
                let addr = cpu.pop_stack(mmu);
                cpu.write_register16(&Register::PC, addr);
                return Ok(instruction.cycles.0);
            }

            Ok(instruction.cycles.1.unwrap())
        } else {
            invalid_handler!(instruction)
        }
    }

    pub fn push(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs => instruction);

        let operand = instruction.lhs.as_ref().unwrap();
        match operand {
            Operand::Reg16(reg, _) => {
                let value = cpu.read_register16(reg);
                cpu.push_stack(mmu, value);
            }
            _ => return invalid_handler!(instruction),
        }

        Ok(instruction.cycles.0)
    }

    pub fn pop(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        ensure!(lhs => instruction);

        let operand = instruction.lhs.as_ref().unwrap();
        match operand {
            Operand::Reg16(reg, _) => {
                let value = cpu.pop_stack(mmu);
                cpu.write_register16(reg, value);
            }
            _ => return invalid_handler!(instruction),
        }

        Ok(instruction.cycles.0)
    }

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
                    let value = mmu.read(addr);
                    let result = value.wrapping_add(1);
                    mmu.write(addr, result);

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
                    let value = mmu.read(addr);
                    let result = value.wrapping_sub(1);
                    mmu.write(addr, result);

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

    pub fn handle_interrupt(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, AyyError> {
        if instruction.opcode == Opcode::Ei {
            cpu.enable_interrupts();
        } else {
            cpu.disable_interrupts();
        }

        Ok(instruction.cycles.0)
    }

    fn resolve_operand(cpu: &mut Cpu, mmu: &Mmu, operand: &Operand, is_ldh: bool) -> Result<usize, AyyError> {
        match operand {
            Operand::Reg16(reg, mode) if mode.contains(AddressingMode::Direct) => Ok(cpu.read_register16(&reg) as usize),
            Operand::Reg16(reg, mode) if mode.contains(AddressingMode::Indirect) => {
                let addr = cpu.read_register16(&reg);
                Handlers::process_additional_address_mode(cpu, reg, addr, mode);
                Ok(mmu.read16(addr) as usize)
            }
            Operand::Reg8(reg, mode) if mode.contains(AddressingMode::Direct) => Ok(cpu.read_register(&reg) as usize),
            Operand::Reg8(reg, mode) if mode.contains(AddressingMode::Indirect) => {
                // ld a, (c)
                let addr = cpu.read_register(&reg);
                Ok(mmu.read(0xff00 + addr as u16) as usize)
            }
            Operand::Imm16(imm, mode) if mode.contains(AddressingMode::Direct) => Ok(*imm as usize),
            Operand::Imm8(imm, mode) if mode.contains(AddressingMode::Direct) => Ok(*imm as usize),
            Operand::Imm8(imm, mode) if mode.contains(AddressingMode::Indirect) && is_ldh => {
                // ldh a, (imm)
                let addr = 0xff00 + *imm as u16;
                Ok(mmu.read(addr) as usize)
            }
            Operand::Bit(bit) => Ok(*bit as usize),
            Operand::Offset(offset) => Ok(*offset as usize),
            _ => Err(UnresolvedTarget { target: operand.clone() }),
        }
    }

    fn process_additional_address_mode(cpu: &mut Cpu, reg: &Register, addr: u16, mode: &AddressingMode) {
        if mode.contains(AddressingMode::Increment) {
            cpu.write_register16(&reg, addr.wrapping_add(1));
        } else if mode.contains(AddressingMode::Decrement) {
            cpu.write_register16(&reg, addr.wrapping_sub(1));
        }
    }

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
