use pixel_model2_rust::cpu::*;

// Helper pour construire un mot 16 bits selon l'encodage du décodeur
fn make_word(opcode: u8, r2: u8, r1: u8) -> [u8; 2] {
    let word: u16 = ((opcode as u16) << 10) | ((r2 as u16) << 5) | (r1 as u16);
    word.to_le_bytes()
}

#[test]
fn test_instruction_decoder_format1() {
    let mut decoder = V60InstructionDecoder::new();

    // Construire un MOV Format1: opcode=0x00, r2=2, r1=1
    let instruction_data = make_word(0x00, 2, 1);

    let result = decoder.decode(&instruction_data, 0x1000).unwrap();

    assert_eq!(result.address, 0x1000);
    assert_eq!(result.size, 2);

    match result.instruction {
        Instruction::Mov { dest, src } => {
            assert_eq!(dest, Operand::Register(2));
            assert_eq!(src, Operand::Register(1));
        }
        _ => panic!(
            "Instruction décodée incorrectement: {:?}",
            result.instruction
        ),
    }
}

#[test]
fn test_instruction_decoder_format2() {
    let mut decoder = V60InstructionDecoder::new();
    // Test d'une instruction ADD Format 2: opcode=0x11 (Add imm selon decode_format2)
    // dest=r2=2, r1=1, immediate=100
    let mut instruction_data = Vec::new();
    instruction_data.extend_from_slice(&make_word(0x11, 2, 1));
    instruction_data.extend_from_slice(&100u16.to_le_bytes());

    let result = decoder.decode(&instruction_data, 0x2000).unwrap();

    assert_eq!(result.address, 0x2000);
    assert_eq!(result.size, 4);

    match result.instruction {
        Instruction::Add { dest, src1, src2 } => {
            // Note: decode_format2 encodera src1 = dest (selon implémentation actuelle)
            assert_eq!(dest, Operand::Register(2));
            assert_eq!(src1, Operand::Register(2));
            assert_eq!(src2, Operand::Immediate(100));
        }
        _ => panic!(
            "Instruction décodée incorrectement: {:?}",
            result.instruction
        ),
    }
}

#[test]
fn test_instruction_decoder_format4_branch() {
    let mut decoder = V60InstructionDecoder::new();
    // Construire un Jump Format4: opcode=0x30 (Jump)
    let mut instruction_data = Vec::new();
    instruction_data.extend_from_slice(&make_word(0x30, 0, 0));
    // ajouter un déplacement pseudo (les détails d'encodage sont gérés par le décodeur)
    instruction_data.extend_from_slice(&200u16.to_le_bytes());

    let result = decoder.decode(&instruction_data, 0x3000).unwrap();

    assert_eq!(result.address, 0x3000);
    assert_eq!(result.size, 4);

    match result.instruction {
        Instruction::Jump { .. } => {
            // On vérifie seulement le type ici (le décodage précis du déplacement dépend de la
            // représentation interne et est testé ailleurs)
        }
        _ => panic!(
            "Instruction décodée incorrectement: {:?}",
            result.instruction
        ),
    }
}

#[test]
fn test_instruction_decoder_format5_system() {
    let mut decoder = V60InstructionDecoder::new();
    // Test NOP Format5: opcode dans la plage 0x38..0x3F (fonction 0)
    let instruction_data = make_word(0x38, 0, 0);

    let result = decoder.decode(&instruction_data, 0x4000).unwrap();

    assert_eq!(result.address, 0x4000);
    assert_eq!(result.size, 2);

    match result.instruction {
        Instruction::Nop => {}
        _ => panic!(
            "Instruction décodée incorretement: {:?}",
            result.instruction
        ),
    }
}

#[test]
fn test_instruction_cache() {
    let mut decoder = V60InstructionDecoder::new();

    let instruction_data = make_word(0x38, 0, 0); // NOP

    // Première fois - décodage normal
    let result1 = decoder.decode(&instruction_data, 0x5000).unwrap();

    // Deuxième fois - devrait venir du cache
    let result2 = decoder.decode(&instruction_data, 0x5000).unwrap();

    assert_eq!(result1.address, result2.address);
    assert_eq!(result1.size, result2.size);

    // Vérifier que les instructions sont identiques
    match (&result1.instruction, &result2.instruction) {
        (Instruction::Nop, Instruction::Nop) => {}
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
        // Format 1: SUB R1, R0 (selon implémentation: src1 = dest)
        (
            make_word(0x02, 1, 0),
            Instruction::Sub {
                dest: Operand::Register(1),
                src1: Operand::Register(1),
                src2: Operand::Register(0),
            },
        ),
        // Format 1: ADD R2, R0 (opcode=0x01)
        (
            make_word(0x01, 2, 0),
            Instruction::Add {
                dest: Operand::Register(2),
                src1: Operand::Register(2),
                src2: Operand::Register(0),
            },
        ),
    ];

    for (i, (data, expected)) in test_cases.iter().enumerate() {
        let result = decoder.decode(data, 0x6000 + i as u32 * 0x10);
        assert!(result.is_ok(), "Test case {} failed to decode", i);

        let decoded = result.unwrap();
        match (&decoded.instruction, expected) {
            (
                Instruction::Sub {
                    dest: d1,
                    src1: s1,
                    src2: s2,
                },
                Instruction::Sub {
                    dest: d2,
                    src1: s3,
                    src2: s4,
                },
            ) => {
                assert_eq!(d1, d2);
                assert_eq!(s1, s3);
                assert_eq!(s2, s4);
            }
            (
                Instruction::Add {
                    dest: d1,
                    src1: s1,
                    src2: s2,
                },
                Instruction::Add {
                    dest: d2,
                    src1: s3,
                    src2: s4,
                },
            ) => {
                assert_eq!(d1, d2);
                assert_eq!(s1, s3);
                assert_eq!(s2, s4);
            }
            (Instruction::Mov { dest: d1, src: s1 }, Instruction::Mov { dest: d2, src: s2 }) => {
                assert_eq!(d1, d2);
                assert_eq!(s1, s2);
            }
            _ => panic!("Test case {} instruction mismatch", i),
        }
    }
}
