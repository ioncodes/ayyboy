use crate::error::AyyError;
use crate::memory::mmu::Mmu;
use bitflags::bitflags;
use rhai::{CustomType, TypeBuilder};
use std::cmp::PartialEq;
use std::collections::HashMap;

type FDecode = fn(&Mmu, u16, Opcode) -> Result<Instruction, AyyError>;

#[derive(PartialEq, Debug, Clone)]
pub enum Register {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    F,
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

bitflags! {
    #[derive(PartialEq, Debug, Clone)]
    pub struct AddressingMode: u8 {
        const Direct    = 0b0001;
        const Indirect  = 0b0010;
        const Increment = 0b0100;
        const Decrement = 0b1000;
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Condition {
    None,
    NZ,
    Z,
    NC,
    C,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Reg8(Register, AddressingMode),
    Reg16(Register, AddressingMode),
    Imm8(u8, AddressingMode),
    Imm16(u16, AddressingMode),
    Conditional(Condition),
    DisplacedReg16(Register, i8, AddressingMode),
    Offset(i8),
    Bit(u8),
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Opcode {
    Nop,
    Ld,
    Inc,
    Dec,
    Rlc,
    Rrc,
    Swap,
    Rr,
    Srl,
    Bit,
    Res,
    Set,
    Jp,
    Jr,
    Call,
    Ret,
    Rst,
    Push,
    Pop,
    Add,
    Adc,
    Sub,
    Sbc,
    And,
    Xor,
    Or,
    Cp,
    Reti,
    Halt,
    Stop,
    Di,
    Ei,
    Ldh,
    Ldl,
    Rl,
    Sla,
    Sra,
    Ccf,
    Scf,
    Cpl,
    Daa,
    Rra,
    Rla,
    Rrca,
    Rlca,
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: Opcode,
    pub lhs: Option<Operand>,
    pub rhs: Option<Operand>,
    pub length: usize,
    pub cycles: (usize, Option<usize>),
}

macro_rules! define_decoder {
    ( $pattern:expr, $opcode:expr, $function:expr ) => {{
        (String::from($pattern), $opcode, $function)
    }};
}

#[derive(Clone, CustomType)]
pub struct Sm83 {
    decoder_lut: Vec<(String, Opcode, FDecode)>,
    decoder_lut_prefixed: Vec<(String, Opcode, FDecode)>,
    cached_lut: HashMap<u8, Instruction>,
    cached_lut_prefixed: HashMap<u8, Instruction>,
    invalid_opcodes_lut: Vec<u8>,
}

//noinspection DuplicatedCode
impl Sm83 {
    pub fn new() -> Sm83 {
        let mut decoder_lut = Vec::new();
        let mut decoder_lut_prefixed = Vec::new();

        Sm83::propagate_decoders(&mut decoder_lut);
        Sm83::propagate_decoders_prefixed(&mut decoder_lut_prefixed);

        Sm83 {
            decoder_lut,
            decoder_lut_prefixed,
            cached_lut: HashMap::new(),
            cached_lut_prefixed: HashMap::new(),
            invalid_opcodes_lut: vec![0xd3, 0xdb, 0xdd, 0xe3, 0xe4, 0xeb, 0xec, 0xed, 0xf4, 0xfc, 0xfd],
        }
    }

    pub fn decode(&mut self, mmu: &mut Mmu, current_pc: u16) -> Result<Instruction, AyyError> {
        let mut opcode_byte = mmu.read(current_pc);

        #[cfg(debug_assertions)]
        if self.invalid_opcodes_lut.contains(&opcode_byte) {
            return Err(AyyError::IllegalOpcode { opcode: opcode_byte });
        }

        let mut prefix = false;
        if opcode_byte == 0xcb {
            opcode_byte = mmu.read(current_pc.wrapping_add(1));
            prefix = true;
        }

        let cached_lut = if prefix { &self.cached_lut_prefixed } else { &self.cached_lut };
        if let Some(instruction) = cached_lut.get(&opcode_byte) {
            let mut instruction = instruction.clone();

            instruction.lhs = match instruction.lhs {
                Some(Operand::Imm8(_, mode)) => Some(Operand::Imm8(mmu.read(current_pc.wrapping_add(1)), mode)),
                Some(Operand::Imm16(_, mode)) => Some(Operand::Imm16(mmu.read16(current_pc.wrapping_add(1)), mode)),
                Some(Operand::Offset(_)) => Some(Operand::Offset(mmu.read(current_pc.wrapping_add(1)) as i8)),
                Some(Operand::DisplacedReg16(reg, _, mode)) => {
                    Some(Operand::DisplacedReg16(reg, mmu.read(current_pc.wrapping_add(1)) as i8, mode))
                }
                _ => instruction.lhs,
            };

            instruction.rhs = match instruction.rhs {
                Some(Operand::Imm8(_, mode)) => Some(Operand::Imm8(mmu.read(current_pc.wrapping_add(1)), mode)),
                Some(Operand::Imm16(_, mode)) => Some(Operand::Imm16(mmu.read16(current_pc.wrapping_add(1)), mode)),
                Some(Operand::Offset(_)) => Some(Operand::Offset(mmu.read(current_pc.wrapping_add(1)) as i8)),
                Some(Operand::DisplacedReg16(reg, _, mode)) => {
                    Some(Operand::DisplacedReg16(reg, mmu.read(current_pc.wrapping_add(1)) as i8, mode))
                }
                _ => instruction.rhs,
            };

            return Ok(instruction);
        }

        let opcode_str = format!("{:08b}", opcode_byte);
        let lut = if prefix { &self.decoder_lut_prefixed } else { &self.decoder_lut };

        for (pattern, opcode, decoder_fn) in lut {
            if pattern.len() != opcode_str.len() {
                continue;
            }

            let mut matched = true;
            for (i, c) in pattern.chars().enumerate() {
                if c != 'x' && c != opcode_str.chars().nth(i).unwrap() {
                    matched = false;
                    break;
                }
            }

            if matched {
                let instruction = decoder_fn(mmu, current_pc, *opcode)?;
                if prefix {
                    self.cached_lut_prefixed.insert(opcode_byte, instruction.clone());
                } else {
                    self.cached_lut.insert(opcode_byte, instruction.clone());
                }
                return Ok(instruction);
            }
        }

        Err(AyyError::DecoderFailure {
            opcode: mmu.read(current_pc),
            address: current_pc,
        })
    }

    fn lookup_register(data: u8) -> Result<Register, AyyError> {
        match data {
            0b000 => Ok(Register::B),
            0b001 => Ok(Register::C),
            0b010 => Ok(Register::D),
            0b011 => Ok(Register::E),
            0b100 => Ok(Register::H),
            0b101 => Ok(Register::L),
            0b110 => Ok(Register::HL),
            0b111 => Ok(Register::A),
            _ => Err(AyyError::UnknownRegisterBits { data }),
        }
    }

    fn lookup_register_16(data: u8) -> Result<Register, AyyError> {
        match data {
            0b00 => Ok(Register::BC),
            0b01 => Ok(Register::DE),
            0b10 => Ok(Register::HL),
            0b11 => Ok(Register::SP),
            _ => Err(AyyError::UnknownRegisterBits { data }),
        }
    }

    fn lookup_condition_3bits(data: u8) -> Result<Condition, AyyError> {
        match data {
            0b011 => Ok(Condition::None),
            0b100 => Ok(Condition::NZ),
            0b101 => Ok(Condition::Z),
            0b110 => Ok(Condition::NC),
            0b111 => Ok(Condition::C),
            _ => Err(AyyError::UnknownConditionBits { data }),
        }
    }

    fn lookup_condition_2bits(data: u8) -> Result<Condition, AyyError> {
        match data {
            0b00 => Ok(Condition::NZ),
            0b01 => Ok(Condition::Z),
            0b10 => Ok(Condition::NC),
            0b11 => Ok(Condition::C),
            _ => Err(AyyError::UnknownConditionBits { data }),
        }
    }

    fn decode_8bit_operand(value: u8, base_cycles: usize, hl_cycles: usize) -> Result<(Operand, usize), AyyError> {
        let operand = if value == 0b110 {
            Operand::Reg16(Register::HL, AddressingMode::Indirect)
        } else {
            Operand::Reg8(Sm83::lookup_register(value)?, AddressingMode::Direct)
        };
        let cycles = if value != 0b110 { base_cycles } else { hl_cycles };
        Ok((operand, cycles))
    }

    fn propagate_decoders(lut: &mut Vec<(String, Opcode, FDecode)>) {
        // nop
        lut.push(define_decoder!("00000000", Opcode::Nop, |_, _, opcode| {
            Ok(Instruction {
                opcode,
                lhs: None,
                rhs: None,
                length: 1,
                cycles: (4, None),
            })
        }));

        // cpl
        lut.push(define_decoder!("00101111", Opcode::Cpl, |_, _, opcode| {
            Ok(Instruction {
                opcode,
                lhs: None,
                rhs: None,
                length: 1,
                cycles: (4, None),
            })
        }));

        // ccf
        lut.push(define_decoder!("00111111", Opcode::Ccf, |_, _, opcode| {
            Ok(Instruction {
                opcode,
                lhs: None,
                rhs: None,
                length: 1,
                cycles: (4, None),
            })
        }));

        // ld (imm16), SP
        lut.push(define_decoder!("00001000", Opcode::Ld, |mmu, pc, opcode| {
            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Imm16(mmu.read16(pc.wrapping_add(1)), AddressingMode::Indirect)),
                rhs: Some(Operand::Reg16(Register::SP, AddressingMode::Direct)),
                length: 3,
                cycles: (20, None),
            })
        }));

        // stop imm8
        lut.push(define_decoder!("00010000", Opcode::Stop, |mmu, pc, opcode| {
            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Imm8(mmu.read(pc.wrapping_add(1)), AddressingMode::Direct)),
                rhs: None,
                length: 2,
                cycles: (4, None),
            })
        }));

        // daa
        lut.push(define_decoder!("00100111", Opcode::Daa, |_, _, opcode| {
            Ok(Instruction {
                opcode,
                lhs: None,
                rhs: None,
                length: 1,
                cycles: (4, None),
            })
        }));

        // scf
        lut.push(define_decoder!("00110111", Opcode::Scf, |_, _, opcode| {
            Ok(Instruction {
                opcode,
                lhs: None,
                rhs: None,
                length: 1,
                cycles: (4, None),
            })
        }));

        // add sp, imm8
        lut.push(define_decoder!("11101000", Opcode::Add, |mmu, pc, opcode| {
            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg16(Register::SP, AddressingMode::Direct)),
                rhs: Some(Operand::Offset(mmu.read(pc.wrapping_add(1)) as i8)),
                length: 2,
                cycles: (16, None),
            })
        }));

        // ld hl, sp+/-imm8
        lut.push(define_decoder!("11111000", Opcode::Ld, |mmu, pc, opcode| {
            let offset = mmu.read(pc.wrapping_add(1)) as i8;

            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg16(Register::HL, AddressingMode::Direct)),
                rhs: Some(Operand::DisplacedReg16(Register::SP, offset, AddressingMode::Direct)),
                length: 2,
                cycles: (12, None),
            })
        }));

        // ld sp, hl
        lut.push(define_decoder!("11111001", Opcode::Ld, |_, _, opcode| {
            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg16(Register::SP, AddressingMode::Direct)),
                rhs: Some(Operand::Reg16(Register::HL, AddressingMode::Direct)),
                length: 1,
                cycles: (8, None),
            })
        }));

        // adc a, imm8
        lut.push(define_decoder!("11001110", Opcode::Adc, |mmu, pc, opcode| {
            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg8(Register::A, AddressingMode::Direct)),
                rhs: Some(Operand::Imm8(mmu.read(pc.wrapping_add(1)), AddressingMode::Direct)),
                length: 2,
                cycles: (8, None),
            })
        }));

        // sbc a, imm8
        lut.push(define_decoder!("11011110", Opcode::Sbc, |mmu, pc, opcode| {
            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg8(Register::A, AddressingMode::Direct)),
                rhs: Some(Operand::Imm8(mmu.read(pc.wrapping_add(1)), AddressingMode::Direct)),
                length: 2,
                cycles: (8, None),
            })
        }));

        // xor a, imm8
        lut.push(define_decoder!("11101110", Opcode::Xor, |mmu, pc, opcode| {
            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg8(Register::A, AddressingMode::Direct)),
                rhs: Some(Operand::Imm8(mmu.read(pc.wrapping_add(1)), AddressingMode::Direct)),
                length: 2,
                cycles: (8, None),
            })
        }));

        // cp a, imm8
        lut.push(define_decoder!("11111110", Opcode::Cp, |mmu, pc, opcode| {
            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg8(Register::A, AddressingMode::Direct)),
                rhs: Some(Operand::Imm8(mmu.read(pc.wrapping_add(1)), AddressingMode::Direct)),
                length: 2,
                cycles: (8, None),
            })
        }));

        // halt
        lut.push(define_decoder!("01110110", Opcode::Halt, |_, _, opcode| {
            Ok(Instruction {
                opcode,
                lhs: None,
                rhs: None,
                length: 1,
                cycles: (4, None),
            })
        }));

        // ld (imm16), A
        lut.push(define_decoder!("11101010", Opcode::Ld, |mmu, pc, opcode| {
            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Imm16(mmu.read16(pc + 1), AddressingMode::Indirect)),
                rhs: Some(Operand::Reg8(Register::A, AddressingMode::Direct)),
                length: 3,
                cycles: (16, None),
            })
        }));

        // ld A, (imm16)
        lut.push(define_decoder!("11111010", Opcode::Ld, |mmu, pc, opcode| {
            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg8(Register::A, AddressingMode::Direct)),
                rhs: Some(Operand::Imm16(mmu.read16(pc + 1), AddressingMode::Indirect)),
                length: 3,
                cycles: (16, None),
            })
        }));

        // ldh (imm8), A
        lut.push(define_decoder!("11100000", Opcode::Ldh, |mmu, pc, opcode| {
            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Imm8(mmu.read(pc.wrapping_add(1)), AddressingMode::Indirect)),
                rhs: Some(Operand::Reg8(Register::A, AddressingMode::Direct)),
                length: 2,
                cycles: (12, None),
            })
        }));

        // ldh A, (imm8)
        lut.push(define_decoder!("11110000", Opcode::Ldh, |mmu, pc, opcode| {
            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg8(Register::A, AddressingMode::Direct)),
                rhs: Some(Operand::Imm8(mmu.read(pc.wrapping_add(1)), AddressingMode::Indirect)),
                length: 2,
                cycles: (12, None),
            })
        }));

        // cp A, imm8
        lut.push(define_decoder!("11111110", Opcode::Cp, |mmu, pc, opcode| {
            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg8(Register::A, AddressingMode::Direct)),
                rhs: Some(Operand::Imm8(mmu.read(pc.wrapping_add(1)), AddressingMode::Direct)),
                length: 2,
                cycles: (8, None),
            })
        }));

        // ld (C), A
        lut.push(define_decoder!("11100010", Opcode::Ld, |_, _, opcode| {
            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg8(Register::C, AddressingMode::Indirect)),
                rhs: Some(Operand::Reg8(Register::A, AddressingMode::Direct)),
                length: 1,
                cycles: (8, None),
            })
        }));

        // ld A, (C)
        lut.push(define_decoder!("11110010", Opcode::Ld, |_, _, opcode| {
            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg8(Register::A, AddressingMode::Direct)),
                rhs: Some(Operand::Reg8(Register::C, AddressingMode::Indirect)),
                length: 1,
                cycles: (8, None),
            })
        }));

        // rlca
        lut.push(define_decoder!("00000111", Opcode::Rlca, |_, _, opcode| {
            Ok(Instruction {
                opcode,
                lhs: None,
                rhs: None,
                length: 1,
                cycles: (4, None),
            })
        }));

        // rla
        lut.push(define_decoder!("00010111", Opcode::Rla, |_, _, opcode| {
            Ok(Instruction {
                opcode,
                lhs: None,
                rhs: None,
                length: 1,
                cycles: (4, None),
            })
        }));

        // rrca
        lut.push(define_decoder!("00001111", Opcode::Rrca, |_, _, opcode| {
            Ok(Instruction {
                opcode,
                lhs: None,
                rhs: None,
                length: 1,
                cycles: (4, None),
            })
        }));

        // rra
        lut.push(define_decoder!("00011111", Opcode::Rra, |_, _, opcode| {
            Ok(Instruction {
                opcode,
                lhs: None,
                rhs: None,
                length: 1,
                cycles: (4, None),
            })
        }));

        // halt
        lut.push(define_decoder!("01110110", Opcode::Halt, |_, _, opcode| {
            Ok(Instruction {
                opcode,
                lhs: None,
                rhs: None,
                length: 1,
                cycles: (4, None),
            })
        }));

        // add A, imm8
        lut.push(define_decoder!("11000110", Opcode::Add, |mmu, pc, opcode| {
            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg8(Register::A, AddressingMode::Direct)),
                rhs: Some(Operand::Imm8(mmu.read(pc.wrapping_add(1)), AddressingMode::Direct)),
                length: 2,
                cycles: (8, None),
            })
        }));

        // sub A, imm8
        lut.push(define_decoder!("11010110", Opcode::Sub, |mmu, pc, opcode| {
            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg8(Register::A, AddressingMode::Direct)),
                rhs: Some(Operand::Imm8(mmu.read(pc.wrapping_add(1)), AddressingMode::Direct)),
                length: 2,
                cycles: (8, None),
            })
        }));

        // and A, imm8
        lut.push(define_decoder!("11100110", Opcode::And, |mmu, pc, opcode| {
            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg8(Register::A, AddressingMode::Direct)),
                rhs: Some(Operand::Imm8(mmu.read(pc.wrapping_add(1)), AddressingMode::Direct)),
                length: 2,
                cycles: (8, None),
            })
        }));

        // or A, imm8
        lut.push(define_decoder!("11110110", Opcode::Or, |mmu, pc, opcode| {
            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg8(Register::A, AddressingMode::Direct)),
                rhs: Some(Operand::Imm8(mmu.read(pc.wrapping_add(1)), AddressingMode::Direct)),
                length: 2,
                cycles: (8, None),
            })
        }));

        // reti
        lut.push(define_decoder!("11011001", Opcode::Reti, |_, _, opcode| {
            Ok(Instruction {
                opcode,
                lhs: None,
                rhs: None,
                length: 1,
                cycles: (16, None),
            })
        }));

        // jp hl
        lut.push(define_decoder!("11101001", Opcode::Jp, |_, _, opcode| {
            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Conditional(Condition::None)),
                rhs: Some(Operand::Reg16(Register::HL, AddressingMode::Direct)),
                length: 1,
                cycles: (4, None),
            })
        }));

        // di
        lut.push(define_decoder!("11110011", Opcode::Di, |_, _, opcode| {
            Ok(Instruction {
                opcode,
                lhs: None,
                rhs: None,
                length: 1,
                cycles: (4, None),
            })
        }));

        // ei
        lut.push(define_decoder!("11111011", Opcode::Ei, |_, _, opcode| {
            Ok(Instruction {
                opcode,
                lhs: None,
                rhs: None,
                length: 1,
                cycles: (4, None),
            })
        }));

        // jr cond, imm8
        lut.push(define_decoder!("00xxx000", Opcode::Jr, |mmu, pc, _| {
            let opcode_byte = mmu.read(pc);
            let condition = Sm83::lookup_condition_3bits((opcode_byte & 0b0011_1000) >> 3)?;
            let offset = mmu.read(pc.wrapping_add(1)) as i8;
            let cycles = if condition != Condition::None { (12, Some(8)) } else { (12, None) };

            Ok(Instruction {
                opcode: Opcode::Jr,
                lhs: Some(Operand::Conditional(condition)),
                rhs: Some(Operand::Offset(offset)),
                length: 2,
                cycles,
            })
        }));

        // ld r16, imm16
        lut.push(define_decoder!("00xx0001", Opcode::Ld, |mmu, pc, _| {
            let opcode_byte = mmu.read(pc);
            let destination = (opcode_byte & 0b0011_0000) >> 4;

            Ok(Instruction {
                opcode: Opcode::Ld,
                lhs: Some(Operand::Reg16(Sm83::lookup_register_16(destination)?, AddressingMode::Direct)),
                rhs: Some(Operand::Imm16(mmu.read16(pc.wrapping_add(1)), AddressingMode::Direct)),
                length: 3,
                cycles: (12, None),
            })
        }));

        // ld (r16), A
        lut.push(define_decoder!("00xx0010", Opcode::Ld, |mmu, pc, _| {
            let opcode_byte = mmu.read(pc);

            if opcode_byte == 0x22 || opcode_byte == 0x32 {
                return Ok(Instruction {
                    opcode: Opcode::Ld,
                    lhs: Some(Operand::Reg16(
                        Register::HL,
                        AddressingMode::Indirect
                            | if opcode_byte == 0x22 {
                                AddressingMode::Increment
                            } else {
                                AddressingMode::Decrement
                            },
                    )),
                    rhs: Some(Operand::Reg8(Register::A, AddressingMode::Direct)),
                    length: 1,
                    cycles: (8, None),
                });
            }

            let destination = (opcode_byte & 0b0011_0000) >> 4;
            Ok(Instruction {
                opcode: Opcode::Ld,
                lhs: Some(Operand::Reg16(Sm83::lookup_register_16(destination)?, AddressingMode::Indirect)),
                rhs: Some(Operand::Reg8(Register::A, AddressingMode::Direct)),
                length: 1,
                cycles: (8, None),
            })
        }));

        // add HL, r16
        lut.push(define_decoder!("00xx1001", Opcode::Add, |mmu, pc, _| {
            let opcode_byte = mmu.read(pc);
            let source = (opcode_byte & 0b0011_0000) >> 4;

            Ok(Instruction {
                opcode: Opcode::Add,
                lhs: Some(Operand::Reg16(Register::HL, AddressingMode::Direct)),
                rhs: Some(Operand::Reg16(Sm83::lookup_register_16(source)?, AddressingMode::Direct)),
                length: 1,
                cycles: (8, None),
            })
        }));

        // ld A, (r16)
        lut.push(define_decoder!("00xx1010", Opcode::Ld, |mmu, pc, _| {
            let opcode_byte = mmu.read(pc);
            if opcode_byte == 0x2a || opcode_byte == 0x3a {
                return Ok(Instruction {
                    opcode: Opcode::Ld,
                    lhs: Some(Operand::Reg8(Register::A, AddressingMode::Direct)),
                    rhs: Some(Operand::Reg16(
                        Register::HL,
                        AddressingMode::Indirect
                            | if opcode_byte == 0x2a {
                                AddressingMode::Increment
                            } else {
                                AddressingMode::Decrement
                            },
                    )),
                    length: 1,
                    cycles: (8, None),
                });
            }

            let source = (opcode_byte & 0b0011_0000) >> 4;
            Ok(Instruction {
                opcode: Opcode::Ld,
                lhs: Some(Operand::Reg8(Register::A, AddressingMode::Direct)),
                rhs: Some(Operand::Reg16(Sm83::lookup_register_16(source)?, AddressingMode::Indirect)),
                length: 1,
                cycles: (8, None),
            })
        }));

        // ld r8, imm8 / ld (HL), imm8
        lut.push(define_decoder!("00xxx110", Opcode::Ld, |mmu, pc, _| {
            let opcode_byte = mmu.read(pc);
            let destination = (opcode_byte & 0b0011_1000) >> 3;
            let (lhs, cycles) = Sm83::decode_8bit_operand(destination, 8, 12)?;

            Ok(Instruction {
                opcode: Opcode::Ld,
                lhs: Some(lhs),
                rhs: Some(Operand::Imm8(mmu.read(pc.wrapping_add(1)), AddressingMode::Direct)),
                length: 2,
                cycles: (cycles, None),
            })
        }));

        // inc r16
        lut.push(define_decoder!("00xx0011", Opcode::Inc, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc);
            let destination = (opcode_byte & 0b0011_0000) >> 4;

            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg16(Sm83::lookup_register_16(destination)?, AddressingMode::Direct)),
                rhs: None,
                length: 1,
                cycles: (8, None),
            })
        }));

        // inc r8 / inc (HL)
        lut.push(define_decoder!("00xxx100", Opcode::Inc, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc);
            let destination = (opcode_byte & 0b0011_1000) >> 3;
            let (lhs, cycles) = Sm83::decode_8bit_operand(destination, 4, 12)?;

            Ok(Instruction {
                opcode,
                lhs: Some(lhs),
                rhs: None,
                length: 1,
                cycles: (cycles, None),
            })
        }));

        // dec r8 / dec (HL)
        lut.push(define_decoder!("00xxx101", Opcode::Dec, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc);
            let destination = (opcode_byte & 0b0011_1000) >> 3;
            let (lhs, cycles) = Sm83::decode_8bit_operand(destination, 4, 12)?;

            Ok(Instruction {
                opcode,
                lhs: Some(lhs),
                rhs: None,
                length: 1,
                cycles: (cycles, None),
            })
        }));

        // dec r16
        lut.push(define_decoder!("00xx1011", Opcode::Dec, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc);
            let destination = (opcode_byte & 0b0011_0000) >> 4;

            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg16(Sm83::lookup_register_16(destination)?, AddressingMode::Direct)),
                rhs: None,
                length: 1,
                cycles: (8, None),
            })
        }));

        // ld r8, r8 / ld r8, (HL) / ld (HL), r8
        lut.push(define_decoder!("01xxxxxx", Opcode::Ld, |mmu, pc, _| {
            let opcode_byte = mmu.read(pc);

            let destination = (opcode_byte & 0b0011_1000) >> 3;
            let source = opcode_byte & 0b0000_0111;

            let (lhs, cycles1) = Sm83::decode_8bit_operand(destination, 4, 8)?;
            let (rhs, cycles2) = Sm83::decode_8bit_operand(source, 4, 8)?;

            Ok(Instruction {
                opcode: Opcode::Ld,
                lhs: Some(lhs),
                rhs: Some(rhs),
                length: 1,
                cycles: (std::cmp::max(cycles1, cycles2), None),
            })
        }));

        // pop r16
        lut.push(define_decoder!("11xx0001", Opcode::Pop, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc);
            let destination = (opcode_byte & 0b0011_0000) >> 4;
            let mut lhs = Sm83::lookup_register_16(destination)?;

            // The register pattern for SP is 11,
            // but it's actually AF in the case of pop instruction
            if lhs == Register::SP {
                lhs = Register::AF;
            }

            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg16(lhs, AddressingMode::Direct)),
                rhs: None,
                length: 1,
                cycles: (12, None),
            })
        }));

        // ret cond / ret
        lut.push(define_decoder!("110xx00x", Opcode::Ret, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc);

            if (opcode_byte & 0b0000_0001) != 0 {
                return Ok(Instruction {
                    opcode,
                    lhs: Some(Operand::Conditional(Condition::None)),
                    rhs: None,
                    length: 1,
                    cycles: (16, None),
                });
            }

            let condition = Sm83::lookup_condition_2bits((opcode_byte & 0b0001_1000) >> 3)?;
            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Conditional(condition)),
                rhs: None,
                length: 1,
                cycles: (20, Some(8)),
            })
        }));

        // jp cond, imm16 / jp imm16
        lut.push(define_decoder!("110xx01x", Opcode::Jp, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc);

            let condition = if (opcode_byte & 0b0000_0001) == 0 {
                Sm83::lookup_condition_2bits((opcode_byte & 0b0001_1000) >> 3)?
            } else {
                Condition::None
            };

            let cycles = if condition != Condition::None { (16, Some(12)) } else { (16, None) };

            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Conditional(condition)),
                rhs: Some(Operand::Imm16(mmu.read16(pc + 1), AddressingMode::Direct)),
                length: 3,
                cycles,
            })
        }));

        // push r16
        lut.push(define_decoder!("11xx0101", Opcode::Push, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc);
            let source = (opcode_byte & 0b0011_0000) >> 4;
            let mut lhs = Sm83::lookup_register_16(source)?;

            // The register pattern for SP is 11,
            // but it's actually AF in the case of push instruction
            if lhs == Register::SP {
                lhs = Register::AF;
            }

            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg16(lhs, AddressingMode::Direct)),
                rhs: None,
                length: 1,
                cycles: (16, None),
            })
        }));

        // call cond, imm16 / call imm16
        lut.push(define_decoder!("110xx10x", Opcode::Call, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc);

            let condition = if (opcode_byte & 0b0000_0001) == 0 {
                Sm83::lookup_condition_2bits((opcode_byte & 0b0001_1000) >> 3)?
            } else {
                Condition::None
            };

            let cycles = if condition != Condition::None { (24, Some(12)) } else { (24, None) };

            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Conditional(condition)),
                rhs: Some(Operand::Imm16(mmu.read16(pc + 1), AddressingMode::Direct)),
                length: 3,
                cycles,
            })
        }));

        // add a, r8 / add a, (HL)
        lut.push(define_decoder!("10000xxx", Opcode::Add, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc);
            let source = opcode_byte & 0b0000_0111;
            let (rhs, cycles) = Sm83::decode_8bit_operand(source, 4, 8)?;

            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg8(Register::A, AddressingMode::Direct)),
                rhs: Some(rhs),
                length: 1,
                cycles: (cycles, None),
            })
        }));

        // adc a, r8 / adc a, (HL)
        lut.push(define_decoder!("10001xxx", Opcode::Adc, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc);
            let source = opcode_byte & 0b0000_0111;
            let (rhs, cycles) = Sm83::decode_8bit_operand(source, 4, 8)?;

            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg8(Register::A, AddressingMode::Direct)),
                rhs: Some(rhs),
                length: 1,
                cycles: (cycles, None),
            })
        }));

        // sub a, r8 / sub a, (HL)
        lut.push(define_decoder!("10010xxx", Opcode::Sub, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc);
            let source = opcode_byte & 0b0000_0111;
            let (rhs, cycles) = Sm83::decode_8bit_operand(source, 4, 8)?;

            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg8(Register::A, AddressingMode::Direct)),
                rhs: Some(rhs),
                length: 1,
                cycles: (cycles, None),
            })
        }));

        // sbc a, r8 / sbc a, (HL)
        lut.push(define_decoder!("10011xxx", Opcode::Sbc, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc);
            let source = opcode_byte & 0b0000_0111;
            let (rhs, cycles) = Sm83::decode_8bit_operand(source, 4, 8)?;

            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg8(Register::A, AddressingMode::Direct)),
                rhs: Some(rhs),
                length: 1,
                cycles: (cycles, None),
            })
        }));

        // and r8, r8 / and r8, (HL)
        lut.push(define_decoder!("10100xxx", Opcode::And, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc);
            let source = opcode_byte & 0b0000_0111;
            let (rhs, cycles) = Sm83::decode_8bit_operand(source, 4, 8)?;

            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg8(Register::A, AddressingMode::Direct)),
                rhs: Some(rhs),
                length: 1,
                cycles: (cycles, None),
            })
        }));

        // xor r8, r8 / xor r8, (HL)
        lut.push(define_decoder!("10101xxx", Opcode::Xor, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc);
            let source = opcode_byte & 0b0000_0111;
            let (rhs, cycles) = Sm83::decode_8bit_operand(source, 4, 8)?;

            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg8(Register::A, AddressingMode::Direct)),
                rhs: Some(rhs),
                length: 1,
                cycles: (cycles, None),
            })
        }));

        // or r8, r8 / or r8, (HL)
        lut.push(define_decoder!("10110xxx", Opcode::Or, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc);
            let source = opcode_byte & 0b0000_0111;
            let (rhs, cycles) = Sm83::decode_8bit_operand(source, 4, 8)?;

            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg8(Register::A, AddressingMode::Direct)),
                rhs: Some(rhs),
                length: 1,
                cycles: (cycles, None),
            })
        }));

        // cp a, r8 / cp a, (HL)
        lut.push(define_decoder!("10111xxx", Opcode::Cp, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc);
            let source = opcode_byte & 0b0000_0111;
            let (rhs, cycles) = Sm83::decode_8bit_operand(source, 4, 8)?;

            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Reg8(Register::A, AddressingMode::Direct)),
                rhs: Some(rhs),
                length: 1,
                cycles: (cycles, None),
            })
        }));

        // rst n
        lut.push(define_decoder!("11xxx111", Opcode::Rst, |mmu, pc, opcode| {
            let n = match (mmu.read(pc) & 0b0011_1000) >> 3 {
                0b000 => 0x00,
                0b001 => 0x08,
                0b010 => 0x10,
                0b011 => 0x18,
                0b100 => 0x20,
                0b101 => 0x28,
                0b110 => 0x30,
                0b111 => 0x38,
                _ => unreachable!(),
            };

            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Imm8(n, AddressingMode::Direct)),
                rhs: None,
                length: 1,
                cycles: (16, None),
            })
        }));
    }

    fn propagate_decoders_prefixed(lut: &mut Vec<(String, Opcode, FDecode)>) {
        // rlc r8 / rlc (HL)
        lut.push(define_decoder!("00000xxx", Opcode::Rlc, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc.wrapping_add(1));
            let source = opcode_byte & 0b0000_0111;
            let (rhs, cycles) = Sm83::decode_8bit_operand(source, 8, 16)?;

            Ok(Instruction {
                opcode,
                lhs: Some(rhs),
                rhs: None,
                length: 2,
                cycles: (cycles, None),
            })
        }));

        // rrc r8 / rrc (HL)
        lut.push(define_decoder!("00001xxx", Opcode::Rrc, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc.wrapping_add(1));
            let source = opcode_byte & 0b0000_0111;
            let (rhs, cycles) = Sm83::decode_8bit_operand(source, 8, 16)?;

            Ok(Instruction {
                opcode,
                lhs: Some(rhs),
                rhs: None,
                length: 2,
                cycles: (cycles, None),
            })
        }));

        // rl r8 / rl (HL)
        lut.push(define_decoder!("00010xxx", Opcode::Rl, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc.wrapping_add(1));
            let source = opcode_byte & 0b0000_0111;
            let (rhs, cycles) = Sm83::decode_8bit_operand(source, 8, 16)?;

            Ok(Instruction {
                opcode,
                lhs: Some(rhs),
                rhs: None,
                length: 2,
                cycles: (cycles, None),
            })
        }));

        // rr r8 / rr (HL)
        lut.push(define_decoder!("00011xxx", Opcode::Rr, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc.wrapping_add(1));
            let source = opcode_byte & 0b0000_0111;
            let (rhs, cycles) = Sm83::decode_8bit_operand(source, 8, 16)?;

            Ok(Instruction {
                opcode,
                lhs: Some(rhs),
                rhs: None,
                length: 2,
                cycles: (cycles, None),
            })
        }));

        // sla r8 / sla (HL)
        lut.push(define_decoder!("00100xxx", Opcode::Sla, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc.wrapping_add(1));
            let source = opcode_byte & 0b0000_0111;
            let (rhs, cycles) = Sm83::decode_8bit_operand(source, 8, 16)?;

            Ok(Instruction {
                opcode,
                lhs: Some(rhs),
                rhs: None,
                length: 2,
                cycles: (cycles, None),
            })
        }));

        // sra r8 / sra (HL)
        lut.push(define_decoder!("00101xxx", Opcode::Sra, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc.wrapping_add(1));
            let source = opcode_byte & 0b0000_0111;
            let (rhs, cycles) = Sm83::decode_8bit_operand(source, 8, 16)?;

            Ok(Instruction {
                opcode,
                lhs: Some(rhs),
                rhs: None,
                length: 2,
                cycles: (cycles, None),
            })
        }));

        // swap r8 / swap (HL)
        lut.push(define_decoder!("00110xxx", Opcode::Swap, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc.wrapping_add(1));
            let source = opcode_byte & 0b0000_0111;
            let (rhs, cycles) = Sm83::decode_8bit_operand(source, 8, 16)?;

            Ok(Instruction {
                opcode,
                lhs: Some(rhs),
                rhs: None,
                length: 2,
                cycles: (cycles, None),
            })
        }));

        // srl r8 / srl (HL)
        lut.push(define_decoder!("00111xxx", Opcode::Srl, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc.wrapping_add(1));
            let source = opcode_byte & 0b0000_0111;
            let (rhs, cycles) = Sm83::decode_8bit_operand(source, 8, 16)?;

            Ok(Instruction {
                opcode,
                lhs: Some(rhs),
                rhs: None,
                length: 2,
                cycles: (cycles, None),
            })
        }));

        // bit n, r8 / bit n, (HL)
        lut.push(define_decoder!("01xxxxxx", Opcode::Bit, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc.wrapping_add(1));
            let bit = (opcode_byte & 0b0011_1000) >> 3;
            let source = opcode_byte & 0b0000_0111;
            let (rhs, cycles) = Sm83::decode_8bit_operand(source, 8, 12)?;

            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Bit(bit)),
                rhs: Some(rhs),
                length: 2,
                cycles: (cycles, None),
            })
        }));

        // res n, r8 / res n, (HL)
        lut.push(define_decoder!("10xxxxxx", Opcode::Res, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc.wrapping_add(1));
            let bit = (opcode_byte & 0b0011_1000) >> 3;
            let source = opcode_byte & 0b0000_0111;
            let (rhs, cycles) = Sm83::decode_8bit_operand(source, 8, 16)?;

            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Bit(bit)),
                rhs: Some(rhs),
                length: 2,
                cycles: (cycles, None),
            })
        }));

        // set n, r8 / set n, (HL)
        lut.push(define_decoder!("11xxxxxx", Opcode::Set, |mmu, pc, opcode| {
            let opcode_byte = mmu.read(pc.wrapping_add(1));
            let bit = (opcode_byte & 0b0011_1000) >> 3;
            let source = opcode_byte & 0b0000_0111;
            let (rhs, cycles) = Sm83::decode_8bit_operand(source, 8, 16)?;

            Ok(Instruction {
                opcode,
                lhs: Some(Operand::Bit(bit)),
                rhs: Some(rhs),
                length: 2,
                cycles: (cycles, None),
            })
        }));
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut output = format!("{:?}", self.opcode).to_lowercase();

        let mut ignore_destination = false;
        if let Some(destination) = &self.lhs {
            match destination {
                Operand::Conditional(cond) if *cond == Condition::None => ignore_destination = true,
                _ => output.push_str(&format!(" {}", destination)),
            };
        }

        if let Some(source) = &self.rhs {
            if !ignore_destination {
                output.push_str(&format!(", {}", source));
            } else {
                output.push_str(&format!(" {}", source));
            }
        }

        write!(f, "{}", output)
    }
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let output = match self {
            Register::A => "a",
            Register::B => "b",
            Register::C => "c",
            Register::D => "d",
            Register::E => "e",
            Register::H => "h",
            Register::L => "l",
            Register::F => "f",
            Register::AF => "af",
            Register::BC => "bc",
            Register::DE => "de",
            Register::HL => "hl",
            Register::SP => "sp",
            Register::PC => "pc",
        };

        write!(f, "{}", output)
    }
}

impl std::fmt::Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let output = match self {
            Operand::Reg8(reg, mode) => {
                if mode.contains(AddressingMode::Indirect) {
                    format!("({})", reg)
                } else {
                    format!("{}", reg)
                }
            }
            Operand::Reg16(reg, mode) => {
                if mode.contains(AddressingMode::Indirect) {
                    if mode.contains(AddressingMode::Increment) {
                        format!("({}+)", reg)
                    } else if mode.contains(AddressingMode::Decrement) {
                        format!("({}-)", reg)
                    } else {
                        format!("({})", reg)
                    }
                } else {
                    format!("{}", reg)
                }
            }
            Operand::Imm8(value, mode) => {
                if mode.contains(AddressingMode::Indirect) {
                    format!("({:#02x})", value)
                } else {
                    format!("{:#02x}", value)
                }
            }
            Operand::Imm16(value, mode) => {
                if mode.contains(AddressingMode::Indirect) {
                    format!("({:#04x})", value)
                } else {
                    format!("{:#04x}", value)
                }
            }
            Operand::Conditional(cond) => {
                if *cond != Condition::None {
                    format!("{}", cond)
                } else {
                    String::new()
                }
            }
            Operand::Offset(value) => {
                if *value > 0 {
                    format!("+{}", value)
                } else {
                    format!("{}", value)
                }
            }
            Operand::Bit(value) => format!("{}", value),
            Operand::DisplacedReg16(reg, value, mode) => {
                if mode.contains(AddressingMode::Indirect) {
                    format!("({}+{:#02x})", reg, value)
                } else {
                    format!("{}+{:#02x}", reg, value)
                }
            }
        };

        write!(f, "{}", output)
    }
}

impl std::fmt::Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let output = match self {
            Condition::None => "",
            Condition::NZ => "nz",
            Condition::Z => "z",
            Condition::NC => "nc",
            Condition::C => "c",
        };

        write!(f, "{}", output)
    }
}
