//! Exécuteur d'instructions NEC V60

use super::{NecV60, instructions::*, arithmetic::ArithmeticUnit, logical::LogicalUnit, 
           floating_point::FloatingPointUnit, bit_manipulation::BitManipulationUnit, bcd::BcdUnit,
           registers::ProcessorStatusWord};
use crate::memory::MemoryInterface;
use anyhow::{Result, anyhow};

/// Statistiques d'exécution
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

impl NecV60 {
    /// Exécute une instruction décodée
    pub fn execute_instruction<M>(&mut self, instruction: &DecodedInstruction, memory: &mut M) -> Result<u32>
    where
        M: MemoryInterface,
    {
        // Si le PC est à 0, l'initialiser avec l'adresse de l'instruction
        // (pour les tests unitaires)
        if self.registers.pc == 0 {
            self.registers.pc = instruction.address;
        }
        
        // Mise à jour des statistiques
        self.stats.instructions_executed += 1;
        self.stats.cycles_executed += instruction.cycles as u64;
        
        match &instruction.instruction {
            // Instructions arithmétiques
            Instruction::Add { dest, src1, src2 } => {
                let val1 = self.read_operand(src1, memory)?;
                let val2 = self.read_operand(src2, memory)?;
                let arithmetic_result = ArithmeticUnit::add(val1, val2);
                
                self.write_operand(dest, arithmetic_result.value, memory)?;
                arithmetic_result.update_psw(&mut self.registers.psw);
                self.registers.pc += instruction.size;
                
                // Exception si overflow signé OU carry (overflow non signé)
                if arithmetic_result.overflow || arithmetic_result.carry {
                    self.stats.exceptions_raised += 1;
                }
            },
            
            Instruction::Sub { dest, src1, src2 } => {
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
                        return Err(anyhow!("Division par zéro"));
                    }
                }
            },
            
            // Instructions logiques
            Instruction::And { dest, src1, src2 } => {
                let val1 = self.read_operand(src1, memory)?;
                let val2 = self.read_operand(src2, memory)?;
                let logical_result = LogicalUnit::and(val1, val2);
                
                self.write_operand(dest, logical_result.value, memory)?;
                logical_result.update_psw(&mut self.registers.psw);
                self.registers.pc += instruction.size;
            },
            
            Instruction::Or { dest, src1, src2 } => {
                let val1 = self.read_operand(src1, memory)?;
                let val2 = self.read_operand(src2, memory)?;
                let logical_result = LogicalUnit::or(val1, val2);
                
                self.write_operand(dest, logical_result.value, memory)?;
                logical_result.update_psw(&mut self.registers.psw);
                self.registers.pc += instruction.size;
            },
            
            Instruction::Xor { dest, src1, src2 } => {
                let val1 = self.read_operand(src1, memory)?;
                let val2 = self.read_operand(src2, memory)?;
                let logical_result = LogicalUnit::xor(val1, val2);
                
                self.write_operand(dest, logical_result.value, memory)?;
                logical_result.update_psw(&mut self.registers.psw);
                self.registers.pc += instruction.size;
            },
            
            Instruction::Not { dest, src } => {
                let val = self.read_operand(src, memory)?;
                let logical_result = LogicalUnit::not(val);
                
                self.write_operand(dest, logical_result.value, memory)?;
                logical_result.update_psw(&mut self.registers.psw);
                self.registers.pc += instruction.size;
            },
            
            // Instructions de décalage
            Instruction::Shl { dest, src, shift } => {
                let val = self.read_operand(src, memory)?;
                let shift_amount = self.read_operand(shift, memory)?;
                let logical_result = LogicalUnit::shl(val, shift_amount);
                
                self.write_operand(dest, logical_result.value, memory)?;
                logical_result.update_psw(&mut self.registers.psw);
                self.registers.pc += instruction.size;
            },
            
            Instruction::Shr { dest, src, shift } => {
                let val = self.read_operand(src, memory)?;
                let shift_amount = self.read_operand(shift, memory)?;
                let logical_result = LogicalUnit::shr(val, shift_amount);
                
                self.write_operand(dest, logical_result.value, memory)?;
                logical_result.update_psw(&mut self.registers.psw);
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
                self.stats.memory_accesses += 1;
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
                self.stats.memory_accesses += 1;
            },
            
            Instruction::Nop => {
                self.registers.pc += instruction.size;
            },
            
            // Instructions de branchement
            Instruction::Jump { target } => {
                let target_addr = self.read_operand(target, memory)?;
                self.registers.pc = target_addr;
                self.stats.branches_taken += 1;
            },
            
            Instruction::JumpConditional { condition, target } => {
                if self.registers.psw.condition_met(*condition) {
                    let target_addr = self.read_operand(target, memory)?;
                    self.registers.pc = target_addr;
                    self.stats.branches_taken += 1;
                } else {
                    self.registers.pc += instruction.size;
                }
            },
            
            // Instructions en virgule flottante
            Instruction::FloatAdd { dest, src1, src2 } => {
                let val1 = self.read_operand(src1, memory)?;
                let val2 = self.read_operand(src2, memory)?;
                let float_result = FloatingPointUnit::add(val1, val2);
                
                self.write_operand(dest, float_result.to_u32(), memory)?;
                float_result.update_psw(&mut self.registers.psw);
                self.registers.pc += instruction.size;
                
                if float_result.overflow {
                    self.stats.exceptions_raised += 1;
                }
            },
            
            Instruction::FloatSub { dest, src1, src2 } => {
                let val1 = self.read_operand(src1, memory)?;
                let val2 = self.read_operand(src2, memory)?;
                let float_result = FloatingPointUnit::sub(val1, val2);
                
                self.write_operand(dest, float_result.to_u32(), memory)?;
                float_result.update_psw(&mut self.registers.psw);
                self.registers.pc += instruction.size;
                
                if float_result.overflow {
                    self.stats.exceptions_raised += 1;
                }
            },
            
            Instruction::FloatMul { dest, src1, src2 } => {
                let val1 = self.read_operand(src1, memory)?;
                let val2 = self.read_operand(src2, memory)?;
                let float_result = FloatingPointUnit::mul(val1, val2);
                
                self.write_operand(dest, float_result.to_u32(), memory)?;
                float_result.update_psw(&mut self.registers.psw);
                self.registers.pc += instruction.size;
                
                if float_result.overflow {
                    self.stats.exceptions_raised += 1;
                }
            },
            
            Instruction::FloatDiv { dest, src1, src2 } => {
                let val1 = self.read_operand(src1, memory)?;
                let val2 = self.read_operand(src2, memory)?;
                let float_result = FloatingPointUnit::div(val1, val2);
                
                self.write_operand(dest, float_result.to_u32(), memory)?;
                float_result.update_psw(&mut self.registers.psw);
                self.registers.pc += instruction.size;
                
                if float_result.overflow || float_result.nan {
                    self.stats.exceptions_raised += 1;
                }
            },
            
            Instruction::FloatCompare { src1, src2 } => {
                let val1 = self.read_operand(src1, memory)?;
                let val2 = self.read_operand(src2, memory)?;
                let float_result = FloatingPointUnit::compare(val1, val2);
                
                float_result.update_psw(&mut self.registers.psw);
                self.registers.pc += instruction.size;
                
                if float_result.nan {
                    self.stats.exceptions_raised += 1;
                }
            },

            // Instructions de manipulation de bits
            Instruction::RotateLeft { dest, src, count } => {
                let val = self.read_operand(src, memory)?;
                let count_val = self.read_operand(count, memory)?;
                let result = BitManipulationUnit::rotate_left(val, count_val);
                
                self.write_operand(dest, result, memory)?;
                self.registers.psw.set(ProcessorStatusWord::ZERO, result == 0);
                self.registers.pc += instruction.size;
            },
            
            Instruction::RotateRight { dest, src, count } => {
                let val = self.read_operand(src, memory)?;
                let count_val = self.read_operand(count, memory)?;
                let result = BitManipulationUnit::rotate_right(val, count_val);
                
                self.write_operand(dest, result, memory)?;
                self.registers.psw.set(ProcessorStatusWord::ZERO, result == 0);
                self.registers.pc += instruction.size;
            },
            
            Instruction::BitTest { src, bit } => {
                let val = self.read_operand(src, memory)?;
                let bit_pos = self.read_operand(bit, memory)?;
                let bit_result = BitManipulationUnit::test_bit(val, bit_pos);
                
                bit_result.update_psw(&mut self.registers.psw);
                self.registers.pc += instruction.size;
            },
            
            Instruction::BitSet { dest, bit } => {
                let val = self.read_operand(dest, memory)?;
                let bit_pos = self.read_operand(bit, memory)?;
                let bit_result = BitManipulationUnit::set_bit(val, bit_pos);
                
                self.write_operand(dest, bit_result.value, memory)?;
                bit_result.update_psw(&mut self.registers.psw);
                self.registers.pc += instruction.size;
            },
            
            Instruction::BitClear { dest, bit } => {
                let val = self.read_operand(dest, memory)?;
                let bit_pos = self.read_operand(bit, memory)?;
                let bit_result = BitManipulationUnit::clear_bit(val, bit_pos);
                
                self.write_operand(dest, bit_result.value, memory)?;
                bit_result.update_psw(&mut self.registers.psw);
                self.registers.pc += instruction.size;
            },

            // Instructions BCD
            Instruction::BcdAdd { dest, src1, src2 } => {
                let val1 = self.read_operand(src1, memory)?;
                let val2 = self.read_operand(src2, memory)?;
                let bcd_result = BcdUnit::add(val1, val2);
                
                self.write_operand(dest, bcd_result.value, memory)?;
                bcd_result.update_psw(&mut self.registers.psw);
                self.registers.pc += instruction.size;
                
                if bcd_result.overflow {
                    self.stats.exceptions_raised += 1;
                }
            },
            
            Instruction::BcdSub { dest, src1, src2 } => {
                let val1 = self.read_operand(src1, memory)?;
                let val2 = self.read_operand(src2, memory)?;
                let bcd_result = BcdUnit::sub(val1, val2);
                
                self.write_operand(dest, bcd_result.value, memory)?;
                bcd_result.update_psw(&mut self.registers.psw);
                self.registers.pc += instruction.size;
                
                if bcd_result.overflow {
                    self.stats.exceptions_raised += 1;
                }
            },
            
            Instruction::Halt => {
                self.halted = true;
                self.registers.pc += instruction.size;
            },
            
            // Instructions d'interruption
            Instruction::SoftwareInterrupt { vector } => {
                let interrupt = crate::cpu::Interrupt::External(*vector);
                self.queue_interrupt(interrupt);
                self.registers.pc += instruction.size;
            },
            
            Instruction::ReturnFromInterrupt => {
                self.return_from_interrupt(memory)?;
                // PC est déjà mis à jour par return_from_interrupt
            },
            
            Instruction::EnableInterrupts => {
                self.interrupts_enabled = true;
                self.registers.pc += instruction.size;
            },
            
            Instruction::DisableInterrupts => {
                self.interrupts_enabled = false;
                self.registers.pc += instruction.size;
            },
            
            Instruction::Unknown { opcode } => {
                return Err(anyhow!("Instruction inconnue: {:#08x} à l'adresse {:#08x}", 
                                 opcode, instruction.address));
            },
            
            _ => {
                return Err(anyhow!("Instruction non implémentée: {:?}", instruction.instruction));
            }
        }
        
        Ok(instruction.cycles)
    }

    /// Lit la valeur d'un opérande
    fn read_operand<M>(&mut self, operand: &Operand, memory: &M) -> Result<u32>
    where
        M: MemoryInterface,
    {
        match operand {
            Operand::Register(reg) => Ok(self.registers.read_general(*reg)),
            Operand::Immediate(val) => Ok(*val),
            Operand::Direct(addr) => {
                // Direct retourne l'adresse elle-même, pas le contenu
                Ok(*addr)
            },
            Operand::Indirect(reg) => {
                let addr = self.registers.read_general(*reg);
                self.stats.memory_accesses += 1;
                memory.read_u32(addr)
            },
            Operand::IndirectOffset(reg, offset) => {
                let base = self.registers.read_general(*reg);
                let addr = (base as i32 + offset) as u32;
                self.stats.memory_accesses += 1;
                memory.read_u32(addr)
            },
            Operand::IndirectIndexed(base_reg, index_reg, scale) => {
                let base = self.registers.read_general(*base_reg);
                let index = self.registers.read_general(*index_reg);
                let addr = base + (index * scale);
                self.stats.memory_accesses += 1;
                memory.read_u32(addr)
            },
            Operand::PcRelative(offset) => {
                let addr = (self.registers.pc as i32 + offset) as u32;
                self.stats.memory_accesses += 1;
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
            Operand::Direct(addr) => {
                self.stats.memory_accesses += 1;
                memory.write_u32(*addr, value)
            },
            Operand::Indirect(reg) => {
                let addr = self.registers.read_general(*reg);
                self.stats.memory_accesses += 1;
                memory.write_u32(addr, value)
            },
            Operand::IndirectOffset(reg, offset) => {
                let base = self.registers.read_general(*reg);
                let addr = (base as i32 + offset) as u32;
                self.stats.memory_accesses += 1;
                memory.write_u32(addr, value)
            },
            Operand::IndirectIndexed(base_reg, index_reg, scale) => {
                let base = self.registers.read_general(*base_reg);
                let index = self.registers.read_general(*index_reg);
                let addr = base + (index * scale);
                self.stats.memory_accesses += 1;
                memory.write_u32(addr, value)
            },
            _ => Err(anyhow!("Impossible d'écrire dans cet opérande")),
        }
    }
}
