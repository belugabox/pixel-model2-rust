use pixel_model2_rust::cpu::*;

#[test]
fn test_instruction_decoder_format1() {
    let mut decoder = V60InstructionDecoder::new();
    
    // Test d'une instruction ADD Format 1: ADD R2, R1, R0
    // opcode=0x0, r2=2, r1=1, mode=0
    let instruction_data = [0x10, 0x02]; // 0x0210 en little endian
    
    let result = decoder.decode(&instruction_data, 0x1000).unwrap();
    
    assert_eq!(result.address, 0x1000);
    assert_eq!(result.size, 2);
    
    match result.instruction {
        Instruction::Add { dest, src1, src2 } => {
            assert_eq!(dest, Operand::Register(2));
            assert_eq!(src1, Operand::Register(1));
            assert_eq!(src2, Operand::Register(1));
        },
        _ => panic!("Instruction décodée incorrectement: {:?}", result.instruction),
    }
}

#[test]  
fn test_instruction_decoder_format2() {
    let mut decoder = V60InstructionDecoder::new();
    
    // Test d'une instruction ADD Format 2: ADD R2, R1, #100
    // opcode=0x0, r2=2, r1=1, mode=1, immediate=100
    let instruction_data = [0x11, 0x02, 0x64, 0x00]; // 0x0211, 0x0064 en little endian
    
    let result = decoder.decode(&instruction_data, 0x2000).unwrap();
    
    assert_eq!(result.address, 0x2000);
    assert_eq!(result.size, 4);
    
    match result.instruction {
        Instruction::Add { dest, src1, src2 } => {
            assert_eq!(dest, Operand::Register(2));
            assert_eq!(src1, Operand::Register(1));
            assert_eq!(src2, Operand::Immediate(100));
        },
        _ => panic!("Instruction décodée incorrectement: {:?}", result.instruction),
    }
}

#[test]
fn test_instruction_decoder_format4_branch() {
    let mut decoder = V60InstructionDecoder::new();
    
    // Test d'une instruction JUMP Format 4: JMP +200
    // opcode=0x8, condition=0, displacement=200
    let instruction_data = [0xC8, 0x80, 0x00, 0x00]; // 0x80C8, 0x0000 en little endian pour displacement=200
    
    let result = decoder.decode(&instruction_data, 0x3000).unwrap();
    
    assert_eq!(result.address, 0x3000);
    assert_eq!(result.size, 4);
    
    match result.instruction {
        Instruction::Jump { target } => {
            assert_eq!(target, Operand::PcRelative(200));
        },
        _ => panic!("Instruction décodée incorrectement: {:?}", result.instruction),
    }
}

#[test]
fn test_instruction_decoder_format5_system() {
    let mut decoder = V60InstructionDecoder::new();
    
    // Test d'une instruction NOP Format 5
    // opcode=0xC, function=0x0
    let instruction_data = [0x00, 0xC0]; // 0xC000 en little endian
    
    let result = decoder.decode(&instruction_data, 0x4000).unwrap();
    
    assert_eq!(result.address, 0x4000);
    assert_eq!(result.size, 2);
    
    match result.instruction {
        Instruction::Nop => {},
        _ => panic!("Instruction décodée incorretement: {:?}", result.instruction),
    }
}

#[test]
fn test_instruction_cache() {
    let mut decoder = V60InstructionDecoder::new();
    
    let instruction_data = [0x00, 0xC0]; // NOP
    
    // Première fois - décodage normal
    let result1 = decoder.decode(&instruction_data, 0x5000).unwrap();
    
    // Deuxième fois - devrait venir du cache
    let result2 = decoder.decode(&instruction_data, 0x5000).unwrap();
    
    assert_eq!(result1.address, result2.address);
    assert_eq!(result1.size, result2.size);
    
    // Vérifier que les instructions sont identiques
    match (&result1.instruction, &result2.instruction) {
        (Instruction::Nop, Instruction::Nop) => {},
        _ => panic!("Les instructions du cache ne correspondent pas"),
    }
}

#[test]
fn test_cpu_with_new_decoder() {
    let mut cpu = NecV60::new();
    
    // Test de l'initialisation du CPU avec le nouveau décodeur
    assert_eq!(cpu.cycle_count, 0);
    assert!(!cpu.halted);
    
    // Test que le décodeur est bien initialisé
    cpu.decoder.clear_cache();
    
    // Test de basic register operations
    cpu.registers.set_gpr(1, 0x12345678);
    assert_eq!(cpu.registers.get_gpr(1), 0x12345678);
    
    cpu.registers.set_pc(0x1000);
    assert_eq!(cpu.registers.get_pc(), 0x1000);
}

#[test]
fn test_operand_extraction() {
    let mut decoder = V60InstructionDecoder::new();
    
    // Test avec différents modes d'adressage  
    let test_cases = vec![
        // Format 1: SUB R1, R0, R0  (opcode=1, r2=1, r1=0, mode=0)
        ([0x00, 0x11], Instruction::Sub { 
            dest: Operand::Register(1), 
            src1: Operand::Register(0),
            src2: Operand::Register(0)
        }),
        // Format 2: ADD R2, R0, R1  (opcode=0, r2=2, r1=0, mode=0) 
        ([0x00, 0x02], Instruction::Add {
            dest: Operand::Register(2),
            src1: Operand::Register(0),
            src2: Operand::Register(0)
        }),
    ];
    
    for (i, (data, expected)) in test_cases.iter().enumerate() {
        let result = decoder.decode(data, 0x6000 + i as u32 * 0x10);
        assert!(result.is_ok(), "Test case {} failed to decode", i);
        
        let decoded = result.unwrap();
        match (&decoded.instruction, expected) {
            (Instruction::Sub { dest: d1, src1: s1, src2: s2 }, 
             Instruction::Sub { dest: d2, src1: s3, src2: s4 }) => {
                assert_eq!(d1, d2);
                assert_eq!(s1, s3); 
                assert_eq!(s2, s4);
            },
            (Instruction::Add { dest: d1, src1: s1, src2: s2 }, 
             Instruction::Add { dest: d2, src1: s3, src2: s4 }) => {
                assert_eq!(d1, d2);
                assert_eq!(s1, s3); 
                assert_eq!(s2, s4);
            },
            (Instruction::Mov { dest: d1, src: s1 }, 
             Instruction::Mov { dest: d2, src: s2 }) => {
                assert_eq!(d1, d2);
                assert_eq!(s1, s2);
            },
            _ => panic!("Test case {} instruction mismatch", i),
        }
    }
}