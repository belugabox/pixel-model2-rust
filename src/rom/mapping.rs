//! Mapping ROM vers système mémoire SEGA Model 2

use anyhow::{Result, anyhow};
use std::collections::HashMap;

use super::loader::{RomSet, LoadedRom, MemoryRegion};
use super::database::RomType;
use crate::memory::MemoryInterface;

/// Gestionnaire de mapping ROM vers mémoire système
pub struct RomMemoryMapper {
    /// Ensemble de ROMs actuel
    current_rom_set: Option<RomSet>,
    
    /// Configuration du mapping SEGA Model 2
    mapping_config: Model2MemoryConfig,
    
    /// Cache des données mappées
    mapped_data: HashMap<u32, Vec<u8>>,
}

/// Configuration mémoire SEGA Model 2
#[derive(Debug, Clone)]
pub struct Model2MemoryConfig {
    /// Base des ROMs programme (68000)
    pub program_rom_base: u32,
    
    /// Base des ROMs graphiques
    pub graphics_rom_base: u32,
    
    /// Base des ROMs audio
    pub audio_rom_base: u32,
    
    /// Base des ROMs données
    pub data_rom_base: u32,
    
    /// Taille des banques mémoire
    pub bank_size: u32,
    
    /// Masque d'adresse pour banking
    pub bank_mask: u32,
}

impl Default for Model2MemoryConfig {
    fn default() -> Self {
        Self {
            // Configuration typique SEGA Model 2
            program_rom_base: 0x00000000,  // ROMs programme 68000
            graphics_rom_base: 0x08000000, // ROMs graphiques
            audio_rom_base: 0x10000000,    // ROMs audio (DSP)
            data_rom_base: 0x18000000,     // ROMs données diverses
            bank_size: 0x100000,           // 1MB par banque
            bank_mask: 0x0FFFFF,           // Masque pour banking
        }
    }
}

impl RomMemoryMapper {
    /// Crée un nouveau mapper mémoire
    pub fn new() -> Self {
        Self {
            current_rom_set: None,
            mapping_config: Model2MemoryConfig::default(),
            mapped_data: HashMap::new(),
        }
    }
    
    /// Configure le mapping mémoire
    pub fn set_config(&mut self, config: Model2MemoryConfig) {
        self.mapping_config = config;
        self.remap_current_roms().ok(); // Remapper si des ROMs sont chargées
    }
    
    /// Charge un ensemble de ROMs et les mappe en mémoire
    pub fn load_rom_set(&mut self, rom_set: RomSet, memory: &mut dyn MemoryInterface) -> Result<()> {
        println!("Mapping de {} ROMs en mémoire système", rom_set.roms.len());
        
        // Vider le cache précédent
        self.mapped_data.clear();
        
        // Mapper chaque ROM selon son type
        for (rom_name, loaded_rom) in &rom_set.roms {
            self.map_rom_to_memory(rom_name, loaded_rom, memory)?;
        }
        
        // Stocker l'ensemble de ROMs
        self.current_rom_set = Some(rom_set);
        
        println!("Mapping ROM terminé avec succès");
        Ok(())
    }
    
    /// Mappe une ROM individuelle en mémoire
    fn map_rom_to_memory(&mut self, rom_name: &str, loaded_rom: &LoadedRom, memory: &mut dyn MemoryInterface) -> Result<()> {
        let base_address = self.calculate_base_address(&loaded_rom.info.rom_type);
        let final_address = base_address + (loaded_rom.info.bank as u32 * self.mapping_config.bank_size);
        
        println!("Mapping ROM {} ({}) vers 0x{:08X} ({} octets)", 
                 rom_name, 
                 format!("{:?}", loaded_rom.info.rom_type),
                 final_address,
                 loaded_rom.data.len());
        
        // Vérifier la taille
        if loaded_rom.data.len() > self.mapping_config.bank_size as usize {
            return Err(anyhow!("ROM {} trop grande pour une banque ({} > {})", 
                              rom_name, loaded_rom.data.len(), self.mapping_config.bank_size));
        }
        
        // Écrire les données en mémoire
        for (offset, &byte) in loaded_rom.data.iter().enumerate() {
            let address = final_address + offset as u32;
            memory.write_u8(address, byte)?;
        }
        
        // Stocker dans le cache pour lecture rapide
        self.mapped_data.insert(final_address, loaded_rom.data.clone());
        
        // Configuration spéciale selon le type de ROM
        match loaded_rom.info.rom_type {
            RomType::Program => {
                self.setup_program_rom_mapping(final_address, &loaded_rom.data, memory)?;
            },
            RomType::Graphics => {
                self.setup_graphics_rom_mapping(final_address, &loaded_rom.data, memory)?;
            },
            RomType::Sound => {
                self.setup_audio_rom_mapping(final_address, &loaded_rom.data, memory)?;
            },
            RomType::Data => {
                self.setup_data_rom_mapping(final_address, &loaded_rom.data, memory)?;
            },
            RomType::Geometry => {
                self.setup_graphics_rom_mapping(final_address, &loaded_rom.data, memory)?;
            },
            RomType::Texture => {
                self.setup_graphics_rom_mapping(final_address, &loaded_rom.data, memory)?;
            },
            RomType::Samples => {
                self.setup_audio_rom_mapping(final_address, &loaded_rom.data, memory)?;
            },
            RomType::Config => {
                self.setup_data_rom_mapping(final_address, &loaded_rom.data, memory)?;
            },
            RomType::Microcode => {
                self.setup_data_rom_mapping(final_address, &loaded_rom.data, memory)?;
            },
        }
        
        Ok(())
    }
    
    /// Calcule l'adresse de base selon le type de ROM
    fn calculate_base_address(&self, rom_type: &RomType) -> u32 {
        match rom_type {
            RomType::Program => self.mapping_config.program_rom_base,
            RomType::Graphics | RomType::Geometry | RomType::Texture => self.mapping_config.graphics_rom_base,
            RomType::Sound | RomType::Samples => self.mapping_config.audio_rom_base,
            RomType::Data | RomType::Config | RomType::Microcode => self.mapping_config.data_rom_base,
        }
    }
    
    /// Configure le mapping spécifique aux ROMs programme
    fn setup_program_rom_mapping(&self, base_address: u32, data: &[u8], memory: &mut dyn MemoryInterface) -> Result<()> {
        // Configuration pour CPU 68000
        println!("Configuration ROM programme à 0x{:08X}", base_address);
        
        // Vérifier les vecteurs d'interruption (premiers 1024 octets)
        if data.len() >= 1024 {
            let stack_pointer = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
            let reset_vector = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
            
            println!("  Stack Pointer initial: 0x{:08X}", stack_pointer);
            println!("  Reset Vector: 0x{:08X}", reset_vector);
            
            // Valider les vecteurs
            if stack_pointer < 0x10000000 && reset_vector >= base_address && reset_vector < base_address + data.len() as u32 {
                println!("  ✅ Vecteurs d'interruption valides");
            } else {
                println!("  ⚠️ Vecteurs d'interruption suspects");
            }
        }
        
        Ok(())
    }
    
    /// Configure le mapping spécifique aux ROMs graphiques
    fn setup_graphics_rom_mapping(&self, base_address: u32, data: &[u8], memory: &mut dyn MemoryInterface) -> Result<()> {
        println!("Configuration ROM graphiques à 0x{:08X}", base_address);
        
        // Analyser les données graphiques
        let mut texture_count = 0;
        let mut sprite_count = 0;
        
        // Heuristiques simples pour détecter le contenu graphique
        for chunk in data.chunks(1024) {
            let entropy = calculate_entropy(chunk);
            
            if entropy > 0.7 {
                // Données avec beaucoup d'entropie = probablement des textures
                texture_count += 1;
            } else if entropy > 0.3 {
                // Données avec entropie moyenne = probablement des sprites
                sprite_count += 1;
            }
        }
        
        println!("  Estimation: {} chunks de textures, {} chunks de sprites", texture_count, sprite_count);
        
        Ok(())
    }
    
    /// Configure le mapping spécifique aux ROMs audio
    fn setup_audio_rom_mapping(&self, base_address: u32, data: &[u8], memory: &mut dyn MemoryInterface) -> Result<()> {
        println!("Configuration ROM audio à 0x{:08X}", base_address);
        
        // Détecter le format audio (PCM, ADPCM, etc.)
        let sample_rate = self.detect_audio_format(data);
        
        if let Some(rate) = sample_rate {
            println!("  Format audio détecté: {} Hz", rate);
        } else {
            println!("  Format audio non reconnu");
        }
        
        Ok(())
    }
    
    /// Configure le mapping spécifique aux ROMs données
    fn setup_data_rom_mapping(&self, base_address: u32, data: &[u8], memory: &mut dyn MemoryInterface) -> Result<()> {
        println!("Configuration ROM données à 0x{:08X}", base_address);
        
        // Analyser le type de données
        if data.len() >= 16 {
            // Chercher des patterns communs
            if data[0..4] == [0x00, 0x00, 0x00, 0x00] {
                println!("  Possibles données de configuration");
            } else if data.iter().all(|&b| b.is_ascii()) {
                println!("  Possibles données texte/ASCII");
            } else {
                println!("  Données binaires génériques");
            }
        }
        
        Ok(())
    }
    
    /// Détecte le format audio dans les données ROM
    fn detect_audio_format(&self, data: &[u8]) -> Option<u32> {
        // Heuristiques simples pour détecter le sample rate
        if data.len() < 1024 {
            return None;
        }
        
        // Chercher des patterns périodiques
        let mut best_period = None;
        let mut best_score = 0.0;
        
        for period in [44100, 22050, 11025, 8000].iter() {
            let score = self.calculate_periodicity_score(data, *period as usize);
            if score > best_score {
                best_score = score;
                best_period = Some(*period);
            }
        }
        
        if best_score > 0.5 {
            best_period
        } else {
            None
        }
    }
    
    /// Calcule un score de périodicité pour détecter le sample rate
    fn calculate_periodicity_score(&self, data: &[u8], period: usize) -> f32 {
        if data.len() < period * 2 {
            return 0.0;
        }
        
        let mut correlation = 0.0;
        let samples = (data.len() / period).min(100); // Limiter le calcul
        
        for i in 0..samples {
            let offset1 = i * period;
            let offset2 = offset1 + period;
            
            if offset2 < data.len() {
                let diff = (data[offset1] as i16 - data[offset2] as i16).abs();
                correlation += 1.0 / (1.0 + diff as f32);
            }
        }
        
        correlation / samples as f32
    }
    
    /// Remappe les ROMs actuelles (après changement de configuration)
    fn remap_current_roms(&mut self) -> Result<()> {
        if let Some(rom_set) = &self.current_rom_set {
            // Pour une implémentation complète, on aurait besoin d'une référence au système mémoire
            println!("Remapping nécessaire après changement de configuration");
            // self.load_rom_set(rom_set.clone(), memory)?;
        }
        Ok(())
    }
    
    /// Obtient les informations sur le mapping actuel
    pub fn get_mapping_info(&self) -> Option<MappingInfo> {
        self.current_rom_set.as_ref().map(|rom_set| {
            let mut regions = Vec::new();
            
            for (rom_name, loaded_rom) in &rom_set.roms {
                let base_address = self.calculate_base_address(&loaded_rom.info.rom_type);
                let final_address = base_address + (loaded_rom.info.bank as u32 * self.mapping_config.bank_size);
                
                regions.push((
                    rom_name.clone(),
                    final_address,
                    loaded_rom.data.len(),
                    loaded_rom.info.rom_type.clone(),
                ));
            }
            
            MappingInfo {
                game_name: rom_set.game_info.name.clone(),
                total_roms: rom_set.roms.len(),
                total_size: regions.iter().map(|(_, _, size, _)| size).sum(),
                regions,
            }
        })
    }
    
    /// Lecture rapide depuis le cache ROM
    pub fn read_rom_data(&self, address: u32, size: usize) -> Option<Vec<u8>> {
        // Trouver la région contenant l'adresse
        for (&base_addr, data) in &self.mapped_data {
            if address >= base_addr && address + size as u32 <= base_addr + data.len() as u32 {
                let offset = (address - base_addr) as usize;
                return Some(data[offset..offset + size].to_vec());
            }
        }
        None
    }
    
    /// Valide la cohérence du mapping mémoire
    pub fn validate_mapping(&self) -> Result<ValidationReport> {
        let mut report = ValidationReport {
            is_valid: true,
            warnings: Vec::new(),
            errors: Vec::new(),
            statistics: MappingStatistics::default(),
        };
        
        if let Some(info) = self.get_mapping_info() {
            report.statistics.total_roms = info.total_roms;
            report.statistics.total_size = info.total_size;
            
            // Vérifier les chevauchements
            let mut sorted_regions: Vec<_> = info.regions.iter().collect();
            sorted_regions.sort_by_key(|(_, addr, _, _)| *addr);
            
            for i in 1..sorted_regions.len() {
                let (_, addr1, size1, _) = sorted_regions[i-1];
                let (name2, addr2, _, _) = sorted_regions[i];
                
                if addr1 + *size1 as u32 > *addr2 {
                    report.errors.push(format!("Chevauchement détecté avec ROM {}", name2));
                    report.is_valid = false;
                }
            }
            
            // Statistiques par type
            for (_, _, size, rom_type) in &info.regions {
                match rom_type {
                    RomType::Program => report.statistics.program_size += size,
                    RomType::Graphics | RomType::Geometry | RomType::Texture => report.statistics.graphics_size += size,
                    RomType::Sound | RomType::Samples => report.statistics.audio_size += size,
                    RomType::Data | RomType::Config | RomType::Microcode => report.statistics.data_size += size,
                }
            }
        } else {
            report.warnings.push("Aucune ROM mappée".to_string());
        }
        
        Ok(report)
    }
}

/// Informations sur le mapping actuel
#[derive(Debug)]
pub struct MappingInfo {
    pub game_name: String,
    pub total_roms: usize,
    pub total_size: usize,
    pub regions: Vec<(String, u32, usize, RomType)>, // nom, adresse, taille, type
}

/// Rapport de validation du mapping
#[derive(Debug)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub statistics: MappingStatistics,
}

/// Statistiques de mapping
#[derive(Debug, Default)]
pub struct MappingStatistics {
    pub total_roms: usize,
    pub total_size: usize,
    pub program_size: usize,
    pub graphics_size: usize,
    pub audio_size: usize,
    pub data_size: usize,
}

/// Calcule l'entropie des données (0.0 à 1.0)
fn calculate_entropy(data: &[u8]) -> f32 {
    let mut freq = [0u32; 256];
    
    // Compter les fréquences
    for &byte in data {
        freq[byte as usize] += 1;
    }
    
    // Calculer l'entropie de Shannon
    let len = data.len() as f32;
    let mut entropy = 0.0;
    
    for &count in &freq {
        if count > 0 {
            let p = count as f32 / len;
            entropy -= p * p.log2();
        }
    }
    
    // Normaliser entre 0 et 1
    entropy / 8.0 // log2(256) = 8
}

impl Default for RomMemoryMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_mapper_creation() {
        let mapper = RomMemoryMapper::new();
        assert!(mapper.current_rom_set.is_none());
        assert!(mapper.mapped_data.is_empty());
    }

    #[test]
    fn test_calculate_base_address() {
        let mapper = RomMemoryMapper::new();
        
        assert_eq!(mapper.calculate_base_address(&RomType::Program), 0x00000000);
        assert_eq!(mapper.calculate_base_address(&RomType::Graphics), 0x08000000);
        assert_eq!(mapper.calculate_base_address(&RomType::Sound), 0x10000000);
        assert_eq!(mapper.calculate_base_address(&RomType::Data), 0x18000000);
    }

    #[test]
    fn test_entropy_calculation() {
        // Données uniformes (haute entropie)
        let uniform_data: Vec<u8> = (0..255).collect();
        let entropy = calculate_entropy(&uniform_data);
        assert!(entropy > 0.9);
        
        // Données répétitives (basse entropie)
        let repetitive_data = vec![0u8; 1000];
        let entropy = calculate_entropy(&repetitive_data);
        assert!(entropy < 0.1);
    }

    #[test]
    fn test_model2_memory_config() {
        let config = Model2MemoryConfig::default();
        
        assert_eq!(config.program_rom_base, 0x00000000);
        assert_eq!(config.bank_size, 0x100000);
        assert_eq!(config.bank_mask, 0x0FFFFF);
    }

    #[test]
    fn test_mapping_statistics() {
        let mut stats = MappingStatistics::default();
        stats.total_roms = 5;
        stats.program_size = 1024 * 1024;
        stats.graphics_size = 2048 * 1024;
        
        assert_eq!(stats.total_roms, 5);
        assert_eq!(stats.program_size + stats.graphics_size, 3072 * 1024);
    }
}