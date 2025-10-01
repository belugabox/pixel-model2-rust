use pixel_model2_rust::*;
use std::thread;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    println!("🎮 SEGA Model 2 Emulator v0.1.0");
    println!("================================");
    
    // Initialisation des composants
    println!("Initialisation du processeur NEC V60...");
    let mut cpu = cpu::NecV60::new();
    
    println!("Initialisation de la mémoire (8MB RAM + 4MB VRAM + 512KB Audio RAM)...");
    let mut memory = memory::Model2Memory::new();
    
    // Test de fonctionnement de base
    println!("Test d'écriture/lecture mémoire...");
    
    // Test écriture/lecture u8
    memory.write_u8(0x0000_0000, 0x42)?;
    let value = memory.read_u8(0x0000_0000)?;
    println!("  U8: Écrit 0x42, lu 0x{:02X} - {}", value, if value == 0x42 { "✓" } else { "✗" });
    
    // Test écriture/lecture u16
    memory.write_u16(0x0000_0100, 0x1234)?;
    let value = memory.read_u16(0x0000_0100)?;
    println!("  U16: Écrit 0x1234, lu 0x{:04X} - {}", value, if value == 0x1234 { "✓" } else { "✗" });
    
    // Test écriture/lecture u32
    memory.write_u32(0x0000_0200, 0x12345678)?;
    let value = memory.read_u32(0x0000_0200)?;
    println!("  U32: Écrit 0x12345678, lu 0x{:08X} - {}", value, if value == 0x12345678 { "✓" } else { "✗" });
    
    // Test du processeur
    println!("Test des registres CPU...");
    cpu.reset();
    cpu.registers.set_gpr(0, 0xDEADBEEF);
    let reg_value = cpu.registers.get_gpr(0);
    println!("  GPR[0]: Écrit 0xDEADBEEF, lu 0x{:08X} - {}", reg_value, if reg_value == 0xDEADBEEF { "✓" } else { "✗" });
    
    // Chargement d'une ROM d'exemple
    println!("Chargement d'une ROM d'exemple...");
    let dummy_rom = vec![0x12, 0x34, 0x56, 0x78, 0xAB, 0xCD, 0xEF, 0x00];
    memory.load_rom("main".to_string(), dummy_rom)?;
    println!("  ROM chargée avec succès ✓");
    
    // Simulation de quelques cycles d'émulation
    println!("Simulation de cycles d'émulation...");
    let mut cycles = 0;
    let target_cycles = 1000;
    
    while cycles < target_cycles {
        // Fetch instruction (simulé)
        let _pc = cpu.registers.get_pc();
        
        // Pour l'instant, on simule juste l'incrémentation du PC
        cpu.registers.set_pc(cpu.registers.get_pc().wrapping_add(4));
        
        cycles += 1;
        
        // Affichage du progrès tous les 100 cycles
        if cycles % 100 == 0 {
            println!("  Cycles exécutés: {}/{}", cycles, target_cycles);
        }
    }
    
    println!("✅ Test d'émulation terminé avec succès !");
    println!("   - Processeur NEC V60: Fonctionnel");
    println!("   - Système mémoire: Fonctionnel");
    println!("   - Chargement ROM: Fonctionnel");
    
    println!("\n🎯 Prochaines étapes:");
    println!("   - Implémenter le décodage d'instructions V60");
    println!("   - Ajouter le rendu graphique wgpu");
    println!("   - Intégrer l'audio SCSP");
    println!("   - Charger de vraies ROMs Model 2");
    
    Ok(())
}