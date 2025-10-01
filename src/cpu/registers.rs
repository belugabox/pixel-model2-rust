//! Registres du processeur NEC V60

use bitflags::bitflags;

/// Structure contenant tous les registres du NEC V60
#[derive(Debug, Clone)]
pub struct V60Registers {
    /// Registres généraux (R0-R31)
    pub general: [u32; 32],
    
    /// Program Counter (compteur de programme)
    pub pc: u32,
    
    /// Stack Pointer (pointeur de pile)
    pub sp: u32,
    
    /// Frame Pointer (pointeur de frame)
    pub fp: u32,
    
    /// Processor Status Word (mot d'état du processeur)
    pub psw: ProcessorStatusWord,
    
    /// Registres de contrôle système
    pub control: [u32; 16],
}

impl V60Registers {
    /// Crée une nouvelle instance des registres avec des valeurs par défaut
    pub fn new() -> Self {
        Self {
            general: [0; 32],
            pc: 0x0000_0000, // Adresse de démarrage
            sp: 0x0000_0000, // Sera initialisée par le système
            fp: 0x0000_0000,
            psw: ProcessorStatusWord::empty(),
            control: [0; 16],
        }
    }

    /// Réinitialise tous les registres à leur valeur par défaut
    pub fn reset(&mut self) {
        self.general.fill(0);
        self.pc = 0x0000_0000;
        self.sp = 0x0000_0000;
        self.fp = 0x0000_0000;
        self.psw = ProcessorStatusWord::empty();
        self.control.fill(0);
    }

    /// Lit un registre général par son index
    pub fn read_general(&self, index: usize) -> u32 {
        if index < 32 {
            self.general[index]
        } else {
            0 // Registre invalide retourne 0
        }
    }

    /// Écrit dans un registre général par son index
    pub fn write_general(&mut self, index: usize, value: u32) {
        if index < 32 && index != 0 {
            // R0 est toujours 0 sur certains processeurs RISC,
            // mais le V60 permet d'écrire dans R0
            self.general[index] = value;
        }
    }

    /// Lit un registre de contrôle par son index
    pub fn read_control(&self, index: usize) -> u32 {
        if index < 16 {
            self.control[index]
        } else {
            0
        }
    }

    /// Écrit dans un registre de contrôle par son index
    pub fn write_control(&mut self, index: usize, value: u32) {
        if index < 16 {
            self.control[index] = value;
        }
    }
    
    /// Alias pour read_general - lecture d'un GPR (General Purpose Register)
    pub fn get_gpr(&self, index: usize) -> u32 {
        self.read_general(index)
    }
    
    /// Alias pour write_general - écriture d'un GPR
    pub fn set_gpr(&mut self, index: usize, value: u32) {
        self.write_general(index, value);
    }
    
    /// Lit le compteur de programme (PC)
    pub fn get_pc(&self) -> u32 {
        self.pc
    }
    
    /// Écrit le compteur de programme (PC)
    pub fn set_pc(&mut self, value: u32) {
        self.pc = value;
    }
}

impl Default for V60Registers {
    fn default() -> Self {
        Self::new()
    }
}

bitflags! {
    /// Processor Status Word - contient les flags de condition et d'état
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ProcessorStatusWord: u32 {
        /// Carry flag - résultat d'une opération génère une retenue
        const CARRY = 1 << 0;
        
        /// Zero flag - résultat d'une opération est zéro
        const ZERO = 1 << 1;
        
        /// Sign flag - résultat d'une opération est négatif
        const SIGN = 1 << 2;
        
        /// Overflow flag - débordement arithmétique
        const OVERFLOW = 1 << 3;
        
        /// Interrupt enable - autorise les interruptions
        const INTERRUPT_ENABLE = 1 << 8;
        
        /// Supervisor mode - mode superviseur activé
        const SUPERVISOR = 1 << 15;
        
        /// Parity flag - parité du résultat (nombre pair de bits à 1)
        const PARITY = 1 << 4;
        
        /// Debug mode - mode débogage activé
        const DEBUG = 1 << 16;
    }
}

impl ProcessorStatusWord {
    /// Met à jour les flags de condition basés sur un résultat
    pub fn update_flags(&mut self, result: u32, carry: bool, overflow: bool) {
        // Clear existing condition flags
        self.remove(Self::CARRY | Self::ZERO | Self::SIGN | Self::OVERFLOW);
        
        // Set flags based on result
        if carry {
            self.insert(Self::CARRY);
        }
        
        if result == 0 {
            self.insert(Self::ZERO);
        }
        
        if (result as i32) < 0 {
            self.insert(Self::SIGN);
        }
        
        if overflow {
            self.insert(Self::OVERFLOW);
        }
    }

    /// Méthodes individuelles pour les flags (pour compatibility avec ArithmeticResult)
    pub fn set_zero_flag(&mut self, value: bool) {
        if value { self.insert(Self::ZERO); } else { self.remove(Self::ZERO); }
    }
    
    pub fn set_negative_flag(&mut self, value: bool) {
        if value { self.insert(Self::SIGN); } else { self.remove(Self::SIGN); }
    }
    
    pub fn set_carry_flag(&mut self, value: bool) {
        if value { self.insert(Self::CARRY); } else { self.remove(Self::CARRY); }
    }
    
    pub fn set_overflow_flag(&mut self, value: bool) {
        if value { self.insert(Self::OVERFLOW); } else { self.remove(Self::OVERFLOW); }
    }
    
    pub fn set_parity_flag(&mut self, value: bool) {
        if value { self.insert(Self::PARITY); } else { self.remove(Self::PARITY); }
    }

    /// Vérifie si une condition est vraie basée sur les flags
    pub fn condition_met(&self, condition: ConditionCode) -> bool {
        match condition {
            ConditionCode::Always => true,
            ConditionCode::Never => false,
            ConditionCode::Equal => self.contains(Self::ZERO),
            ConditionCode::NotEqual => !self.contains(Self::ZERO),
            ConditionCode::Carry => self.contains(Self::CARRY),
            ConditionCode::NotCarry => !self.contains(Self::CARRY),
            ConditionCode::Negative => self.contains(Self::SIGN),
            ConditionCode::Positive => !self.contains(Self::SIGN),
            ConditionCode::Overflow => self.contains(Self::OVERFLOW),
            ConditionCode::NotOverflow => !self.contains(Self::OVERFLOW),
        }
    }
}

/// Codes de condition pour les instructions conditionnelles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionCode {
    Always,
    Never,
    Equal,
    NotEqual,
    Carry,
    NotCarry,
    Negative,
    Positive,
    Overflow,
    NotOverflow,
}