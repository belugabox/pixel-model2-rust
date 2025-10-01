//! Formats d'instructions réels du NEC V60
//! 
//! Le NEC V60 utilise plusieurs formats d'instructions avec des longueurs variables

use super::instructions::*;
use super::registers::ConditionCode;
use anyhow::{Result, anyhow};

/// Formats d'instructions NEC V60
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstructionFormat {
    /// Format 1: Instruction basique (16 bits)
    /// +------+------+------+------+
    /// |opcode|  r2  |  r1  |mode  |
    /// +------+------+------+------+
    Format1 {
        opcode: u8,
        r2: u8,
        r1: u8,
        mode: u8,
    },
    
    /// Format 2: Instruction avec immédiat (32 bits)
    /// +------+------+------+------+------+------+------+------+
    /// |opcode|  r2  |  r1  |mode  |       immediate           |
    /// +------+------+------+------+------+------+------+------+
    Format2 {
        opcode: u8,
        r2: u8,
        r1: u8,
        mode: u8,
        immediate: u16,
    },
    
    /// Format 3: Instruction avec déplacement (48 bits)
    /// +------+------+------+------+------+------+------+------+
    /// |opcode|  r2  |  r1  |mode  |       displacement        |
    /// +------+------+------+------+------+------+------+------+
    /// |              displacement (suite)                     |
    /// +------+------+------+------+------+------+------+------+
    Format3 {
        opcode: u8,
        r2: u8,
        r1: u8,
        mode: u8,
        displacement: u32,
    },
    
    /// Format 4: Branchement (32 bits)
    /// +------+------+------+------+------+------+------+------+
    /// |opcode| cond |       displacement                      |
    /// +------+------+------+------+------+------+------+------+
    Format4 {
        opcode: u8,
        condition: u8,
        displacement: i32,
    },
    
    /// Format 5: Instruction système (16 bits)
    /// +------+------+------+------+
    /// |opcode| func |      imm    |
    /// +------+------+------+------+
    Format5 {
        opcode: u8,
        function: u8,
        immediate: u8,
    },
}

/// Décodeur d'instructions amélioré pour le NEC V60
#[derive(Debug)]
pub struct V60InstructionDecoder {
    /// Cache des instructions décodées pour optimisation
    instruction_cache: std::collections::HashMap<u32, DecodedInstruction>,
}

impl V60InstructionDecoder {
    /// Crée un nouveau décodeur
    pub fn new() -> Self {
        Self {
            instruction_cache: std::collections::HashMap::new(),
        }
    }
    
    /// Décode une instruction à partir de données brutes
    pub fn decode(&mut self, data: &[u8], address: u32) -> Result<DecodedInstruction> {
        // Vérifier le cache d'abord
        if let Some(cached) = self.instruction_cache.get(&address) {
            return Ok(cached.clone());
        }
        
        if data.len() < 2 {
            return Err(anyhow!("Données insuffisantes pour décoder l'instruction"));
        }
        
        // Lire les premiers 16 bits pour déterminer le format
        let first_word = u16::from_le_bytes([data[0], data[1]]);
        let opcode = ((first_word >> 12) & 0xF) as u8;
        
        let format = self.determine_format(opcode, first_word, data)?;
        let size = self.calculate_instruction_size(&format);
        let instruction = self.decode_format(format)?;
        
        let decoded = DecodedInstruction::new(instruction, address, size);
        
        // Mettre en cache
        self.instruction_cache.insert(address, decoded.clone());
        
        Ok(decoded)
    }
    
    /// Détermine le format d'instruction basé sur l'opcode
    fn determine_format(&self, opcode: u8, first_word: u16, data: &[u8]) -> Result<InstructionFormat> {
        match opcode {
            // Instructions arithmétiques et logiques (Formats 1-3)
            0x0..=0x7 => {
                let r2 = ((first_word >> 8) & 0xF) as u8;
                let r1 = ((first_word >> 4) & 0xF) as u8;
                let mode = (first_word & 0xF) as u8;
                
                match mode {
                    // Mode immédiat - Format 2
                    0x1 | 0x2 => {
                        if data.len() < 4 {
                            return Err(anyhow!("Données insuffisantes pour Format 2"));
                        }
                        let immediate = u16::from_le_bytes([data[2], data[3]]);
                        Ok(InstructionFormat::Format2 { opcode, r2, r1, mode, immediate })
                    },
                    // Mode avec déplacement - Format 3
                    0x3 | 0x4 => {
                        if data.len() < 6 {
                            return Err(anyhow!("Données insuffisantes pour Format 3"));
                        }
                        let disp_low = u16::from_le_bytes([data[2], data[3]]);
                        let disp_high = u16::from_le_bytes([data[4], data[5]]);
                        let displacement = ((disp_high as u32) << 16) | (disp_low as u32);
                        Ok(InstructionFormat::Format3 { opcode, r2, r1, mode, displacement })
                    },
                    // Mode registre - Format 1
                    _ => Ok(InstructionFormat::Format1 { opcode, r2, r1, mode }),
                }
            },
            
            // Instructions de branchement (Format 4)
            0x8..=0xB => {
                if data.len() < 4 {
                    return Err(anyhow!("Données insuffisantes pour Format 4"));
                }
                let condition = ((first_word >> 8) & 0xF) as u8;
                let disp_low = (first_word & 0xFF) as i32;
                let disp_high = i16::from_le_bytes([data[2], data[3]]) as i32;
                let displacement = (disp_high << 8) | disp_low;
                Ok(InstructionFormat::Format4 { opcode, condition, displacement })
            },
            
            // Instructions système (Format 5)
            0xC..=0xF => {
                let function = ((first_word >> 8) & 0xF) as u8;
                let immediate = (first_word & 0xFF) as u8;
                Ok(InstructionFormat::Format5 { opcode, function, immediate })
            },
            // Cas par défaut pour les opcodes non supportés
            _ => {
                let r2 = ((first_word >> 8) & 0xF) as u8;
                let r1 = ((first_word >> 4) & 0xF) as u8;
                let mode = (first_word & 0xF) as u8;
                Ok(InstructionFormat::Format1 { opcode, r2, r1, mode })
            },
        }
    }
    
    /// Décode le format d'instruction en instruction exécutable
    fn decode_format(&self, format: InstructionFormat) -> Result<Instruction> {
        match format {
            InstructionFormat::Format1 { opcode, r2, r1, mode } => {
                self.decode_format1(opcode, r2, r1, mode)
            },
            InstructionFormat::Format2 { opcode, r2, r1, mode, immediate } => {
                self.decode_format2(opcode, r2, r1, mode, immediate)
            },
            InstructionFormat::Format3 { opcode, r2, r1, mode, displacement } => {
                self.decode_format3(opcode, r2, r1, mode, displacement)
            },
            InstructionFormat::Format4 { opcode, condition, displacement } => {
                self.decode_format4(opcode, condition, displacement)
            },
            InstructionFormat::Format5 { opcode, function, immediate } => {
                self.decode_format5(opcode, function, immediate)
            },
        }
    }
    
    /// Décode les instructions Format 1 (registre vers registre)
    fn decode_format1(&self, opcode: u8, r2: u8, r1: u8, _mode: u8) -> Result<Instruction> {
        let dest = Operand::Register(r2 as usize);
        let src1 = Operand::Register(r1 as usize);
        let src2 = Operand::Register(r1 as usize); // Pour certaines instructions à 2 opérandes
        
        match opcode {
            0x0 => Ok(Instruction::Add { dest, src1, src2 }),
            0x1 => Ok(Instruction::Sub { dest, src1, src2 }),
            0x2 => Ok(Instruction::Mul { dest, src1, src2 }),
            0x3 => Ok(Instruction::Div { dest, src1, src2 }),
            0x4 => Ok(Instruction::And { dest, src1, src2 }),
            0x5 => Ok(Instruction::Or { dest, src1, src2 }),
            0x6 => Ok(Instruction::Xor { dest, src1, src2 }),
            0x7 => Ok(Instruction::Mov { dest, src: src1 }),
            _ => Err(anyhow!("Opcode Format1 inconnu: {:#x}", opcode)),
        }
    }
    
    /// Décode les instructions Format 2 (avec immédiat)
    fn decode_format2(&self, opcode: u8, r2: u8, r1: u8, _mode: u8, immediate: u16) -> Result<Instruction> {
        let dest = Operand::Register(r2 as usize);
        let src1 = Operand::Register(r1 as usize);
        let src2 = Operand::Immediate(immediate as u32);
        
        match opcode {
            0x0 => Ok(Instruction::Add { dest, src1, src2 }),
            0x1 => Ok(Instruction::Sub { dest, src1, src2 }),
            0x2 => Ok(Instruction::Mul { dest, src1, src2 }),
            0x3 => Ok(Instruction::Compare { src1, src2 }),
            0x4 => Ok(Instruction::And { dest, src1, src2 }),
            0x5 => Ok(Instruction::Or { dest, src1, src2 }),
            0x6 => Ok(Instruction::Test { src1, src2 }),
            0x7 => Ok(Instruction::Mov { dest, src: src2 }),
            _ => Err(anyhow!("Opcode Format2 inconnu: {:#x}", opcode)),
        }
    }
    
    /// Décode les instructions Format 3 (avec déplacement)
    fn decode_format3(&self, opcode: u8, r2: u8, r1: u8, mode: u8, displacement: u32) -> Result<Instruction> {
        let reg = Operand::Register(r2 as usize);
        let base_reg = r1 as usize;
        
        let address = match mode {
            0x3 => Operand::IndirectOffset(base_reg, displacement as i32),
            0x4 => Operand::Direct(displacement),
            _ => Operand::Direct(displacement),
        };
        
        let size = DataSize::DWord; // Par défaut, peut être déterminé par d'autres bits
        
        match opcode {
            0x0 => Ok(Instruction::Load { dest: reg, address, size }),
            0x1 => Ok(Instruction::Store { src: reg, address, size }),
            _ => Err(anyhow!("Opcode Format3 inconnu: {:#x}", opcode)),
        }
    }
    
    /// Décode les instructions Format 4 (branchement)
    fn decode_format4(&self, opcode: u8, condition: u8, displacement: i32) -> Result<Instruction> {
        let target = Operand::PcRelative(displacement);
        
        match opcode {
            0x8 => Ok(Instruction::Jump { target }),
            0x9 => Ok(Instruction::Call { target }),
            0xA | 0xB => {
                let cond = self.decode_condition(condition)?;
                Ok(Instruction::JumpConditional { condition: cond, target })
            },
            _ => Err(anyhow!("Opcode Format4 inconnu: {:#x}", opcode)),
        }
    }
    
    /// Décode les instructions Format 5 (système)
    fn decode_format5(&self, opcode: u8, function: u8, _immediate: u8) -> Result<Instruction> {
        match opcode {
            0xC => match function {
                0x0 => Ok(Instruction::Nop),
                0x1 => Ok(Instruction::Halt),
                0x2 => Ok(Instruction::Return),
                0x3 => Ok(Instruction::InterruptReturn),
                _ => Ok(Instruction::Unknown { opcode: (opcode as u32) << 8 | (function as u32) }),
            },
            _ => Ok(Instruction::Unknown { opcode: opcode as u32 }),
        }
    }
    
    /// Décode un code de condition
    fn decode_condition(&self, condition: u8) -> Result<ConditionCode> {
        match condition {
            0x0 => Ok(ConditionCode::Always),
            0x1 => Ok(ConditionCode::Never),
            0x2 => Ok(ConditionCode::Equal),
            0x3 => Ok(ConditionCode::NotEqual),
            0x4 => Ok(ConditionCode::Carry),
            0x5 => Ok(ConditionCode::NotCarry),
            0x6 => Ok(ConditionCode::Negative),
            0x7 => Ok(ConditionCode::Positive),
            0x8 => Ok(ConditionCode::Overflow),
            0x9 => Ok(ConditionCode::NotOverflow),
            _ => Err(anyhow!("Code de condition invalide: {:#x}", condition)),
        }
    }
    
    /// Calcule la taille d'une instruction en octets
    fn calculate_instruction_size(&self, format: &InstructionFormat) -> u32 {
        match format {
            InstructionFormat::Format1 { .. } => 2,
            InstructionFormat::Format2 { .. } | InstructionFormat::Format4 { .. } => 4,
            InstructionFormat::Format3 { .. } => 6,
            InstructionFormat::Format5 { .. } => 2,
        }
    }
    
    /// Vide le cache d'instructions
    pub fn clear_cache(&mut self) {
        self.instruction_cache.clear();
    }
}