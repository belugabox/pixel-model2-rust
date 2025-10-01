//! Exemple d'utilisation du système ROM SEGA Model 2

use anyhow::Result;
use crate::rom::{Model2RomSystem, Model2MemoryConfig, LoadConfig};
use crate::memory::MemoryInterface;

/// Exemple complet de chargement et mapping ROM
pub fn example_rom_loading() -> Result<()> {
    println!("=== EXEMPLE DE CHARGEMENT ROM SEGA MODEL 2 ===\n");
    
    // Créer le système ROM
    let mut rom_system = Model2RomSystem::new();
    
    // Ajouter des chemins de recherche personnalisés
    rom_system.add_search_path("./roms/model2");
    rom_system.add_search_path("../roms");
    rom_system.add_search_path("D:/ROMS/SEGA");
    
    // Configuration personnalisée du mapping mémoire
    let memory_config = Model2MemoryConfig {
        program_rom_base: 0x00000000,  // ROMs programme 68000
        graphics_rom_base: 0x08000000, // ROMs graphiques
        audio_rom_base: 0x10000000,    // ROMs audio DSP
        data_rom_base: 0x18000000,     // ROMs données
        bank_size: 0x200000,           // 2MB par banque
        bank_mask: 0x1FFFFF,           // Masque pour banking
    };
    rom_system.set_memory_config(memory_config);
    
    // Configuration de chargement personnalisée
    let mut load_config = LoadConfig::default();
    load_config.validate_checksums = true;
    load_config.allow_bad_checksums = false;
    load_config.auto_load_missing = true;
    load_config.max_cache_size = 512 * 1024 * 1024; // 512 MB de cache
    rom_system.rom_manager.set_load_config(load_config);
    
    // Créer un système mémoire fictif pour l'exemple
    // Dans un vrai émulateur, on utiliserait le système mémoire principal
    let mut memory = crate::memory::TestMemory::new(64 * 1024 * 1024); // 64 MB
    
    // Essayer de charger différents jeux
    let games_to_try = ["virtua_fighter_2", "daytona_usa", "virtua_cop"];
    
    for game_name in &games_to_try {
        println!("Tentative de chargement: {}", game_name);
        
        match rom_system.load_and_map_game(game_name, &mut memory) {
            Ok(()) => {
                println!("✅ {} chargé avec succès!", game_name);
                
                // Générer un rapport détaillé
                if let Ok(report) = rom_system.generate_status_report() {
                    println!("\n{}", report);
                }
                
                // Valider le mapping
                if let Ok(validation) = rom_system.memory_mapper.validate_mapping() {
                    if validation.is_valid {
                        println!("✅ Mapping mémoire valide");
                    } else {
                        println!("⚠️ Problèmes de mapping détectés:");
                        for error in &validation.errors {
                            println!("  ❌ {}", error);
                        }
                        for warning in &validation.warnings {
                            println!("  ⚠️ {}", warning);
                        }
                    }
                    
                    // Afficher les statistiques
                    let stats = &validation.statistics;
                    println!("\n📊 Statistiques de mapping:");
                    println!("  Total ROMs: {}", stats.total_roms);
                    println!("  Taille totale: {} octets", stats.total_size);
                    println!("  Programme: {} octets", stats.program_size);
                    println!("  Graphiques: {} octets", stats.graphics_size);
                    println!("  Audio: {} octets", stats.audio_size);
                    println!("  Données: {} octets", stats.data_size);
                }
                
                break; // Premier jeu trouvé et chargé
            },
            Err(e) => {
                println!("❌ Échec du chargement: {}", e);
                println!();
            }
        }
    }
    
    // Tester la lecture rapide depuis le cache ROM
    if let Some(data) = rom_system.memory_mapper.read_rom_data(0x00000000, 1024) {
        println!("✅ Lecture rapide depuis le cache: {} octets", data.len());
        
        // Analyser les premiers octets (vecteurs d'interruption 68000)
        if data.len() >= 8 {
            let stack_pointer = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
            let reset_vector = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
            println!("  Stack Pointer: 0x{:08X}", stack_pointer);
            println!("  Reset Vector: 0x{:08X}", reset_vector);
        }
    }
    
    // Générer un rapport de disponibilité ROM
    if let Ok(availability_report) = rom_system.rom_manager.generate_availability_report() {
        println!("\n{}", availability_report);
    }
    
    Ok(())
}

/// Test unitaire du système ROM
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_rom_system_creation() {
        let rom_system = Model2RomSystem::new();
        assert!(rom_system.rom_manager.rom_cache.is_empty());
    }

    #[test]
    fn test_rom_system_with_test_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut rom_system = Model2RomSystem::new();
        
        // Ajouter le répertoire temporaire comme chemin de recherche
        rom_system.add_search_path(temp_dir.path());
        
        // Créer des fichiers ROM de test
        let program_rom = temp_dir.path().join("mpr-17572.ic1");
        fs::write(&program_rom, create_test_program_rom())?;
        
        let graphics_rom = temp_dir.path().join("mpr-17573.ic2");
        fs::write(&graphics_rom, create_test_graphics_rom())?;
        
        // Générer le rapport de disponibilité
        let report = rom_system.rom_manager.generate_availability_report()?;
        assert!(report.contains("ROMs trouvées"));
        
        Ok(())
    }
    
    /// Crée une ROM programme de test avec vecteurs d'interruption valides
    fn create_test_program_rom() -> Vec<u8> {
        let mut rom = vec![0u8; 1024 * 1024]; // 1 MB
        
        // Vecteurs d'interruption 68000 valides
        rom[0..4].copy_from_slice(&0x00100000u32.to_be_bytes()); // Stack pointer
        rom[4..8].copy_from_slice(&0x00001000u32.to_be_bytes()); // Reset vector
        
        // Quelques instructions NOP pour faire ressembler à du code
        for i in (0x1000..0x2000).step_by(2) {
            rom[i..i+2].copy_from_slice(&0x4E71u16.to_be_bytes()); // NOP
        }
        
        rom
    }
    
    /// Crée une ROM graphique de test avec des données aléatoires
    fn create_test_graphics_rom() -> Vec<u8> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut rom = vec![0u8; 2 * 1024 * 1024]; // 2 MB
        
        // Remplir avec des données pseudo-aléatoires pour simuler des textures
        for (i, byte) in rom.iter_mut().enumerate() {
            let mut hasher = DefaultHasher::new();
            i.hash(&mut hasher);
            *byte = (hasher.finish() & 0xFF) as u8;
        }
        
        rom
    }
}

/// Implémentation de mémoire de test pour les exemples
mod test_memory {
    use super::*;
    use crate::memory::MemoryInterface;

    pub struct TestMemory {
        data: Vec<u8>,
    }

    impl TestMemory {
        pub fn new(size: usize) -> Self {
            Self {
                data: vec![0; size],
            }
        }
    }

    impl MemoryInterface for TestMemory {
        fn read_byte(&self, address: u32) -> Result<u8> {
            let addr = address as usize;
            if addr < self.data.len() {
                Ok(self.data[addr])
            } else {
                Ok(0)
            }
        }

        fn write_byte(&mut self, address: u32, value: u8) -> Result<()> {
            let addr = address as usize;
            if addr < self.data.len() {
                self.data[addr] = value;
            }
            Ok(())
        }

        fn read_word(&self, address: u32) -> Result<u16> {
            let low = self.read_byte(address)?;
            let high = self.read_byte(address + 1)?;
            Ok(((high as u16) << 8) | (low as u16))
        }

        fn write_word(&mut self, address: u32, value: u16) -> Result<()> {
            self.write_byte(address, (value & 0xFF) as u8)?;
            self.write_byte(address + 1, (value >> 8) as u8)?;
            Ok(())
        }

        fn read_dword(&self, address: u32) -> Result<u32> {
            let low = self.read_word(address)?;
            let high = self.read_word(address + 2)?;
            Ok(((high as u32) << 16) | (low as u32))
        }

        fn write_dword(&mut self, address: u32, value: u32) -> Result<()> {
            self.write_word(address, (value & 0xFFFF) as u16)?;
            self.write_word(address + 2, (value >> 16) as u16)?;
            Ok(())
        }
    }
}

// Réexporter pour faciliter l'utilisation
pub use test_memory::TestMemory;