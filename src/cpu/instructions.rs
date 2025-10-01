//! Instructions du processeur NEC V60

/// Types d'instructions supportées par le NEC V60
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    // Instructions arithmétiques
    Add { dest: Operand, src1: Operand, src2: Operand },
    Sub { dest: Operand, src1: Operand, src2: Operand },
    Mul { dest: Operand, src1: Operand, src2: Operand },
    Div { dest: Operand, src1: Operand, src2: Operand },
    
    // Instructions logiques
    And { dest: Operand, src1: Operand, src2: Operand },
    Or { dest: Operand, src1: Operand, src2: Operand },
    Xor { dest: Operand, src1: Operand, src2: Operand },
    Not { dest: Operand, src: Operand },
    
    // Instructions de déplacement
    Shl { dest: Operand, src: Operand, shift: Operand },
    Shr { dest: Operand, src: Operand, shift: Operand },
    
    // Instructions de transfert
    Mov { dest: Operand, src: Operand },
    Load { dest: Operand, address: Operand, size: DataSize },
    Store { src: Operand, address: Operand, size: DataSize },
    
    // Instructions de branchement
    Jump { target: Operand },
    JumpConditional { condition: super::registers::ConditionCode, target: Operand },
    Call { target: Operand },
    Return,
    
    // Instructions de comparaison
    Compare { src1: Operand, src2: Operand },
    Test { src1: Operand, src2: Operand },
    
    // Instructions système
    Nop,
    Halt,
    InterruptReturn,
    
    // Instructions spéciales V60
    FloatAdd { dest: Operand, src1: Operand, src2: Operand },
    FloatMul { dest: Operand, src1: Operand, src2: Operand },
    FloatSub { dest: Operand, src1: Operand, src2: Operand },
    FloatDiv { dest: Operand, src1: Operand, src2: Operand },
    FloatCompare { src1: Operand, src2: Operand },
    
    // Instructions de rotation
    RotateLeft { dest: Operand, src: Operand, count: Operand },
    RotateRight { dest: Operand, src: Operand, count: Operand },
    
    // Instructions de manipulation de bits
    BitTest { src: Operand, bit: Operand },
    BitSet { dest: Operand, bit: Operand },
    BitClear { dest: Operand, bit: Operand },
    BitScan { dest: Operand, src: Operand },
    
    // Instructions de pile
    Push { src: Operand },
    Pop { dest: Operand },
    PushMultiple { registers: Vec<usize> },
    PopMultiple { registers: Vec<usize> },
    
    // Instructions de chaîne (string operations)
    StringMove { size: DataSize },
    StringCompare { size: DataSize },
    StringScan { size: DataSize },
    
    // Instructions MMU et système avancées
    LoadControlRegister { dest: Operand, control_reg: u8 },
    StoreControlRegister { src: Operand, control_reg: u8 },
    InvalidateTLB,
    FlushCache,
    
    // Instructions d'interruption
    SoftwareInterrupt { vector: u8 },
    ReturnFromInterrupt,
    EnableInterrupts,
    DisableInterrupts,
    
    // Instructions de synchronization
    TestAndSet { dest: Operand, src: Operand },
    CompareAndSwap { dest: Operand, compare: Operand, new_value: Operand },
    
    // Instructions BCD (Binary Coded Decimal)
    BcdAdd { dest: Operand, src1: Operand, src2: Operand },
    BcdSub { dest: Operand, src1: Operand, src2: Operand },
    
    // Instruction inconnue/non implémentée
    Unknown { opcode: u32 },
}

/// Opérandes des instructions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operand {
    /// Registre général (R0-R31)
    Register(usize),
    
    /// Valeur immédiate
    Immediate(u32),
    
    /// Adresse mémoire directe
    Direct(u32),
    
    /// Adresse indirecte via registre
    Indirect(usize),
    
    /// Adresse indirecte avec décalage
    IndirectOffset(usize, i32),
    
    /// Adresse indirecte avec index
    IndirectIndexed(usize, usize, u32), // base, index, scale
    
    /// Adresse relative au PC
    PcRelative(i32),
}

/// Tailles de données supportées
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataSize {
    Byte,    // 8 bits
    Word,    // 16 bits  
    DWord,   // 32 bits
}

impl DataSize {
    /// Retourne la taille en octets
    pub fn bytes(self) -> usize {
        match self {
            DataSize::Byte => 1,
            DataSize::Word => 2,
            DataSize::DWord => 4,
        }
    }
    
    /// Retourne la taille en bits
    pub fn bits(self) -> usize {
        self.bytes() * 8
    }
}

/// Instruction décodée avec métadonnées
#[derive(Debug, Clone)]
pub struct DecodedInstruction {
    /// L'instruction elle-même
    pub instruction: Instruction,
    
    /// Adresse de l'instruction
    pub address: u32,
    
    /// Taille de l'instruction en octets
    pub size: u32,
    
    /// Nombre de cycles estimé pour l'exécution
    pub cycles: u32,
}

impl DecodedInstruction {
    /// Crée une nouvelle instruction décodée
    pub fn new(instruction: Instruction, address: u32, size: u32) -> Self {
        let cycles = estimate_cycles(&instruction);
        Self {
            instruction,
            address,
            size,
            cycles,
        }
    }
}

/// Estime le nombre de cycles nécessaires pour une instruction
fn estimate_cycles(instruction: &Instruction) -> u32 {
    match instruction {
        // Instructions simples - 1 cycle
        Instruction::Nop | 
        Instruction::Mov { .. } => 1,
        
        // Instructions arithmétiques simples - 2 cycles
        Instruction::Add { .. } | 
        Instruction::Sub { .. } |
        Instruction::And { .. } |
        Instruction::Or { .. } |
        Instruction::Xor { .. } |
        Instruction::Not { .. } => 2,
        
        // Instructions de déplacement - 3 cycles
        Instruction::Shl { .. } |
        Instruction::Shr { .. } |
        Instruction::RotateLeft { .. } |
        Instruction::RotateRight { .. } => 3,
        
        // Multiplication - 10 cycles
        Instruction::Mul { .. } => 10,
        
        // Division - 20 cycles
        Instruction::Div { .. } => 20,
        
        // Accès mémoire - 3 cycles
        Instruction::Load { .. } |
        Instruction::Store { .. } => 3,
        
        // Branchements - 2 cycles si pas pris, 4 si pris
        Instruction::Jump { .. } |
        Instruction::JumpConditional { .. } => 4,
        
        // Appels et retours - 5 cycles
        Instruction::Call { .. } |
        Instruction::Return => 5,
        
        // Instructions de comparaison - 2 cycles
        Instruction::Compare { .. } |
        Instruction::Test { .. } => 2,
        
        // Instructions flottantes - 8-12 cycles
        Instruction::FloatAdd { .. } |
        Instruction::FloatSub { .. } => 8,
        Instruction::FloatMul { .. } => 10,
        Instruction::FloatDiv { .. } => 15,
        Instruction::FloatCompare { .. } => 6,
        
        // Instructions de manipulation de bits - 2-4 cycles
        Instruction::BitTest { .. } |
        Instruction::BitSet { .. } |
        Instruction::BitClear { .. } => 2,
        Instruction::BitScan { .. } => 4,
        
        // Instructions de pile - 2-5 cycles
        Instruction::Push { .. } |
        Instruction::Pop { .. } => 2,
        Instruction::PushMultiple { registers } => 2 + registers.len() as u32,
        Instruction::PopMultiple { registers } => 2 + registers.len() as u32,
        
        // Instructions de chaîne - 5-15 cycles selon la taille
        Instruction::StringMove { .. } => 8,
        Instruction::StringCompare { .. } => 10,
        Instruction::StringScan { .. } => 12,
        
        // Instructions MMU et système - cycles élevés
        Instruction::LoadControlRegister { .. } |
        Instruction::StoreControlRegister { .. } => 15,
        Instruction::InvalidateTLB => 25,
        Instruction::FlushCache => 50,
        
        // Instructions d'interruption - cycles variables
        Instruction::SoftwareInterrupt { .. } => 20,
        Instruction::ReturnFromInterrupt => 15,
        Instruction::EnableInterrupts |
        Instruction::DisableInterrupts => 3,
        
        // Instructions de synchronisation - cycles élevés
        Instruction::TestAndSet { .. } => 8,
        Instruction::CompareAndSwap { .. } => 12,
        
        // Instructions BCD - cycles moyens
        Instruction::BcdAdd { .. } |
        Instruction::BcdSub { .. } => 6,
        
        // Instructions système
        Instruction::Halt => 1,
        Instruction::InterruptReturn => 10,
        
        // Instruction inconnue - 1 cycle par défaut
        Instruction::Unknown { .. } => 1,
    }
}