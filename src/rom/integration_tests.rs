//! Tests d'intégration du système ROM SEGA Model 2

use anyhow::Result;
use tempfile::TempDir;
use std::fs;

use crate::rom::{Model2RomSystem, Model2MemoryConfig};
use crate::memory::MemoryInterface;

/// Implémentation de mémoire de test
struct TestMemory {
    data: Vec<u8>,
}

impl TestMemory {
    fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
        }
    }
}

impl MemoryInterface for TestMemory {
    fn read_u8(&self, address: u32) -> Result<u8> {
        let addr = address as usize;
        if addr < self.data.len() {
            Ok(self.data[addr])
        } else {
            Ok(0)
        }
    }

    fn write_u8(&mut self, address: u32, value: u8) -> Result<()> {
        let addr = address as usize;
        if addr < self.data.len() {
            self.data[addr] = value;
        }
        Ok(())
    }

    fn read_u16(&self, address: u32) -> Result<u16> {
        let low = self.read_u8(address)?;
        let high = self.read_u8(address + 1)?;
        Ok(((high as u16) << 8) | (low as u16))
    }

    fn write_u16(&mut self, address: u32, value: u16) -> Result<()> {
        self.write_u8(address, (value & 0xFF) as u8)?;
        self.write_u8(address + 1, (value >> 8) as u8)?;
        Ok(())
    }

    fn read_u32(&self, address: u32) -> Result<u32> {
        let low = self.read_u16(address)?;
        let high = self.read_u16(address + 2)?;
        Ok(((high as u32) << 16) | (low as u32))
    }

    fn write_u32(&mut self, address: u32, value: u32) -> Result<()> {
        self.write_u16(address, (value & 0xFFFF) as u16)?;
        self.write_u16(address + 2, (value >> 16) as u16)?;
        Ok(())
    }
}

/// Test d'intégration complète du système ROM
#[test]
fn test_complete_rom_system() -> Result<()> {
    // Créer un répertoire temporaire pour les ROMs de test
    let temp_dir = TempDir::new()?;
    
    // Créer des ROMs de test
    create_test_roms(&temp_dir)?;
    
    // Créer le système ROM
    let mut rom_system = Model2RomSystem::new();
    rom_system.add_search_path(temp_dir.path());
    
    // Configuration mémoire personnalisée
    let memory_config = Model2MemoryConfig {
        program_rom_base: 0x00000000,
        graphics_rom_base: 0x01000000,
        audio_rom_base: 0x02000000,
        data_rom_base: 0x03000000,
        bank_size: 0x100000, // 1MB
        bank_mask: 0x0FFFFF,
    };
    rom_system.set_memory_config(memory_config);
    
    // Créer un système mémoire de test
    let mut memory = TestMemory::new(64 * 1024 * 1024); // 64 MB
    
    // Essayer de charger un jeu (ne fonctionnera pas sans vraies ROMs mais testera le système)
    match rom_system.load_and_map_game("virtua_fighter_2", &mut memory) {
        Ok(()) => {
            // Si ça marche, vérifier le mapping
            let report = rom_system.generate_status_report()?;
            assert!(report.contains("MAPPING MÉMOIRE"));
            
            // Vérifier la validation du mapping
            let validation = rom_system.memory_mapper.validate_mapping()?;
            println!("Validation: {:?}", validation);
        },
        Err(e) => {
            // C'est normal, on n'a pas les vraies ROMs
            println!("Chargement échoué comme attendu: {}", e);
            assert!(e.to_string().contains("ROM non trouvée") || e.to_string().contains("Jeu non trouvé"));
        }
    }
    
    // Tester le rapport de disponibilité
    let availability_report = rom_system.rom_manager.generate_availability_report()?;
    assert!(availability_report.contains("RAPPORT DE DISPONIBILITÉ ROM"));
    assert!(availability_report.contains("ROMs trouvées"));
    
    println!("Test d'intégration ROM réussi !");
    Ok(())
}

/// Test de scan de ROMs disponibles
#[test]
fn test_rom_scanning() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Créer quelques fichiers ROM de test
    fs::write(temp_dir.path().join("test1.bin"), b"test rom 1")?;
    fs::write(temp_dir.path().join("test2.rom"), b"test rom 2")?;
    fs::write(temp_dir.path().join("archive.zip"), b"fake zip")?;
    
    let mut rom_system = Model2RomSystem::new();
    rom_system.add_search_path(temp_dir.path());
    
    let available = rom_system.rom_manager.scan_available_roms()?;
    assert_eq!(available.len(), 3);
    
    let report = rom_system.rom_manager.generate_availability_report()?;
    assert!(report.contains("test1.bin"));
    assert!(report.contains("test2.rom"));
    assert!(report.contains("archive.zip"));
    
    Ok(())
}

/// Test de configuration mémoire
#[test]
fn test_memory_configuration() {
    let mut rom_system = Model2RomSystem::new();
    
    // Configuration par défaut
    let default_info = rom_system.memory_mapper.get_mapping_info();
    assert!(default_info.is_none());
    
    // Configuration personnalisée
    let custom_config = Model2MemoryConfig {
        program_rom_base: 0x10000000,
        graphics_rom_base: 0x20000000,
        audio_rom_base: 0x30000000,
        data_rom_base: 0x40000000,
        bank_size: 0x200000, // 2MB
        bank_mask: 0x1FFFFF,
    };
    
    rom_system.set_memory_config(custom_config.clone());
    
    // Vérifier que la configuration a été appliquée
    // (on ne peut pas tester directement sans charger de ROMs)
    println!("Configuration mémoire appliquée");
}

/// Test de validation de mapping
#[test]
fn test_mapping_validation() -> Result<()> {
    let mut rom_system = Model2RomSystem::new();
    
    // Validation sans ROMs chargées
    let validation = rom_system.memory_mapper.validate_mapping()?;
    assert!(validation.is_valid);
    assert!(validation.warnings.contains(&"Aucune ROM mappée".to_string()));
    assert_eq!(validation.statistics.total_roms, 0);
    
    Ok(())
}

/// Crée des ROMs de test dans le répertoire temporaire
fn create_test_roms(temp_dir: &TempDir) -> Result<()> {
    // Créer une ROM programme avec vecteurs d'interruption valides
    let mut program_rom = vec![0u8; 1024 * 1024]; // 1 MB
    program_rom[0..4].copy_from_slice(&0x00100000u32.to_be_bytes()); // Stack pointer
    program_rom[4..8].copy_from_slice(&0x00001000u32.to_be_bytes()); // Reset vector
    
    // Remplir avec quelques instructions NOP
    for i in (0x1000..0x2000).step_by(2) {
        program_rom[i..i+2].copy_from_slice(&0x4E71u16.to_be_bytes()); // NOP
    }
    
    fs::write(temp_dir.path().join("program_test.bin"), program_rom)?;
    
    // Créer une ROM graphique avec données pseudo-aléatoires
    let graphics_rom = create_pseudo_random_data(2 * 1024 * 1024, 0x12345678);
    fs::write(temp_dir.path().join("graphics_test.bin"), graphics_rom)?;
    
    // Créer une ROM audio
    let audio_rom = create_pseudo_random_data(512 * 1024, 0x87654321);
    fs::write(temp_dir.path().join("audio_test.bin"), audio_rom)?;
    
    // Créer une ROM de données
    let data_rom = vec![0x00, 0x01, 0x02, 0x03].repeat(256 * 1024);
    fs::write(temp_dir.path().join("data_test.bin"), data_rom)?;
    
    Ok(())
}

/// Génère des données pseudo-aléatoires pour les tests
fn create_pseudo_random_data(size: usize, seed: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity(size);
    let mut rng = seed;
    
    for _ in 0..size {
        // LCG simple pour générer des nombres pseudo-aléatoires
        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
        data.push((rng >> 16) as u8);
    }
    
    data
}

/// Test de performance basique
#[test]
fn test_rom_system_performance() -> Result<()> {
    let temp_dir = TempDir::new()?;
    create_test_roms(&temp_dir)?;
    
    let start = std::time::Instant::now();
    
    let mut rom_system = Model2RomSystem::new();
    rom_system.add_search_path(temp_dir.path());
    
    let _available = rom_system.rom_manager.scan_available_roms()?;
    let _report = rom_system.rom_manager.generate_availability_report()?;
    
    let elapsed = start.elapsed();
    
    // Le scan et rapport devraient être rapides même avec plusieurs ROMs
    assert!(elapsed.as_millis() < 1000, "Performance trop lente: {:?}", elapsed);
    
    println!("Performance ROM system: {:?}", elapsed);
    Ok(())
}