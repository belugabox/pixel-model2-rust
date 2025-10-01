//! Émulation du processeur NEC V60
//! 
//! Le NEC V60 est le processeur principal du SEGA Model 2, fonctionnant à 25MHz.
//! Il s'agit d'un processeur CISC 32-bit avec un jeu d'instructions complexe.

pub mod registers;
pub mod instructions;
pub mod instruction_formats;
pub mod decoder;
pub mod executor;
pub mod arithmetic;
pub mod logical;

use anyhow::Result;

pub use registers::*;
pub use instructions::*;
pub use instruction_formats::*;
pub use decoder::*;
pub use executor::*;
pub use arithmetic::*;
pub use logical::*;

/// Structure principale du processeur NEC V60
#[derive(Debug)]
pub struct NecV60 {
    /// Registres du processeur
    pub registers: V60Registers,
    
    /// Décodeur d'instructions avancé
    pub decoder: V60InstructionDecoder,
    
    /// Compteur de cycles pour la synchronisation
    pub cycle_count: u64,
    
    /// Statistiques d'exécution pour profilage
    pub stats: ExecutionStats,
    
    /// État d'arrêt du processeur
    pub halted: bool,
}

impl NecV60 {
    /// Crée une nouvelle instance du processeur NEC V60
    pub fn new() -> Self {
        Self {
            registers: V60Registers::new(),
            decoder: V60InstructionDecoder::new(),
            cycle_count: 0,
            stats: ExecutionStats::new(),
            halted: false,
        }
    }

    /// Réinitialise le processeur à son état initial
    pub fn reset(&mut self) {
        self.registers.reset();
        self.decoder.clear_cache();
        self.cycle_count = 0;
        self.stats.reset();
        self.halted = false;
    }

    /// Exécute un cycle du processeur
    pub fn step<M>(&mut self, memory: &mut M) -> Result<u32>
    where
        M: crate::memory::MemoryInterface,
    {
        if self.halted {
            return Ok(1); // Un cycle minimal si arrêté
        }

        // Récupérer l'instruction à l'adresse du PC
        let pc = self.registers.pc;
        
        // Lire les données d'instruction depuis la mémoire
        let mut instruction_data = [0u8; 8]; // Maximum 8 octets pour une instruction V60
        for i in 0..8 {
            instruction_data[i] = memory.read_u8(pc + i as u32)?;
        }
        
        // Décoder l'instruction
        let instruction = self.decoder.decode(&instruction_data, pc)?;

        // Exécuter l'instruction
        let cycles = self.execute_instruction(&instruction, memory)?;
        self.cycle_count += cycles as u64;

        Ok(cycles)
    }

    /// Exécute plusieurs cycles du processeur
    pub fn run_cycles<M>(&mut self, cycles: u32, memory: &mut M) -> Result<u32>
    where
        M: crate::memory::MemoryInterface,
    {
        let mut executed_cycles = 0;
        
        while executed_cycles < cycles && !self.halted {
            executed_cycles += self.step(memory)?;
        }
        
        Ok(executed_cycles)
    }

    /// Obtient l'état actuel du processeur pour le débogage
    pub fn get_debug_state(&self) -> CpuDebugState {
        CpuDebugState {
            registers: self.registers.clone(),
            cycle_count: self.cycle_count,
            halted: self.halted,
        }
    }
}

impl Default for NecV60 {
    fn default() -> Self {
        Self::new()
    }
}

/// État de débogage du processeur
#[derive(Debug, Clone)]
pub struct CpuDebugState {
    pub registers: V60Registers,
    pub cycle_count: u64,
    pub halted: bool,
}