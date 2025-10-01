//! Décodeur d'instructions NEC V60

use super::instructions::*;
use super::instruction_formats::*;
use super::registers::*;
use anyhow::{Result, anyhow};

/// Décode une instruction brute en instruction structurée
pub fn decode_instruction(opcode: u32, address: u32) -> Result<DecodedInstruction> {
    // Version simplifiée pour commencer
    let instruction = Instruction::Unknown { opcode };
    let size = 4;
    Ok(DecodedInstruction::new(instruction, address, size))
}