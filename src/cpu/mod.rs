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
pub mod floating_point;
pub mod bit_manipulation;
pub mod string_operations;
pub mod bcd;

use anyhow::Result;

pub use registers::*;
pub use instruction_formats::*;
pub use executor::*;

/// Types d'interruptions du SEGA Model 2
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interrupt {
    /// Interruption VBLANK (fin de frame vidéo)
    VBlank = 0x01,
    
    /// Interruption timer principal
    TimerMain = 0x02,
    
    /// Interruption timer secondaire
    TimerSub = 0x03,
    
    /// Interruption GPU
    Gpu = 0x04,
    
    /// Interruption audio
    Audio = 0x05,
    
    /// Interruption d'entrée
    Input = 0x06,
    
    /// Interruption externe générique
    External(u8),
}

impl Interrupt {
    /// Retourne le vecteur d'interruption (adresse dans la table des vecteurs)
    pub fn vector_address(self) -> u32 {
        match self {
            Interrupt::VBlank => 0x00000040,
            Interrupt::TimerMain => 0x00000044,
            Interrupt::TimerSub => 0x00000048,
            Interrupt::Gpu => 0x0000004C,
            Interrupt::Audio => 0x00000050,
            Interrupt::Input => 0x00000054,
            Interrupt::External(vector) => 0x00000058 + (vector as u32 * 4),
        }
    }
}

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
    
    /// État des interruptions
    pub interrupts_enabled: bool,
    
    /// File d'attente des interruptions pendantes
    pub pending_interrupts: Vec<Interrupt>,
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
            interrupts_enabled: true,
            pending_interrupts: Vec::new(),
        }
    }

    /// Réinitialise le processeur à son état initial
    pub fn reset(&mut self) {
        self.registers.reset();
        self.decoder.clear_cache();
        self.cycle_count = 0;
        self.stats.reset();
        self.halted = false;
        self.interrupts_enabled = true;
        self.pending_interrupts.clear();
    }

    /// Exécute un cycle du processeur
    pub fn step<M>(&mut self, memory: &mut M) -> Result<u32>
    where
        M: crate::memory::MemoryInterface,
    {
        if self.halted {
            return Ok(1); // Un cycle minimal si arrêté
        }

        // Vérifier et traiter les interruptions pendantes
        if self.process_interrupts(memory)? {
            return Ok(10); // Cycles pour le traitement d'interruption
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
    
    /// Ajoute une interruption à la file d'attente
    pub fn queue_interrupt(&mut self, interrupt: Interrupt) {
        // Éviter les doublons
        if !self.pending_interrupts.contains(&interrupt) {
            self.pending_interrupts.push(interrupt);
        }
    }
    
    /// Traite les interruptions pendantes
    pub fn process_interrupts<M>(&mut self, memory: &mut M) -> Result<bool>
    where
        M: crate::memory::MemoryInterface,
    {
        if !self.interrupts_enabled || self.pending_interrupts.is_empty() {
            return Ok(false);
        }
        
        // Traiter la première interruption de la file
        if let Some(interrupt) = self.pending_interrupts.first().cloned() {
            self.handle_interrupt(interrupt, memory)?;
            self.pending_interrupts.remove(0);
            return Ok(true);
        }
        
        Ok(false)
    }
    
    /// Gère une interruption spécifique
    fn handle_interrupt<M>(&mut self, interrupt: Interrupt, memory: &mut M) -> Result<()>
    where
        M: crate::memory::MemoryInterface,
    {
        // Désactiver les interruptions pendant le traitement
        let interrupts_were_enabled = self.interrupts_enabled;
        self.interrupts_enabled = false;
        
        // Sauvegarder le PC et les flags sur la pile
        let pc = self.registers.pc;
        let flags = self.registers.psw.bits();
        
        // Empiler PC
        self.registers.sp = self.registers.sp.wrapping_sub(4);
        memory.write_u32(self.registers.sp, pc)?;
        
        // Empiler flags
        self.registers.sp = self.registers.sp.wrapping_sub(4);
        memory.write_u32(self.registers.sp, flags)?;
        
        // Charger l'adresse du gestionnaire d'interruption
        let handler_address = interrupt.vector_address();
        let handler = memory.read_u32(handler_address)?;
        
        // Sauter au gestionnaire
        self.registers.pc = handler;
        
        // Restaurer l'état des interruptions si elles étaient activées
        if interrupts_were_enabled {
            self.interrupts_enabled = true;
        }
        
        Ok(())
    }
    
    /// Retourne d'une interruption
    pub fn return_from_interrupt<M>(&mut self, memory: &mut M) -> Result<()>
    where
        M: crate::memory::MemoryInterface,
    {
        // Dépiler les flags
        let flags = memory.read_u32(self.registers.sp)?;
        self.registers.sp = self.registers.sp.wrapping_add(4);
        self.registers.psw = ProcessorStatusWord::from_bits_truncate(flags);
        
        // Dépiler le PC
        let pc = memory.read_u32(self.registers.sp)?;
        self.registers.sp = self.registers.sp.wrapping_add(4);
        self.registers.pc = pc;
        
        // Réactiver les interruptions
        self.interrupts_enabled = true;
        
        Ok(())
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