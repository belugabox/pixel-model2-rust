//! Tests d'exécution d'instructions pour le processeur NEC V60

use pixel_model2_rust::cpu::*;
use pixel_model2_rust::memory::*;

/// Mock simple de mémoire pour les tests
struct TestMemory {
    data: std::collections::HashMap<u32, u8>,
}

impl TestMemory {
    fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }

    fn write_word(&mut self, address: u32, value: u32) {
        let bytes = value.to_le_bytes();
        for (i, byte) in bytes.iter().enumerate() {
            self.data.insert(address + i as u32, *byte);
        }
    }
}

impl MemoryInterface for TestMemory {
    fn read_u8(&self, address: u32) -> anyhow::Result<u8> {
        Ok(self.data.get(&address).copied().unwrap_or(0))
    }

    fn read_u16(&self, address: u32) -> anyhow::Result<u16> {
        let low = self.read_u8(address)? as u16;
        let high = self.read_u8(address + 1)? as u16;
        Ok(low | (high << 8))
    }

    fn read_u32(&self, address: u32) -> anyhow::Result<u32> {
        let mut bytes = [0u8; 4];
        for i in 0..4 {
            bytes[i] = self.read_u8(address + i as u32)?;
        }
        Ok(u32::from_le_bytes(bytes))
    }

    fn write_u8(&mut self, address: u32, value: u8) -> anyhow::Result<()> {
        self.data.insert(address, value);
        Ok(())
    }

    fn write_u16(&mut self, address: u32, value: u16) -> anyhow::Result<()> {
        let bytes = value.to_le_bytes();
        self.write_u8(address, bytes[0])?;
        self.write_u8(address + 1, bytes[1])?;
        Ok(())
    }

    fn write_u32(&mut self, address: u32, value: u32) -> anyhow::Result<()> {
        let bytes = value.to_le_bytes();
        for (i, byte) in bytes.iter().enumerate() {
            self.write_u8(address + i as u32, *byte)?;
        }
        Ok(())
    }
}

#[test]
fn test_arithmetic_instruction_execution() {
    let mut cpu = NecV60::new();
    let mut memory = TestMemory::new();

    // Test ADD R1, R0, R2  (R1 = R0 + R2)
    cpu.registers.write_general(0, 10); // R0 = 10
    cpu.registers.write_general(2, 20); // R2 = 20

    let instruction = DecodedInstruction {
        address: 0x1000,
        instruction: Instruction::Add {
            dest: Operand::Register(1),
            src1: Operand::Register(0),
            src2: Operand::Register(2),
        },
        size: 2,
        cycles: 1,
    };

    let result = cpu.execute_instruction(&instruction, &mut memory);
    assert!(result.is_ok());

    // Vérifier le résultat
    assert_eq!(cpu.registers.read_general(1), 30); // R1 devrait être 10 + 20 = 30
    assert_eq!(cpu.registers.pc, 0x1000 + 2); // PC devrait avancer
    assert!(!cpu.registers.psw.contains(ProcessorStatusWord::ZERO));
    assert!(!cpu.registers.psw.contains(ProcessorStatusWord::CARRY));
    assert!(!cpu.registers.psw.contains(ProcessorStatusWord::OVERFLOW));

    // Vérifier les statistiques
    assert_eq!(cpu.stats.instructions_executed, 1);
    assert_eq!(cpu.stats.cycles_executed, 1);
}

#[test]
fn test_arithmetic_overflow() {
    let mut cpu = NecV60::new();
    let mut memory = TestMemory::new();

    // Test overflow avec addition
    cpu.registers.write_general(0, u32::MAX); // R0 = MAX
    cpu.registers.write_general(2, 1); // R2 = 1

    let instruction = DecodedInstruction {
        address: 0x1000,
        instruction: Instruction::Add {
            dest: Operand::Register(1),
            src1: Operand::Register(0),
            src2: Operand::Register(2),
        },
        size: 2,
        cycles: 1,
    };

    let result = cpu.execute_instruction(&instruction, &mut memory);
    assert!(result.is_ok());

    // Vérifier le débordement
    assert_eq!(cpu.registers.read_general(1), 0); // Débordement vers 0
    assert!(cpu.registers.psw.contains(ProcessorStatusWord::ZERO));
    assert!(cpu.registers.psw.contains(ProcessorStatusWord::CARRY));
    assert_eq!(cpu.stats.exceptions_raised, 1); // Exception comptée
}

#[test]
fn test_logical_instruction_execution() {
    let mut cpu = NecV60::new();
    let mut memory = TestMemory::new();

    // Test AND R1, R0, R2  (R1 = R0 & R2)
    cpu.registers.write_general(0, 0xFF00FF00); // R0
    cpu.registers.write_general(2, 0x00FF00FF); // R2

    let instruction = DecodedInstruction {
        address: 0x2000,
        instruction: Instruction::And {
            dest: Operand::Register(1),
            src1: Operand::Register(0),
            src2: Operand::Register(2),
        },
        size: 2,
        cycles: 1,
    };

    let result = cpu.execute_instruction(&instruction, &mut memory);
    assert!(result.is_ok());

    // Vérifier le résultat AND
    assert_eq!(cpu.registers.read_general(1), 0x00000000); // Résultat AND
    assert!(cpu.registers.psw.contains(ProcessorStatusWord::ZERO));
    assert!(cpu.registers.psw.contains(ProcessorStatusWord::PARITY)); // Parité paire (0 bits à 1)
}

#[test]
fn test_shift_instruction_execution() {
    let mut cpu = NecV60::new();
    let mut memory = TestMemory::new();

    // Test SHL R1, R0, #4  (R1 = R0 << 4)
    cpu.registers.write_general(0, 0x12345678);

    let instruction = DecodedInstruction {
        address: 0x3000,
        instruction: Instruction::Shl {
            dest: Operand::Register(1),
            src: Operand::Register(0),
            shift: Operand::Immediate(4),
        },
        size: 2,
        cycles: 1,
    };

    let result = cpu.execute_instruction(&instruction, &mut memory);
    assert!(result.is_ok());

    // Vérifier le décalage
    assert_eq!(cpu.registers.read_general(1), 0x23456780); // Décalé de 4 bits
    assert!(cpu.registers.psw.contains(ProcessorStatusWord::CARRY)); // Bit 28 était à 1
}

#[test]
fn test_division_by_zero() {
    let mut cpu = NecV60::new();
    let mut memory = TestMemory::new();

    // Test DIV avec division par zéro
    cpu.registers.write_general(0, 42); // R0 = 42
    cpu.registers.write_general(2, 0); // R2 = 0

    let instruction = DecodedInstruction {
        address: 0x4000,
        instruction: Instruction::Div {
            dest: Operand::Register(1),
            src1: Operand::Register(0),
            src2: Operand::Register(2),
        },
        size: 2,
        cycles: 1,
    };

    let result = cpu.execute_instruction(&instruction, &mut memory);
    assert!(result.is_err()); // Doit générer une erreur
    assert_eq!(cpu.stats.exceptions_raised, 1); // Exception comptée
}

#[test]
fn test_memory_operations() {
    let mut cpu = NecV60::new();
    let mut memory = TestMemory::new();

    // Préparer la mémoire
    memory.write_word(0x5000, 0xDEADBEEF);

    // Test LOAD R1, [0x5000]
    let instruction = DecodedInstruction {
        address: 0x6000,
        instruction: Instruction::Load {
            dest: Operand::Register(1),
            address: Operand::Direct(0x5000),
            size: DataSize::DWord,
        },
        size: 6,
        cycles: 3,
    };

    let result = cpu.execute_instruction(&instruction, &mut memory);
    assert!(result.is_ok());

    // Vérifier le chargement
    assert_eq!(cpu.registers.read_general(1), 0xDEADBEEF);
    assert_eq!(cpu.stats.memory_accesses, 1);

    // Test STORE [0x6000], R1
    let store_instruction = DecodedInstruction {
        address: 0x6006,
        instruction: Instruction::Store {
            src: Operand::Register(1),
            address: Operand::Direct(0x6000),
            size: DataSize::DWord,
        },
        size: 6,
        cycles: 3,
    };

    let result = cpu.execute_instruction(&store_instruction, &mut memory);
    assert!(result.is_ok());

    // Vérifier le stockage
    assert_eq!(memory.read_u32(0x6000).unwrap(), 0xDEADBEEF);
    assert_eq!(cpu.stats.memory_accesses, 2); // Une lecture + une écriture
}

#[test]
fn test_branch_instructions() {
    let mut cpu = NecV60::new();
    let mut memory = TestMemory::new();

    // Test JUMP absolu
    let instruction = DecodedInstruction {
        address: 0x7000,
        instruction: Instruction::Jump {
            target: Operand::Direct(0x8000),
        },
        size: 4,
        cycles: 2,
    };

    let result = cpu.execute_instruction(&instruction, &mut memory);
    assert!(result.is_ok());

    // Vérifier le saut
    assert_eq!(cpu.registers.pc, 0x8000);
    assert_eq!(cpu.stats.branches_taken, 1);
}

#[test]
fn test_conditional_branch() {
    let mut cpu = NecV60::new();
    let mut memory = TestMemory::new();

    // Mettre le flag ZERO pour tester une branche conditionnelle
    cpu.registers.psw.insert(ProcessorStatusWord::ZERO);

    // Test branche conditionnelle qui devrait être prise
    let instruction = DecodedInstruction {
        address: 0x9000,
        instruction: Instruction::JumpConditional {
            condition: ConditionCode::Equal, // Teste le flag ZERO
            target: Operand::Direct(0xA000),
        },
        size: 4,
        cycles: 2,
    };

    let result = cpu.execute_instruction(&instruction, &mut memory);
    assert!(result.is_ok());

    // Vérifier que la branche a été prise
    assert_eq!(cpu.registers.pc, 0xA000);
    assert_eq!(cpu.stats.branches_taken, 1);
}

#[test]
fn test_performance_statistics() {
    let mut cpu = NecV60::new();
    let mut memory = TestMemory::new();

    // Exécuter plusieurs instructions pour tester les statistiques
    let instructions = vec![
        DecodedInstruction {
            address: 0x1000,
            instruction: Instruction::Add {
                dest: Operand::Register(1),
                src1: Operand::Register(0),
                src2: Operand::Register(2),
            },
            size: 2,
            cycles: 1,
        },
        DecodedInstruction {
            address: 0x1002,
            instruction: Instruction::Mov {
                dest: Operand::Register(3),
                src: Operand::Immediate(42),
            },
            size: 4,
            cycles: 1,
        },
        DecodedInstruction {
            address: 0x1006,
            instruction: Instruction::Jump {
                target: Operand::Direct(0x2000),
            },
            size: 4,
            cycles: 2,
        },
    ];

    for instruction in instructions {
        let _ = cpu.execute_instruction(&instruction, &mut memory);
    }

    // Vérifier les statistiques
    assert_eq!(cpu.stats.instructions_executed, 3);
    assert_eq!(cpu.stats.cycles_executed, 4); // 1+1+2
    assert_eq!(cpu.stats.branches_taken, 1);
}
