//! Formats d'instructions réels du NEC V60
//!
//! Le NEC V60 utilise plusieurs formats d'instructions avec des longueurs variables

use super::instructions::*;
use super::registers::ConditionCode;
use anyhow::{anyhow, Result};

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
        let opcode = ((first_word >> 10) & 0x3F) as u8;

        let format = self.determine_format(opcode, first_word, data)?;
        let instruction = self.decode_format(&format)?;
        let size = self.calculate_instruction_size(&format);

        let decoded = DecodedInstruction::new(instruction, address, size);

        // Mettre en cache
        self.instruction_cache.insert(address, decoded.clone());

        Ok(decoded)
    }

    /// Détermine le format de l'instruction
    fn determine_format(
        &self,
        opcode: u8,
        first_word: u16,
        data: &[u8],
    ) -> Result<InstructionFormat> {
        match opcode {
            // Instructions Format 1 (16 bits) - opérations basiques
            0x00..=0x0F => {
                let r2 = ((first_word >> 5) & 0x1F) as u8;
                let r1 = (first_word & 0x1F) as u8;
                Ok(InstructionFormat::Format1 {
                    opcode,
                    r2,
                    r1,
                    mode: 0,
                })
            }

            // Instructions Format 2 (32 bits) - avec immédiat
            0x10..=0x1F => {
                if data.len() < 4 {
                    return Err(anyhow!("Données insuffisantes pour Format 2"));
                }
                let r2 = ((first_word >> 5) & 0x1F) as u8;
                let r1 = (first_word & 0x1F) as u8;
                let immediate = u16::from_le_bytes([data[2], data[3]]);
                Ok(InstructionFormat::Format2 {
                    opcode,
                    r2,
                    r1,
                    mode: 0,
                    immediate,
                })
            }

            // Instructions Format 3 (48 bits) - avec déplacement
            0x20..=0x2F => {
                if data.len() < 6 {
                    return Err(anyhow!("Données insuffisantes pour Format 3"));
                }
                let r2 = ((first_word >> 5) & 0x1F) as u8;
                let r1 = (first_word & 0x1F) as u8;
                let displacement = u32::from_le_bytes([data[2], data[3], data[4], data[5]]);
                Ok(InstructionFormat::Format3 {
                    opcode,
                    r2,
                    r1,
                    mode: 0,
                    displacement,
                })
            }

            // Instructions Format 4 (32 bits) - branchements
            0x30..=0x37 => {
                if data.len() < 4 {
                    return Err(anyhow!("Données insuffisantes pour Format 4"));
                }
                let condition = ((first_word >> 5) & 0x1F) as u8;
                let displacement =
                    i32::from_le_bytes([data[1] as i8 as i32 as u8, data[2], data[3], 0]);
                Ok(InstructionFormat::Format4 {
                    opcode,
                    condition,
                    displacement,
                })
            }

            // Instructions Format 5 (16 bits) - système
            0x38..=0x3F => {
                let function = ((first_word >> 5) & 0x1F) as u8;
                let immediate = (first_word & 0x1F) as u8;
                Ok(InstructionFormat::Format5 {
                    opcode,
                    function,
                    immediate,
                })
            }

            _ => Err(anyhow!("Opcode inconnu: 0x{:02X}", opcode)),
        }
    }

    /// Décode un format en instruction
    fn decode_format(&self, format: &InstructionFormat) -> Result<Instruction> {
        match format {
            InstructionFormat::Format1 { opcode, r2, r1, .. } => {
                self.decode_format1(*opcode, *r2, *r1)
            }
            InstructionFormat::Format2 {
                opcode,
                r2,
                r1,
                immediate,
                ..
            } => self.decode_format2(*opcode, *r2, *r1, *immediate),
            InstructionFormat::Format3 {
                opcode,
                r2,
                r1,
                displacement,
                ..
            } => self.decode_format3(*opcode, *r2, *r1, *displacement),
            InstructionFormat::Format4 {
                opcode,
                condition,
                displacement,
            } => self.decode_format4(*opcode, *condition, *displacement),
            InstructionFormat::Format5 {
                opcode,
                function,
                immediate,
            } => self.decode_format5(*opcode, *function, *immediate),
        }
    }

    /// Décode Format 1 (opérations basiques)
    fn decode_format1(&self, opcode: u8, r2: u8, r1: u8) -> Result<Instruction> {
        let dest = Operand::Register(r2 as usize);
        let src = Operand::Register(r1 as usize);

        match opcode {
            0x00 => Ok(Instruction::Mov { dest, src }),
            0x01 => Ok(Instruction::Add {
                dest: dest.clone(),
                src1: dest,
                src2: src,
            }),
            0x02 => Ok(Instruction::Sub {
                dest: dest.clone(),
                src1: dest,
                src2: src,
            }),
            0x03 => Ok(Instruction::And {
                dest: dest.clone(),
                src1: dest,
                src2: src,
            }),
            0x04 => Ok(Instruction::Or {
                dest: dest.clone(),
                src1: dest,
                src2: src,
            }),
            0x05 => Ok(Instruction::Xor {
                dest: dest.clone(),
                src1: dest,
                src2: src,
            }),
            0x06 => Ok(Instruction::Compare {
                src1: dest,
                src2: src,
            }),
            _ => Ok(Instruction::Unknown {
                opcode: (opcode as u32) << 16 | (r2 as u32) << 8 | r1 as u32,
            }),
        }
    }

    /// Décode Format 2 (avec immédiat)
    fn decode_format2(&self, opcode: u8, r2: u8, r1: u8, immediate: u16) -> Result<Instruction> {
        let dest = Operand::Register(r2 as usize);
        let _src = Operand::Register(r1 as usize);
        let imm = Operand::Immediate(immediate as u32);

        match opcode {
            0x10 => Ok(Instruction::Mov { dest, src: imm }),
            0x11 => Ok(Instruction::Add {
                dest: dest.clone(),
                src1: dest,
                src2: imm,
            }),
            0x12 => Ok(Instruction::Sub {
                dest: dest.clone(),
                src1: dest,
                src2: imm,
            }),
            0x13 => Ok(Instruction::And {
                dest: dest.clone(),
                src1: dest,
                src2: imm,
            }),
            0x14 => Ok(Instruction::Or {
                dest: dest.clone(),
                src1: dest,
                src2: imm,
            }),
            0x15 => Ok(Instruction::Xor {
                dest: dest.clone(),
                src1: dest,
                src2: imm,
            }),
            0x16 => Ok(Instruction::Compare {
                src1: dest,
                src2: imm,
            }),
            _ => Ok(Instruction::Unknown {
                opcode: (opcode as u32) << 24
                    | (r2 as u32) << 16
                    | (r1 as u32) << 8
                    | immediate as u32,
            }),
        }
    }

    /// Décode Format 3 (avec déplacement)
    fn decode_format3(&self, opcode: u8, r2: u8, r1: u8, displacement: u32) -> Result<Instruction> {
        let dest = Operand::Register(r2 as usize);
        let addr = Operand::IndirectOffset(r1 as usize, displacement as i32);

        match opcode {
            0x20 => Ok(Instruction::Load {
                dest,
                address: addr,
                size: DataSize::DWord,
            }),
            0x21 => Ok(Instruction::Store {
                src: dest,
                address: addr,
                size: DataSize::DWord,
            }),
            _ => Ok(Instruction::Unknown {
                opcode: (opcode as u32) << 24 | (r2 as u32) << 16 | (r1 as u32) << 8 | displacement,
            }),
        }
    }

    /// Décode Format 4 (branchements)
    fn decode_format4(&self, opcode: u8, condition: u8, displacement: i32) -> Result<Instruction> {
        let target = Operand::Immediate(displacement as u32);
        let cond = match condition {
            0x00 => ConditionCode::Always,
            0x01 => ConditionCode::Equal,
            0x02 => ConditionCode::NotEqual,
            0x03 => ConditionCode::Greater,
            0x04 => ConditionCode::Less,
            0x05 => ConditionCode::GreaterEqual,
            0x06 => ConditionCode::LessEqual,
            _ => ConditionCode::Always,
        };

        match opcode {
            0x30 => Ok(Instruction::Jump { target }),
            0x31 => Ok(Instruction::JumpConditional {
                condition: cond,
                target,
            }),
            0x32 => Ok(Instruction::Call { target }),
            _ => Ok(Instruction::Unknown {
                opcode: (opcode as u32) << 16 | (condition as u32) << 8 | displacement as u32,
            }),
        }
    }

    /// Décode Format 5 (système)
    fn decode_format5(&self, opcode: u8, function: u8, immediate: u8) -> Result<Instruction> {
        match function {
            0x00 => Ok(Instruction::Nop),
            0x01 => Ok(Instruction::Halt),
            0x02 => Ok(Instruction::Return),
            0x03 => Ok(Instruction::InterruptReturn),
            _ => Ok(Instruction::Unknown {
                opcode: (opcode as u32) << 16 | (function as u32) << 8 | immediate as u32,
            }),
        }
    }

    /// Calcule la taille d'une instruction selon son format
    fn calculate_instruction_size(&self, format: &InstructionFormat) -> u32 {
        match format {
            InstructionFormat::Format1 { .. } => 2,
            InstructionFormat::Format2 { .. } => 4,
            InstructionFormat::Format3 { .. } => 6,
            InstructionFormat::Format4 { .. } => 4,
            InstructionFormat::Format5 { .. } => 2,
        }
    }

    /// Vide le cache d'instructions
    pub fn clear_cache(&mut self) {
        self.instruction_cache.clear();
    }
}
