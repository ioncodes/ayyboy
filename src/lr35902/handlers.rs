use crate::lr35902::cpu::Cpu;
use crate::lr35902::sm83::{AddressingMode, Instruction, Operand};
use crate::memory::mmu::Mmu;
use bitflags::Flags;

pub struct Handlers {}

impl Handlers {
    pub fn load(cpu: &mut Cpu, mmu: &mut Mmu, instruction: &Instruction) -> Result<usize, &'static str> {
        if instruction.source.is_none() || instruction.destination.is_none() {
            return Err("Invalid load instruction");
        }

        let src = instruction.source.as_ref().unwrap();
        let src = Handlers::resolve_operand(cpu, mmu, src);

        match instruction.destination.as_ref().unwrap() {
            Operand::Reg8(reg, _) => cpu.write_register(&reg, src as u8),
            Operand::Reg16(reg, mode) => {
                if mode.contains(AddressingMode::Indirect) {
                    let addr = cpu.read_register16(&reg);
                    mmu.write16(addr, src as u16);

                    if mode.contains(AddressingMode::Increment) {
                        cpu.write_register16(&reg, addr.wrapping_add(1));
                    } else if mode.contains(AddressingMode::Decrement) {
                        cpu.write_register16(&reg, addr.wrapping_sub(1));
                    }
                } else {
                    cpu.write_register16(&reg, src as u16);
                }
            }
            _ => return Err("Unimplemented destination"),
        };

        Ok(instruction.cycles.0)
    }

    fn resolve_operand(cpu: &Cpu, mmu: &Mmu, operand: &Operand) -> usize {
        let target = match operand {
            Operand::Reg16(reg, mode) if mode.contains(AddressingMode::Direct) => cpu.read_register16(&reg),
            Operand::Reg16(reg, mode) if mode.contains(AddressingMode::Indirect) => {
                let addr = cpu.read_register16(&reg);
                mmu.read16(addr)
            }
            Operand::Reg8(reg, mode) if mode.contains(AddressingMode::Direct) => cpu.read_register(&reg) as u16,
            Operand::Imm16(imm, mode) if mode.contains(AddressingMode::Direct) => *imm,
            Operand::Imm8(imm, mode) if mode.contains(AddressingMode::Direct) => *imm as u16,
            _ => panic!("Unimplemented operand: {:?}", operand),
        };
        target as usize
    }
}
