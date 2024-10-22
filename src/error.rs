use crate::lr35902::sm83::{Instruction, Operand};
use snafu::prelude::*;

#[derive(Debug, Snafu)]
pub enum AyyError {
    #[snafu(display("Failed to decode instruction ({:02x}) at address: ${:04x}", opcode, address))]
    DecoderFailure { opcode: u8, address: u16 },
    #[snafu(display("Illegal opcode: {:02x}", opcode))]
    IllegalOpcode { opcode: u8 },
    #[snafu(display("Unknown condition bits: {:08b}", data))]
    UnknownConditionBits { data: u8 },
    #[snafu(display("Unknown register bits: {:08b}", data))]
    UnknownRegisterBits { data: u8 },
    #[snafu(display("Unimplemented instruction: {}", instruction))]
    UnimplementedInstruction { instruction: String, cpu: String },
    #[snafu(display("Invalid instruction handler implementation: {}", instruction))]
    InvalidHandler { instruction: Instruction },
    #[snafu(display("Unresolved target: {:?}", target))]
    UnresolvedTarget { target: Operand },
    #[snafu(display("Unknown interrupt vector: {:08b}", vector))]
    UnknownIrqVector { vector: u8 },
    #[snafu(display("Write to read-only memory at address: {:04x} with data: {:02x}", address, data))]
    WriteToReadOnlyMemory { address: u16, data: u8 },
    #[snafu(display("Write to disabled external RAM at address: {:04x} with data: {:02x}", address, data))]
    WriteToDisabledExternalRam { address: u16, data: u8 },
    #[snafu(display("Out of bounds memory access at address: {:04x}", address))]
    OutOfBoundsMemoryAccess { address: u16 },
}
