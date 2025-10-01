//! Chargement et gestion des ROMs Model 2

use anyhow::{Result, anyhow};
use std::path::Path;
use crate::memory::rom::*;

/// Chargeur de ROMs Model 2
pub struct RomLoader {
    search_paths: Vec<String>,
}

impl RomLoader {
    pub fn new() -> Self {
        Self {
            search_paths: vec![
                "./roms".to_string(),
                "./".to_string(),
            ],
        }
    }
    
    pub fn add_search_path(&mut self, path: String) {
        self.search_paths.push(path);
    }
    
    pub fn load_game(&self, game_name: &str) -> Result<RomSet> {
        let game_info = match game_name {
            "vf2" | "virtua_fighter_2" => GameInfo::virtua_fighter_2(),
            "daytona" | "daytona_usa" => GameInfo::daytona_usa(),
            _ => return Err(anyhow!("Jeu non reconnu: {}", game_name)),
        };
        
        let mut rom_set = RomSet::new(game_info.clone());
        
        // Chercher et charger les ROMs requises
        for rom_name in &game_info.required_roms {
            if let Some(rom_path) = self.find_rom_file(rom_name)? {
                let rom = Rom::from_file(&rom_path)?;
                rom_set.add_rom(rom_name.clone(), rom);
            } else {
                return Err(anyhow!("ROM manquante: {}", rom_name));
            }
        }
        
        // Charger les ROMs optionnelles si disponibles
        for rom_name in &game_info.optional_roms {
            if let Some(rom_path) = self.find_rom_file(rom_name).unwrap_or(None) {
                let rom = Rom::from_file(&rom_path)?;
                rom_set.add_rom(rom_name.clone(), rom);
            }
        }
        
        rom_set.verify_completeness()?;
        rom_set.verify_integrity()?;
        
        Ok(rom_set)
    }
    
    fn find_rom_file(&self, rom_name: &str) -> Result<Option<String>> {
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