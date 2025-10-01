//! Décodeur d'instructions NEC V60

use super::instructions::*;
use anyhow::{Result, anyhow};

/// Décode une instruction brute en instruction structurée
pub fn decode_instruction(opcode: u32, address: u32) -> Result<DecodedInstruction> {
    // Le NEC V60 utilise des instructions de longueur variable
    // Cette implémentation est simplifiée pour l'exemple
    
    let instruction = match opcode >> 24 {
        // Instructions arithmétiques (format simplifié)
        0x00 => decode_arithmetic(opcode)?,
        0x01 => decode_logical(opcode)?,
        0x02 => decode_shift(opcode)?,
        0x03 => decode_move(opcode)?,
        0x04 => decode_memory(opcode)?,
        0x05 => decode_branch(opcode)?,
        0x06 => decode_compare(opcode)?,
        0x07 => decode_system(opcode)?,
        0x08 => decode_float(opcode)?,
        
        // Instructions spéciales
        0xFF => match opcode & 0xFF {
            0x00 => Instruction::Nop,
            0x01 => Instruction::Halt,
            0x02 => Instruction::Return,
            0x03 => Instruction::InterruptReturn,
            _ => Instruction::Unknown { opcode },
        },
        
        _ => Instruction::Unknown { opcode },
    };
    
    // Estimer la taille de l'instruction (simplifié)
    let size = estimate_instruction_size(&instruction);
    
    Ok(DecodedInstruction::new(instruction, address, size))
}

/// Décode les instructions arithmétiques
fn decode_arithmetic(opcode: u32) -> Result<Instruction> {
    let sub_op = (opcode >> 20) & 0xF;
    let dest = extract_register((opcode >> 16) & 0xF)?;
    let src1 = extract_register((opcode >> 12) & 0xF)?;
    let src2 = extract_operand(opcode & 0xFFF)?;
    
    match sub_op {
        0x0 => Ok(Instruction::Add { dest, src1, src2 }),
        0x1 => Ok(Instruction::Sub { dest, src1, src2 }),
        0x2 => Ok(Instruction::Mul { dest, src1, src2 }),
        0x3 => Ok(Instruction::Div { dest, src1, src2 }),
        _ => Err(anyhow!("Instruction arithmétique inconnue: {:#x}", sub_op)),
    }
}

/// Décode les instructions logiques
fn decode_logical(opcode: u32) -> Result<Instruction> {
    let sub_op = (opcode >> 20) & 0xF;
    let dest = extract_register((opcode >> 16) & 0xF)?;
    let src1 = extract_register((opcode >> 12) & 0xF)?;
    let src2 = extract_operand(opcode & 0xFFF)?;
    
    match sub_op {
        0x0 => Ok(Instruction::And { dest, src1, src2 }),
        0x1 => Ok(Instruction::Or { dest, src1, src2 }),
        0x2 => Ok(Instruction::Xor { dest, src1, src2 }),
        0x3 => Ok(Instruction::Not { dest, src: src1 }),
        _ => Err(anyhow!("Instruction logique inconnue: {:#x}", sub_op)),
    }
}

/// Décode les instructions de décalage
fn decode_shift(opcode: u32) -> Result<Instruction> {
    let sub_op = (opcode >> 20) & 0xF;
    let dest = extract_register((opcode >> 16) & 0xF)?;
    let src = extract_register((opcode >> 12) & 0xF)?;
    let shift = extract_operand(opcode & 0xFFF)?;
    
    match sub_op {
        0x0 => Ok(Instruction::Shl { dest, src, shift }),
        0x1 => Ok(Instruction::Shr { dest, src, shift }),
        _ => Err(anyhow!("Instruction de décalage inconnue: {:#x}", sub_op)),
    }
}

/// Décode les instructions de mouvement
fn decode_move(opcode: u32) -> Result<Instruction> {
    let dest = extract_register((opcode >> 16) & 0xF)?;
    let src = extract_operand(opcode & 0xFFFF)?;
    
    Ok(Instruction::Mov { dest, src })
}

/// Décode les instructions mémoire
fn decode_memory(opcode: u32) -> Result<Instruction> {
    let sub_op = (opcode >> 20) & 0xF;
    let reg = extract_register((opcode >> 16) & 0xF)?;
    let address = extract_operand(opcode & 0xFFFF)?;
    let size = extract_data_size((opcode >> 18) & 0x3)?;
    
    match sub_op {
        0x0 => Ok(Instruction::Load { dest: reg, address, size }),
        0x1 => Ok(Instruction::Store { src: reg, address, size }),
        _ => Err(anyhow!("Instruction mémoire inconnue: {:#x}", sub_op)),
    }
}

/// Décode les instructions de branchement
fn decode_branch(opcode: u32) -> Result<Instruction> {
    let sub_op = (opcode >> 20) & 0xF;
    let target = extract_operand(opcode & 0xFFFFF)?;
    
    match sub_op {
        0x0 => Ok(Instruction::Jump { target }),
        0x1 => Ok(Instruction::Call { target }),
        0x2..=0xF => {
            let condition = extract_condition_code(sub_op - 2)?;
            Ok(Instruction::JumpConditional { condition, target })
        },
        _ => Err(anyhow!("Instruction de branchement inconnue: {:#x}", sub_op)),
    }
}

/// Décode les instructions de comparaison
fn decode_compare(opcode: u32) -> Result<Instruction> {
    let sub_op = (opcode >> 20) & 0xF;
    let src1 = extract_register((opcode >> 16) & 0xF)?;
    let src2 = extract_operand(opcode & 0xFFFF)?;
    
    match sub_op {
        0x0 => Ok(Instruction::Compare { src1, src2 }),
        0x1 => Ok(Instruction::Test { src1, src2 }),
        _ => Err(anyhow!("Instruction de comparaison inconnue: {:#x}", sub_op)),
    }
}

/// Décode les instructions système
fn decode_system(opcode: u32) -> Result<Instruction> {
    match opcode & 0xFFFF {
        0x0000 => Ok(Instruction::Nop),
        0x0001 => Ok(Instruction::Halt),
        0x0002 => Ok(Instruction::Return),
        0x0003 => Ok(Instruction::InterruptReturn),
        _ => Ok(Instruction::Unknown { opcode }),
    }
}

/// Décode les instructions flottantes
fn decode_float(opcode: u32) -> Result<Instruction> {
    let sub_op = (opcode >> 20) & 0xF;
    let dest = extract_register((opcode >> 16) & 0xF)?;
    let src1 = extract_register((opcode >> 12) & 0xF)?;
    let src2 = extract_operand(opcode & 0xFFF)?;
    
    match sub_op {
        0x0 => Ok(Instruction::FloatAdd { dest, src1, src2 }),
        0x1 => Ok(Instruction::FloatMul { dest, src1, src2 }),
        _ => Err(anyhow!("Instruction flottante inconnue: {:#x}", sub_op)),
    }
}

/// Extrait un registre depuis un champ d'instruction
fn extract_register(field: u32) -> Result<Operand> {
    if field < 32 {
        Ok(Operand::Register(field as usize))
    } else {
        Err(anyhow!("Numéro de registre invalide: {}", field))
    }
}

/// Extrait un opérande depuis un champ d'instruction
fn extract_operand(field: u32) -> Result<Operand> {
    // Format simplifié pour l'exemple
    let mode = (field >> 28) & 0xF;
    let value = field & 0x0FFFFFFF;
    
    match mode {
        0x0 => Ok(Operand::Register(value as usize)),
        0x1 => Ok(Operand::Immediate(value)),
        0x2 => Ok(Operand::Direct(value)),
        0x3 => Ok(Operand::Indirect(value as usize)),
        0x4 => Ok(Operand::IndirectOffset(
            (value >> 16) as usize,
            (value & 0xFFFF) as i16 as i32,
        )),
        0x5 => Ok(Operand::PcRelative(value as i32)),
        _ => Ok(Operand::Immediate(value)),
    }
}

/// Extrait une taille de données
fn extract_data_size(field: u32) -> Result<DataSize> {
    match field & 0x3 {
        0 => Ok(DataSize::Byte),
        1 => Ok(DataSize::Word),
        2 => Ok(DataSize::DWord),
        _ => Err(anyhow!("Taille de données invalide: {}", field)),
    }
}

/// Extrait un code de condition
fn extract_condition_code(field: u32) -> Result<super::registers::ConditionCode> {
    use super::registers::ConditionCode;
    
    match field {
        0 => Ok(ConditionCode::Always),
        1 => Ok(ConditionCode::Never),
        2 => Ok(ConditionCode::Equal),
        3 => Ok(ConditionCode::NotEqual),
        4 => Ok(ConditionCode::Carry),
        5 => Ok(ConditionCode::NotCarry),
        6 => Ok(ConditionCode::Negative),
        7 => Ok(ConditionCode::Positive),
        8 => Ok(ConditionCode::Overflow),
        9 => Ok(ConditionCode::NotOverflow),
        _ => Err(anyhow!("Code de condition invalide: {}", field)),
    }
}

/// Estime la taille d'une instruction en octets
fn estimate_instruction_size(instruction: &Instruction) -> u32 {
    match instruction {
        // Instructions simples - 4 octets
        Instruction::Nop | 
        Instruction::Halt |
        Instruction::Return |
        Instruction::InterruptReturn |
        Instruction::ReturnFromInterrupt |
        Instruction::EnableInterrupts |
        Instruction::DisableInterrupts |
        Instruction::InvalidateTLB |
        Instruction::FlushCache => 4,
        
        // Instructions avec opérandes - 4 à 8 octets
        Instruction::Add { .. } |
        Instruction::Sub { .. } |
        Instruction::Mul { .. } |
        Instruction::Div { .. } |
        Instruction::And { .. } |
        Instruction::Or { .. } |
        Instruction::Xor { .. } |
        Instruction::Not { .. } |
        Instruction::Shl { .. } |
        Instruction::Shr { .. } |
        Instruction::Mov { .. } |
        Instruction::Compare { .. } |
        Instruction::Test { .. } |
        Instruction::FloatAdd { .. } |
        Instruction::FloatMul { .. } |
        Instruction::FloatSub { .. } |
        Instruction::FloatDiv { .. } |
        Instruction::FloatCompare { .. } |
        Instruction::RotateLeft { .. } |
        Instruction::RotateRight { .. } |
        Instruction::BitTest { .. } |
        Instruction::BitSet { .. } |
        Instruction::BitClear { .. } |
        Instruction::BitScan { .. } |
        Instruction::BcdAdd { .. } |
        Instruction::BcdSub { .. } |
        Instruction::TestAndSet { .. } |
        Instruction::CompareAndSwap { .. } => 8,
        
        // Instructions mémoire - taille variable
        Instruction::Load { .. } |
        Instruction::Store { .. } |
        Instruction::LoadControlRegister { .. } |
        Instruction::StoreControlRegister { .. } => 8,
        
        // Instructions de pile - 4 à 8 octets
        Instruction::Push { .. } |
        Instruction::Pop { .. } => 6,
        Instruction::PushMultiple { registers } => 4 + (registers.len() as u32 / 8) * 4,
        Instruction::PopMultiple { registers } => 4 + (registers.len() as u32 / 8) * 4,
        
        // Instructions de chaîne - taille fixe
        Instruction::StringMove { .. } |
        Instruction::StringCompare { .. } |
        Instruction::StringScan { .. } => 8,
        
        // Instructions de branchement - taille variable
        Instruction::Jump { .. } |
        Instruction::JumpConditional { .. } |
        Instruction::Call { .. } => 8,
        
        // Instructions d'interruption
        Instruction::SoftwareInterrupt { .. } => 6,
        
        // Instruction inconnue - 4 octets par défaut
        Instruction::Unknown { .. } => 4,
    }
}