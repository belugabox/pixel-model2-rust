//! Exécuteur d'instructions NEC V60

use super::{NecV60, instructions::*, registers::ConditionCode, arithmetic::ArithmeticUnit, logical::LogicalUnit};
use crate::memory::MemoryInterface;
use anyhow::{Result, anyhow};

/// Statistiques d'exécution pour profilage
#[derive(Debug, Default)]
pub struct ExecutionStats {
    pub instructions_executed: u64,
    pub cycles_executed: u64,
    pub branches_taken: u64,
    pub memory_accesses: u64,
    pub cache_hits: u64,
    pub exceptions_raised: u64,
}

impl ExecutionStats {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Exceptions d'exécution
#[derive(Debug, Clone)]
pub enum ExecutionException {
    DivisionByZero,
    InvalidMemoryAccess,
    UnknownInstruction,
    StackOverflow,
    StackUnderflow,
    FloatingPointError,
}

impl std::fmt::Display for ExecutionException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionException::DivisionByZero => write!(f, "Division par zéro"),
            ExecutionException::InvalidMemoryAccess => write!(f, "Accès mémoire invalide"),
            ExecutionException::UnknownInstruction => write!(f, "Instruction inconnue"),
            ExecutionException::StackOverflow => write!(f, "Débordement de pile"),
            ExecutionException::StackUnderflow => write!(f, "Pile vide"),
            ExecutionException::FloatingPointError => write!(f, "Erreur de calcul flottant"),
        }
    }
}

impl std::error::Error for ExecutionException {}

impl NecV60 {
    /// Exécute une instruction décodée avec gestion avancée des erreurs et statistiques
    pub fn execute_instruction<M>(&mut self, instruction: &DecodedInstruction, memory: &mut M) -> Result<u32>
    where
        M: MemoryInterface,
    {
        // Mise à jour des statistiques
        self.stats.instructions_executed += 1;
        self.stats.cycles_executed += instruction.cycles as u64;
        
        match &instruction.instruction {
            // Instructions arithmétiques
            Instruction::Add { dest, src1, src2 } => {
                // Compter les accès mémoire pour les lectures
                self.count_memory_access(src1);
                self.count_memory_access(src2);
                
                let val1 = self.read_operand(src1, memory)?;
                let val2 = self.read_operand(src2, memory)?;
                let arithmetic_result = ArithmeticUnit::add(val1, val2);
                
                self.write_operand(dest, arithmetic_result.value, memory)?;
                arithmetic_result.update_psw(&mut self.registers.psw);
                self.registers.pc += instruction.size;
                
                if arithmetic_result.overflow {
                    self.stats.exceptions_raised += 1;
                }
            },
            
            Instruction::Sub { dest, src1, src2 } => {
                self.count_memory_access(src1);
                self.count_memory_access(src2);
                
                let val1 = self.read_operand(src1, memory)?;
                let val2 = self.read_operand(src2, memory)?;
                let arithmetic_result = ArithmeticUnit::sub(val1, val2);
                
                self.write_operand(dest, arithmetic_result.value, memory)?;
                arithmetic_result.update_psw(&mut self.registers.psw);
                self.registers.pc += instruction.size;
                
                if arithmetic_result.overflow {
                    self.stats.exceptions_raised += 1;
                }
            },
            
            Instruction::Mul { dest, src1, src2 } => {
                self.count_memory_access(src1);
                self.count_memory_access(src2);
                
                let val1 = self.read_operand(src1, memory)?;
                let val2 = self.read_operand(src2, memory)?;
                let arithmetic_result = ArithmeticUnit::mul(val1, val2);
                
                self.write_operand(dest, arithmetic_result.value, memory)?;
                arithmetic_result.update_psw(&mut self.registers.psw);
                self.registers.pc += instruction.size;
                
                if arithmetic_result.overflow {
                    self.stats.exceptions_raised += 1;
                }
            },
            
            Instruction::Div { dest, src1, src2 } => {
                self.count_memory_access(src1);
                self.count_memory_access(src2);
                
                let val1 = self.read_operand(src1, memory)?;
                let val2 = self.read_operand(src2, memory)?;
                
                match ArithmeticUnit::div(val1, val2) {
                    Ok(arithmetic_result) => {
                        self.write_operand(dest, arithmetic_result.value, memory)?;
                        arithmetic_result.update_psw(&mut self.registers.psw);
                        self.registers.pc += instruction.size;
                    }
                    Err(_) => {
                        self.stats.exceptions_raised += 1;
                        return Err(ExecutionException::DivisionByZero.into());
                    }
                }
            },
            
            // Instructions logiques
            Instruction::And { dest, src1, src2 } => {
                self.count_memory_access(src1);
                self.count_memory_access(src2);
                
                let val1 = self.read_operand(src1, memory)?;
                let val2 = self.read_operand(src2, memory)?;
                let logical_result = LogicalUnit::and(val1, val2);
                
                self.write_operand(dest, logical_result.value, memory)?;
                logical_result.update_psw(&mut self.registers.psw);
                self.registers.pc += instruction.size;
            },
            
            Instruction::Or { dest, src1, src2 } => {
                self.count_memory_access(src1);
                self.count_memory_access(src2);
                
                let val1 = self.read_operand(src1, memory)?;
                let val2 = self.read_operand(src2, memory)?;
                let logical_result = LogicalUnit::or(val1, val2);
                
                self.write_operand(dest, logical_result.value, memory)?;
                logical_result.update_psw(&mut self.registers.psw);
                self.registers.pc += instruction.size;
            },
            
            Instruction::Xor { dest, src1, src2 } => {
                self.count_memory_access(src1);
                self.count_memory_access(src2);
                
                let val1 = self.read_operand(src1, memory)?;
                let val2 = self.read_operand(src2, memory)?;
                let logical_result = LogicalUnit::xor(val1, val2);
                
                self.write_operand(dest, logical_result.value, memory)?;
                logical_result.update_psw(&mut self.registers.psw);
                self.registers.pc += instruction.size;
            },
            
            Instruction::Not { dest, src } => {
                self.count_memory_access(src);
                
                let val = self.read_operand(src, memory)?;
                let logical_result = LogicalUnit::not(val);
                
                self.write_operand(dest, logical_result.value, memory)?;
                logical_result.update_psw(&mut self.registers.psw);
                self.registers.pc += instruction.size;
            },
            
            // Instructions de décalage
            Instruction::Shl { dest, src, shift } => {
                self.count_memory_access(src);
                self.count_memory_access(shift);
                
                let val = self.read_operand(src, memory)?;
                let shift_amount = self.read_operand(shift, memory)?;
                let logical_result = LogicalUnit::shl(val, shift_amount);
                
                self.write_operand(dest, logical_result.value, memory)?;
                logical_result.update_psw(&mut self.registers.psw);
                self.registers.pc += instruction.size;
            },
            
            Instruction::Shr { dest, src, shift } => {
                self.count_memory_access(src);
                self.count_memory_access(shift);
                
                let val = self.read_operand(src, memory)?;
                let shift_amount = self.read_operand(shift, memory)?;
                let logical_result = LogicalUnit::shr(val, shift_amount);
                
                self.write_operand(dest, logical_result.value, memory)?;
                logical_result.update_psw(&mut self.registers.psw);
                self.registers.pc += instruction.size;
            },
            
            // Instructions de transfert
            Instruction::Mov { dest, src } => {
                self.count_memory_access(src);
                
                let val = self.read_operand(src, memory)?;
                self.write_operand(dest, val, memory)?;
                self.registers.pc += instruction.size;
            },
            
            Instruction::Load { dest, address, size } => {
                self.count_memory_access(address);
                
                let addr = self.read_operand(address, memory)?;
                let val = match size {
                    DataSize::Byte => memory.read_u8(addr)? as u32,
                    DataSize::Word => memory.read_u16(addr)? as u32,
                    DataSize::DWord => memory.read_u32(addr)?,
                };
                self.write_operand(dest, val, memory)?;
                self.registers.pc += instruction.size;
            },
            
            Instruction::Store { src, address, size } => {
                self.count_memory_access(src);
                self.count_memory_access(address);
                
                let val = self.read_operand(src, memory)?;
                let addr = self.read_operand(address, memory)?;
                match size {
                    DataSize::Byte => memory.write_u8(addr, val as u8)?,
                    DataSize::Word => memory.write_u16(addr, val as u16)?,
                    DataSize::DWord => memory.write_u32(addr, val)?,
                };
                self.registers.pc += instruction.size;
            },
            
            // Instructions de branchement
            Instruction::Jump { target } => {
                self.count_memory_access(target);
                
                let addr = self.read_operand(target, memory)?;
                self.registers.pc = addr;
                self.stats.branches_taken += 1;
            },
            
            Instruction::JumpConditional { condition, target } => {
                if self.registers.psw.condition_met(*condition) {
                    self.count_memory_access(target);
                    let addr = self.read_operand(target, memory)?;
                    self.registers.pc = addr;
                    self.stats.branches_taken += 1;
                } else {
                    self.registers.pc += instruction.size;
                }
            },
            
            Instruction::Call { target } => {
                self.count_memory_access(target);
                
                // Pousser l'adresse de retour sur la pile
                self.registers.sp -= 4;
                memory.write_u32(self.registers.sp, self.registers.pc + instruction.size)?;
                
                // Sauter à l'adresse cible
                let addr = self.read_operand(target, memory)?;
                self.registers.pc = addr;
            },
            
            Instruction::Return => {
                // Récupérer l'adresse de retour depuis la pile
                let return_addr = memory.read_u32(self.registers.sp)?;
                self.registers.sp += 4;
                self.registers.pc = return_addr;
            },
            
            // Instructions de comparaison
            Instruction::Compare { src1, src2 } => {
                self.count_memory_access(src1);
                self.count_memory_access(src2);
                
                let val1 = self.read_operand(src1, memory)?;
                let val2 = self.read_operand(src2, memory)?;
                let (result, overflow) = val1.overflowing_sub(val2);
                let carry = val1 < val2;
                
                // Compare met à jour les flags mais ne stocke pas le résultat
                self.registers.psw.update_flags(result, carry, overflow);
                self.registers.pc += instruction.size;
            },
            
            Instruction::Test { src1, src2 } => {
                self.count_memory_access(src1);
                self.count_memory_access(src2);
                
                let val1 = self.read_operand(src1, memory)?;
                let val2 = self.read_operand(src2, memory)?;
                let result = val1 & val2;
                
                // Test met à jour les flags mais ne stocke pas le résultat
                self.registers.psw.update_flags(result, false, false);
                self.registers.pc += instruction.size;
            },
            
            // Instructions système
            Instruction::Nop => {
                self.registers.pc += instruction.size;
            },
            
            Instruction::Halt => {
                self.halted = true;
                self.registers.pc += instruction.size;
            },
            
            Instruction::InterruptReturn => {
                // Restaurer l'état depuis la pile (simplifié)
                let return_addr = memory.read_u32(self.registers.sp)?;
                self.registers.sp += 4;
                let psw_value = memory.read_u32(self.registers.sp)?;
                self.registers.sp += 4;
                
                self.registers.pc = return_addr;
                self.registers.psw = super::registers::ProcessorStatusWord::from_bits_truncate(psw_value);
            },
            
            // Instructions flottantes (implémentation basique)
            Instruction::FloatAdd { dest, src1, src2 } => {
                self.count_memory_access(src1);
                self.count_memory_access(src2);
                
                // Conversion approximative pour les flottants
                let val1 = f32::from_bits(self.read_operand(src1, memory)?);
                let val2 = f32::from_bits(self.read_operand(src2, memory)?);
                let result = val1 + val2;
                
                self.write_operand(dest, result.to_bits(), memory)?;
                self.registers.pc += instruction.size;
            },
            
            Instruction::FloatMul { dest, src1, src2 } => {
                self.count_memory_access(src1);
                self.count_memory_access(src2);
                
                let val1 = f32::from_bits(self.read_operand(src1, memory)?);
                let val2 = f32::from_bits(self.read_operand(src2, memory)?);
                let result = val1 * val2;
                
                self.write_operand(dest, result.to_bits(), memory)?;
                self.registers.pc += instruction.size;
            },
            
            // Instruction inconnue
            Instruction::Unknown { opcode } => {
                return Err(anyhow!("Instruction inconnue: {:#08x} à l'adresse {:#08x}", 
                                 opcode, instruction.address));
            },
        }
        
        Ok(instruction.cycles)
    }
