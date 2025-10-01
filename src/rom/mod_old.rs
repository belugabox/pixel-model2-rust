//! Système de gestion des ROMs pour SEGA Model 2
//! 
//! Ce module gère le chargement, la validation, le mapping mémoire et l'organisation des ROMs
//! nécessaires à l'émulation du système SEGA Model 2.
//! 
//! # Architecture
//! 
//! - `database`: Base de données des jeux et métadonnées ROM
//! - `decompression`: Gestion des archives compressées (ZIP, GZIP, 7Z)
//! - `validation`: Validation d'intégrité des ROMs (CRC32, MD5, SHA256)
//! - `loader`: Chargement et gestion des ensembles de ROMs
//! - `mapping`: Mapping mémoire des ROMs vers l'espace d'adressage Model 2

pub mod database;
pub mod decompression;
pub mod validation;
pub mod loader;
pub mod mapping;

// Réexporter les types principaux pour faciliter l'utilisation
pub use database::{GameDatabase, GameInfo, RomInfo, RomType};
pub use decompression::{RomDecompressor, CompressionType};
pub use validation::{RomValidator, ValidationResult};
pub use loader::{RomManager, RomSet, LoadedRom, LoadConfig};
pub use mapping::{RomMemoryMapper, Model2MemoryConfig, MappingInfo};

/// Système de ROM complet pour SEGA Model 2
/// 
/// Combine tous les composants : database, décompression, validation, chargement et mapping mémoire
pub struct Model2RomSystem {
    /// Gestionnaire de ROMs
    pub rom_manager: RomManager,
    
    /// Mapper mémoire
    pub memory_mapper: RomMemoryMapper,
}

impl Model2RomSystem {
    /// Crée un nouveau système ROM complet
    pub fn new() -> Self {
        Self {
            rom_manager: RomManager::new(),
            memory_mapper: RomMemoryMapper::new(),
        }
    }
    
    /// Charge un jeu et l'installe en mémoire
    pub fn load_and_map_game(&mut self, game_name: &str, memory: &mut dyn crate::memory::MemoryInterface) -> anyhow::Result<()> {
        // Charger le jeu
        let rom_set = self.rom_manager.load_game(game_name)?;
        
        // Mapper en mémoire
        self.memory_mapper.load_rom_set(rom_set, memory)?;
        
        Ok(())
    }
    
    /// Ajoute un chemin de recherche pour les ROMs
    pub fn add_search_path<P: AsRef<std::path::Path>>(&mut self, path: P) {
        self.rom_manager.add_search_path(path);
    }
    
    /// Configure le mapping mémoire
    pub fn set_memory_config(&mut self, config: Model2MemoryConfig) {
        self.memory_mapper.set_config(config);
    }
    
    /// Génère un rapport d'état complet
    pub fn generate_status_report(&self) -> anyhow::Result<String> {
        let mut report = String::new();
        
        // Rapport de disponibilité ROM
        report.push_str(&self.rom_manager.generate_availability_report()?);
        report.push_str("\n\n");
        
        // Rapport de mapping mémoire
        if let Some(mapping_info) = self.memory_mapper.get_mapping_info() {
            report.push_str("=== MAPPING MÉMOIRE ===\n\n");
            report.push_str(&format!("Jeu: {}\n", mapping_info.game_name));
            report.push_str(&format!("ROMs mappées: {}\n", mapping_info.total_roms));
            report.push_str(&format!("Taille totale: {} octets\n\n", mapping_info.total_size));
            
            for (rom_name, address, size, rom_type) in &mapping_info.regions {
                report.push_str(&format!("  {} ({:?}): 0x{:08X} - 0x{:08X} ({} octets)\n", 
                               rom_name, rom_type, address, address + *size as u32, size));
            }
        } else {
            report.push_str("=== MAPPING MÉMOIRE ===\n\nAucun jeu mappé\n");
        }
        
        Ok(report)
    }
        for search_path in &self.search_paths {
            let full_path = Path::new(search_path).join(rom_name);
            if full_path.exists() {
                return Ok(Some(full_path.to_string_lossy().to_string()));
            }
        }
        Ok(None)
    }
}

impl Default for RomLoader {
    fn default() -> Self {
        Self::new()
    }
}