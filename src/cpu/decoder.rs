//! Décodeur d'instructions NEC V60

use super::instructions::*;
use anyhow::Result;

/// Décode une instruction brute en instruction structurée
pub fn decode_instruction(opcode: u32, address: u32) -> Result<DecodedInstruction> {
    // Version simplifiée pour commencer
    let instruction = Instruction::Unknown { opcode };
    let size = 4;
    Ok(DecodedInstruction::new(instruction, address, size))
}
