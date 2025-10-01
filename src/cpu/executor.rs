//! Exécuteur d'instructions NEC V60

use super::{NecV60, instructions::*, registers::ConditionCode};
use crate::memory::MemoryInterface;
use anyhow::{Result, anyhow};

impl NecV60 {
    /// Exécute une instruction décodée
    pub fn execute_instruction<M>(&mut self, instruction: &DecodedInstruction, memory: &mut M) -> Result<u32>
    where
        M: MemoryInterface,
    {
        match &instruction.instruction {
            // Instructions arithmétiques
            Instruction::Add { dest, src1, src2 } => {
                let val1 = self.read_operand(src1, memory)?;
                let val2 = self.read_operand(src2, memory)?;
                let (result, overflow) = val1.overflowing_add(val2);
                let carry = result < val1; // Détection simple de retenue
                
                self.write_operand(dest, result, memory)?;
                self.registers.psw.update_flags(result, carry, overflow);
                self.registers.pc += instruction.size;
            },
            
            Instruction::Sub { dest, src1, src2 } => {
                let val1 = self.read_operand(src1, memory)?;
                let val2 = self.read_operand(src2, memory)?;
                let (result, overflow) = val1.overflowing_sub(val2);
                let carry = val1 < val2; // Détection simple de retenue
                
                self.write_operand(dest, result, memory)?;
                self.registers.psw.update_flags(result, carry, overflow);
                self.registers.pc += instruction.size;
            },
            
            Instruction::Mul { dest, src1, src2 } => {
                let val1 = self.read_operand(src1, memory)? as u64;
                let val2 = self.read_operand(src2, memory)? as u64;
                let result = val1.wrapping_mul(val2);
                let overflow = result > u32::MAX as u64;
                
                self.write_operand(dest, result as u32, memory)?;
                self.registers.psw.update_flags(result as u32, false, overflow);
                self.registers.pc += instruction.size;
            },
            
            Instruction::Div { dest, src1, src2 } => {
                let val1 = self.read_operand(src1, memory)?;
                let val2 = self.read_operand(src2, memory)?;
                
                if val2 == 0 {
                    return Err(anyhow!("Division par zéro"));
                }
                
                let result = val1 / val2;
                self.write_operand(dest, result, memory)?;
                self.registers.psw.update_flags(result, false, false);
                self.registers.pc += instruction.size;
            },
            
            // Instructions logiques
            Instruction::And { dest, src1, src2 } => {
                let val1 = self.read_operand(src1, memory)?;
                let val2 = self.read_operand(src2, memory)?;
                let result = val1 & val2;
                
                self.write_operand(dest, result, memory)?;
                self.registers.psw.update_flags(result, false, false);
                self.registers.pc += instruction.size;
            },
            
            Instruction::Or { dest, src1, src2 } => {
                let val1 = self.read_operand(src1, memory)?;
                let val2 = self.read_operand(src2, memory)?;
                let result = val1 | val2;
                
                self.write_operand(dest, result, memory)?;
                self.registers.psw.update_flags(result, false, false);
                self.registers.pc += instruction.size;
            },
            
            Instruction::Xor { dest, src1, src2 } => {
                let val1 = self.read_operand(src1, memory)?;
                let val2 = self.read_operand(src2, memory)?;
                let result = val1 ^ val2;
                
                self.write_operand(dest, result, memory)?;
                self.registers.psw.update_flags(result, false, false);
                self.registers.pc += instruction.size;
            },
            
            Instruction::Not { dest, src } => {
                let val = self.read_operand(src, memory)?;
                let result = !val;
                
                self.write_operand(dest, result, memory)?;
                self.registers.psw.update_flags(result, false, false);
                self.registers.pc += instruction.size;
            },
            
            // Instructions de décalage
            Instruction::Shl { dest, src, shift } => {
                let val = self.read_operand(src, memory)?;
                let shift_amount = self.read_operand(shift, memory)? & 0x1F; // Masquer à 5 bits
                let result = val << shift_amount;
                let carry = if shift_amount > 0 { (val >> (32 - shift_amount)) & 1 != 0 } else { false };
                
                self.write_operand(dest, result, memory)?;
                self.registers.psw.update_flags(result, carry, false);
                self.registers.pc += instruction.size;
            },
            
            Instruction::Shr { dest, src, shift } => {
                let val = self.read_operand(src, memory)?;
                let shift_amount = self.read_operand(shift, memory)? & 0x1F;
                let result = val >> shift_amount;
                let carry = if shift_amount > 0 { (val >> (shift_amount - 1)) & 1 != 0 } else { false };
                
                self.write_operand(dest, result, memory)?;
                self.registers.psw.update_flags(result, carry, false);
                self.registers.pc += instruction.size;
            },
            
            // Instructions de transfert
            Instruction::Mov { dest, src } => {
                let val = self.read_operand(src, memory)?;
                self.write_operand(dest, val, memory)?;
                self.registers.pc += instruction.size;
            },
            
            Instruction::Load { dest, address, size } => {
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
                let addr = self.read_operand(target, memory)?;
                self.registers.pc = addr;
            },
            
            Instruction::JumpConditional { condition, target } => {
                if self.registers.psw.condition_met(*condition) {
                    let addr = self.read_operand(target, memory)?;
                    self.registers.pc = addr;
                } else {
                    self.registers.pc += instruction.size;
                }
            },
            
            Instruction::Call { target } => {
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
                let val1 = self.read_operand(src1, memory)?;
                let val2 = self.read_operand(src2, memory)?;
                let (result, overflow) = val1.overflowing_sub(val2);
                let carry = val1 < val2;
                
                // Compare met à jour les flags mais ne stocke pas le résultat
                self.registers.psw.update_flags(result, carry, overflow);
                self.registers.pc += instruction.size;
            },
            
            Instruction::Test { src1, src2 } => {
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
                // Conversion approximative pour les flottants
                let val1 = f32::from_bits(self.read_operand(src1, memory)?);
                let val2 = f32::from_bits(self.read_operand(src2, memory)?);
                let result = val1 + val2;
                
                self.write_operand(dest, result.to_bits(), memory)?;
                self.registers.pc += instruction.size;
            },
            
            Instruction::FloatMul { dest, src1, src2 } => {
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

    /// Lit la valeur d'un opérande
    fn read_operand<M>(&self, operand: &Operand, memory: &M) -> Result<u32>
    where
        M: MemoryInterface,
    {
        match operand {
            Operand::Register(reg) => Ok(self.registers.read_general(*reg)),
            Operand::Immediate(val) => Ok(*val),
            Operand::Direct(addr) => memory.read_u32(*addr),
            Operand::Indirect(reg) => {
                let addr = self.registers.read_general(*reg);
                memory.read_u32(addr)
            },
            Operand::IndirectOffset(reg, offset) => {
                let base = self.registers.read_general(*reg);
                let addr = (base as i32 + offset) as u32;
                memory.read_u32(addr)
            },
            Operand::IndirectIndexed(base_reg, index_reg, scale) => {
                let base = self.registers.read_general(*base_reg);
                let index = self.registers.read_general(*index_reg);
                let addr = base + (index * scale);
                memory.read_u32(addr)
            },
            Operand::PcRelative(offset) => {
                let addr = (self.registers.pc as i32 + offset) as u32;
                memory.read_u32(addr)
            },
        }
    }

    /// Écrit une valeur dans un opérande
    fn write_operand<M>(&mut self, operand: &Operand, value: u32, memory: &mut M) -> Result<()>
    where
        M: MemoryInterface,
    {
        match operand {
            Operand::Register(reg) => {
                self.registers.write_general(*reg, value);
                Ok(())
            },
            Operand::Direct(addr) => memory.write_u32(*addr, value),
            Operand::Indirect(reg) => {
                let addr = self.registers.read_general(*reg);
                memory.write_u32(addr, value)
            },
            Operand::IndirectOffset(reg, offset) => {
                let base = self.registers.read_general(*reg);
                let addr = (base as i32 + offset) as u32;
                memory.write_u32(addr, value)
            },
            Operand::IndirectIndexed(base_reg, index_reg, scale) => {
                let base = self.registers.read_general(*base_reg);
                let index = self.registers.read_general(*index_reg);
                let addr = base + (index * scale);
                memory.write_u32(addr, value)
            },
            _ => Err(anyhow!("Impossible d'écrire dans cet opérande")),
        }
    }
}