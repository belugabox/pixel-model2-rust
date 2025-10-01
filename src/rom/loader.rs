//! Système de chargement et mapping mémoire des ROMs

use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use walkdir::WalkDir;

use super::database::{GameDatabase, GameInfo, RomInfo, RomType};
use super::decompression::{RomDecompressor, DecompressionResult};
use super::validation::{RomValidator, ValidationResult};
use crate::memory::MemoryInterface;

/// Gestionnaire principal de ROMs
pub struct RomManager {
    /// Base de données des jeux
    database: GameDatabase,
    
    /// Chemins de recherche pour les ROMs
    search_paths: Vec<PathBuf>,
    
    /// Cache des ROMs chargées
    rom_cache: HashMap<String, LoadedRom>,
    
    /// Configuration de chargement
    load_config: LoadConfig,
}

/// ROM chargée en mémoire
#[derive(Debug, Clone)]
pub struct LoadedRom {
    /// Données de la ROM
    pub data: Vec<u8>,
    
    /// Informations sur la ROM
    pub info: RomInfo,
    
    /// Résultat de validation
    pub validation: ValidationResult,
    
    /// Chemin du fichier source
    pub source_path: PathBuf,
    
    /// Type de compression utilisé
    pub compression_type: super::decompression::CompressionType,
}

/// Configuration de chargement
#[derive(Debug, Clone)]
pub struct LoadConfig {
    /// Valider les checksums
    pub validate_checksums: bool,
    
    /// Permettre les ROMs avec checksums incorrects
    pub allow_bad_checksums: bool,
    
    /// Charger automatiquement les ROMs manquantes
    pub auto_load_missing: bool,
    
    /// Taille maximale de cache en octets
    pub max_cache_size: usize,
    
    /// Extensions de fichiers à rechercher
    pub file_extensions: Vec<String>,
}

/// Ensemble de ROMs pour un jeu
#[derive(Debug)]
pub struct RomSet {
    /// Informations sur le jeu
    pub game_info: GameInfo,
    
    /// ROMs chargées
    pub roms: HashMap<String, LoadedRom>,
    
    /// Statut de validation global
    pub is_valid: bool,
    
    /// Mapping mémoire des ROMs
    pub memory_map: MemoryMap,
}

/// Plan de mapping mémoire
#[derive(Debug, Clone)]
pub struct MemoryMap {
    /// Régions de mémoire mappées
    pub regions: Vec<MemoryRegion>,
    
    /// Taille totale de l'espace mémoire
    pub total_size: usize,
}

/// Région de mémoire mappée
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    /// Nom de la ROM source
    pub rom_name: String,
    
    /// Adresse de départ
    pub start_address: u32,
    
    /// Adresse de fin (exclusive)
    pub end_address: u32,
    
    /// Offset dans la ROM source
    pub rom_offset: usize,
    
    /// Taille de la région
    pub size: usize,
    
    /// Type de ROM
    pub rom_type: RomType,
    
    /// Banque mémoire
    pub bank: u8,
    
    /// Lecture seule
    pub read_only: bool,
}

impl Default for LoadConfig {
    fn default() -> Self {
        Self {
            validate_checksums: true,
            allow_bad_checksums: false,
            auto_load_missing: true,
            max_cache_size: 256 * 1024 * 1024, // 256 MB
            file_extensions: vec![
                "bin".to_string(), "rom".to_string(), "zip".to_string(),
                "gz".to_string(), "7z".to_string(),
                "ic1".to_string(), "ic2".to_string(), "ic3".to_string(),
                "ic4".to_string(), "ic5".to_string(), "ic6".to_string(),
                "ic7".to_string(), "ic8".to_string(), "ic9".to_string(),
                "ic10".to_string(), "ic11".to_string(), "ic12".to_string(),
            ],
        }
    }
}

impl RomManager {
    /// Crée un nouveau gestionnaire de ROMs
    pub fn new() -> Self {
        Self {
            database: GameDatabase::new(),
            search_paths: vec![
                PathBuf::from("./roms"),
                PathBuf::from("./"),
                PathBuf::from("../roms"),
            ],
            rom_cache: HashMap::new(),
            load_config: LoadConfig::default(),
        }
    }
    
    /// Ajoute un chemin de recherche
    pub fn add_search_path<P: AsRef<Path>>(&mut self, path: P) {
        self.search_paths.push(path.as_ref().to_path_buf());
    }
    
    /// Configure les paramètres de chargement
    pub fn set_load_config(&mut self, config: LoadConfig) {
        self.load_config = config;
    }
    
    /// Charge un jeu complet avec toutes ses ROMs
    pub fn load_game(&mut self, game_name: &str) -> Result<RomSet> {
        let game_info = self.database.find_game(game_name)
            .ok_or_else(|| anyhow!("Jeu non trouvé: {}", game_name))?
            .clone();
        
        println!("Chargement du jeu: {}", game_info.name);
        
        let mut rom_set = RomSet {
            game_info: game_info.clone(),
            roms: HashMap::new(),
            is_valid: true,
            memory_map: MemoryMap {
                regions: Vec::new(),
                total_size: 0,
            },
        };
        
        // Charger les ROMs requises
        for rom_info in &game_info.required_roms {
            match self.load_rom(&rom_info.filename, Some(rom_info)) {
                Ok(loaded_rom) => {
                    if !loaded_rom.validation.is_valid && !self.load_config.allow_bad_checksums {
                        rom_set.is_valid = false;
                        eprintln!("ROM invalide: {} ({})", rom_info.filename, 
                                loaded_rom.validation.errors.iter()
                                    .map(|e| e.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", "));
                    }
                    rom_set.roms.insert(rom_info.filename.clone(), loaded_rom);
                },
                Err(e) => {
                    rom_set.is_valid = false;
                    eprintln!("Impossible de charger la ROM {}: {}", rom_info.filename, e);
                    if !self.load_config.auto_load_missing {
                        return Err(e);
                    }
                }
            }
        }
        
        // Charger les ROMs optionnelles
        for rom_info in &game_info.optional_roms {
            if let Ok(loaded_rom) = self.load_rom(&rom_info.filename, Some(rom_info)) {
                rom_set.roms.insert(rom_info.filename.clone(), loaded_rom);
            }
        }
        
        // Créer le mapping mémoire
        rom_set.memory_map = self.create_memory_map(&rom_set)?;
        
        println!("Jeu chargé: {} ROMs, {} octets au total", 
                 rom_set.roms.len(), rom_set.memory_map.total_size);
        
        Ok(rom_set)
    }
    
    /// Charge une ROM individuelle
    pub fn load_rom(&mut self, filename: &str, expected_info: Option<&RomInfo>) -> Result<LoadedRom> {
        // Vérifier le cache
        if let Some(cached_rom) = self.rom_cache.get(filename) {
            return Ok(cached_rom.clone());
        }
        
        // Chercher le fichier
        let file_path = self.find_rom_file(filename)?;
        
        // Décompresser si nécessaire
        let decompression_result = RomDecompressor::decompress_file(&file_path)?;
        
        // Trouver la ROM dans les fichiers décompressés
        let (rom_filename, rom_data) = self.find_rom_in_files(filename, decompression_result.files)?;
        
        // Créer les informations de ROM si non fournies
        let rom_info = if let Some(info) = expected_info {
            info.clone()
        } else {
            RomInfo {
                filename: rom_filename.clone(),
                rom_type: RomValidator::detect_rom_type(&rom_data, &rom_filename),
                size: rom_data.len(),
                crc32: RomValidator::calculate_crc32(&rom_data),
                md5: RomValidator::calculate_md5(&rom_data),
                load_address: 0,
                bank: 0,
                required: true,
            }
        };
        
        // Valider la ROM
        let validation = if self.load_config.validate_checksums {
            RomValidator::validate_rom(&rom_data, &rom_info)
        } else {
            ValidationResult {
                is_valid: true,
                calculated_crc32: RomValidator::calculate_crc32(&rom_data),
                calculated_md5: RomValidator::calculate_md5(&rom_data),
                calculated_sha256: RomValidator::calculate_sha256(&rom_data),
                file_size: rom_data.len(),
                errors: Vec::new(),
                warnings: Vec::new(),
            }
        };
        
        let loaded_rom = LoadedRom {
            data: rom_data,
            info: rom_info,
            validation,
            source_path: file_path,
            compression_type: decompression_result.compression_type,
        };
        
        // Ajouter au cache
        self.rom_cache.insert(filename.to_string(), loaded_rom.clone());
        self.cleanup_cache()?;
        
        Ok(loaded_rom)
    }
    
    /// Recherche un fichier ROM dans les chemins configurés
    fn find_rom_file(&self, filename: &str) -> Result<PathBuf> {
        for search_path in &self.search_paths {
            if !search_path.exists() {
                continue;
            }
            
            // Recherche directe
            let direct_path = search_path.join(filename);
            if direct_path.exists() {
                return Ok(direct_path);
            }
            
            // Recherche récursive avec extensions
            for entry in WalkDir::new(search_path).max_depth(3) {
                let entry = entry.map_err(|e| anyhow!("Erreur de lecture: {}", e))?;
                let path = entry.path();
                
                if path.is_file() {
                    // Vérifier le nom exact
                    if path.file_name() == Some(filename.as_ref()) {
                        return Ok(path.to_path_buf());
                    }
                    
                    // Vérifier avec extensions
                    if let Some(stem) = path.file_stem() {
                        if stem.to_str() == Some(filename) {
                            if let Some(ext) = path.extension() {
                                if self.load_config.file_extensions.contains(&ext.to_string_lossy().to_lowercase()) {
                                    return Ok(path.to_path_buf());
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Err(anyhow!("ROM non trouvée: {}", filename))
    }
    
    /// Trouve une ROM spécifique dans une liste de fichiers décompressés
    fn find_rom_in_files(&self, target_filename: &str, files: Vec<(String, Vec<u8>)>) -> Result<(String, Vec<u8>)> {
        // Recherche exacte
        for (filename, data) in &files {
            if filename == target_filename {
                return Ok((filename.clone(), data.clone()));
            }
        }
        
        // Recherche partielle (sans extension)
        let target_stem = Path::new(target_filename).file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(target_filename);
        
        for (filename, data) in &files {
            let file_stem = Path::new(filename).file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(filename);
            
            if file_stem == target_stem {
                return Ok((filename.clone(), data.clone()));
            }
        }
        
        // Si un seul fichier, l'utiliser
        if files.len() == 1 {
            return Ok(files.into_iter().next().unwrap());
        }
        
        Err(anyhow!("ROM {} non trouvée dans l'archive", target_filename))
    }
    
    /// Crée le mapping mémoire pour un ensemble de ROMs
    fn create_memory_map(&self, rom_set: &RomSet) -> Result<MemoryMap> {
        let mut regions = Vec::new();
        let mut total_size = 0;
        
        // Créer les régions selon les informations des ROMs
        for (filename, loaded_rom) in &rom_set.roms {
            let region = MemoryRegion {
                rom_name: filename.clone(),
                start_address: loaded_rom.info.load_address,
                end_address: loaded_rom.info.load_address + loaded_rom.data.len() as u32,
                rom_offset: 0,
                size: loaded_rom.data.len(),
                rom_type: loaded_rom.info.rom_type.clone(),
                bank: loaded_rom.info.bank,
                read_only: true,
            };
            
            total_size += loaded_rom.data.len();
            regions.push(region);
        }
        
        // Trier les régions par adresse
        regions.sort_by_key(|r| r.start_address);
        
        // Vérifier les chevauchements
        for i in 1..regions.len() {
            if regions[i-1].end_address > regions[i].start_address {
                eprintln!("Avertissement: Chevauchement mémoire détecté entre {} et {}", 
                         regions[i-1].rom_name, regions[i].rom_name);
            }
        }
        
        Ok(MemoryMap {
            regions,
            total_size,
        })
    }
    
    /// Nettoie le cache selon la taille maximale configurée
    fn cleanup_cache(&mut self) -> Result<()> {
        let current_size: usize = self.rom_cache.values()
            .map(|rom| rom.data.len())
            .sum();
        
        if current_size > self.load_config.max_cache_size {
            // Simple LRU: supprimer les entrées les plus anciennes
            // Pour une implémentation complète, on utiliserait un LRU cache
            self.rom_cache.clear();
        }
        
        Ok(())
    }
    
    /// Liste les ROMs disponibles dans les chemins de recherche
    pub fn scan_available_roms(&self) -> Result<Vec<PathBuf>> {
        let mut roms = Vec::new();
        
        for search_path in &self.search_paths {
            if !search_path.exists() {
                continue;
            }
            
            for entry in WalkDir::new(search_path).max_depth(3) {
                let entry = entry.map_err(|e| anyhow!("Erreur de lecture: {}", e))?;
                let path = entry.path();
                
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if self.load_config.file_extensions.contains(&ext.to_string_lossy().to_lowercase()) {
                            roms.push(path.to_path_buf());
                        }
                    }
                }
            }
        }
        
        Ok(roms)
    }
    
    /// Génère un rapport sur les ROMs disponibles
    pub fn generate_availability_report(&self) -> Result<String> {
        let mut report = String::new();
        report.push_str("=== RAPPORT DE DISPONIBILITÉ ROM ===\n\n");
        
        let available_roms = self.scan_available_roms()?;
        report.push_str(&format!("ROMs trouvées: {}\n\n", available_roms.len()));
        
        for path in &available_roms {
            report.push_str(&format!("  {}\n", path.display()));
        }
        
        report.push_str("\n=== JEUX SUPPORTÉS ===\n\n");
        
        for game in self.database.list_games() {
            report.push_str(&format!("{} ({})\n", game.name, game.short_name));
            
            let mut available_count = 0;
            for rom_info in &game.required_roms {
                if available_roms.iter().any(|p| p.file_name().map(|n| n.to_string_lossy()).as_deref() == Some(&rom_info.filename)) {
                    available_count += 1;
                }
            }
            
            report.push_str(&format!("  ROMs disponibles: {}/{}\n", available_count, game.required_roms.len()));
            
            if available_count == game.required_roms.len() {
                report.push_str("  ✅ Prêt à jouer\n");
            } else {
                report.push_str("  ❌ ROMs manquantes\n");
            }
            
            report.push('\n');
        }
        
        Ok(report)
    }
}

impl Default for RomManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_rom_manager_creation() {
        let manager = RomManager::new();
        assert!(!manager.search_paths.is_empty());
        assert!(manager.rom_cache.is_empty());
    }

    #[test]
    fn test_add_search_path() {
        let mut manager = RomManager::new();
        let initial_count = manager.search_paths.len();
        
        manager.add_search_path("/test/path");
        assert_eq!(manager.search_paths.len(), initial_count + 1);
    }

    #[test]
    fn test_memory_region() {
        let region = MemoryRegion {
            rom_name: "test.bin".to_string(),
            start_address: 0x1000,
            end_address: 0x2000,
            rom_offset: 0,
            size: 0x1000,
            rom_type: RomType::Program,
            bank: 0,
            read_only: true,
        };
        
        assert_eq!(region.size, 0x1000);
        assert_eq!(region.end_address - region.start_address, region.size as u32);
    }

    #[test]
    fn test_scan_available_roms() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut manager = RomManager::new();
        manager.search_paths.clear();
        manager.add_search_path(temp_dir.path());
        
        // Créer un fichier ROM test
        let rom_path = temp_dir.path().join("test.bin");
        fs::write(&rom_path, b"test rom data")?;
        
        let available = manager.scan_available_roms()?;
        assert_eq!(available.len(), 1);
        assert_eq!(available[0], rom_path);
        
        Ok(())
    }
}