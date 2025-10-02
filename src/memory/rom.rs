//! Implémentation de la mémoire ROM (Read-Only Memory)

use super::interface::MemoryInterface;
use anyhow::{anyhow, Result};

/// Structure représentant une zone de ROM
#[derive(Debug, Clone)]
pub struct Rom {
    /// Données de la ROM
    data: Vec<u8>,

    /// Taille de la ROM en octets
    size: usize,

    /// Nom/identifiant de la ROM
    name: String,

    /// Checksum pour vérification d'intégrité
    checksum: u32,
}

impl Rom {
    /// Crée une nouvelle ROM à partir de données
    pub fn new(data: Vec<u8>) -> Self {
        let size = data.len();
        let checksum = Self::calculate_checksum(&data);

        Self {
            data,
            size,
            name: String::new(),
            checksum,
        }
    }

    /// Crée une ROM avec un nom spécifique
    pub fn with_name(data: Vec<u8>, name: String) -> Self {
        let mut rom = Self::new(data);
        rom.name = name;
        rom
    }

    /// Charge une ROM depuis un fichier
    pub fn from_file(path: &str) -> Result<Self> {
        let data = std::fs::read(path)?;
        let name = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Self::with_name(data, name))
    }

    /// Obtient la taille de la ROM
    pub fn size(&self) -> usize {
        self.size
    }

    /// Obtient le nom de la ROM
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Obtient le checksum de la ROM
    pub fn checksum(&self) -> u32 {
        self.checksum
    }

    /// Vérifie l'intégrité de la ROM
    pub fn verify_integrity(&self) -> bool {
        Self::calculate_checksum(&self.data) == self.checksum
    }

    /// Calcule un checksum simple (CRC-32 simplifié)
    fn calculate_checksum(data: &[u8]) -> u32 {
        let mut checksum = 0u32;
        for chunk in data.chunks(4) {
            let mut value = 0u32;
            for (i, &byte) in chunk.iter().enumerate() {
                value |= (byte as u32) << (i * 8);
            }
            checksum = checksum.wrapping_add(value);
        }
        checksum
    }

    /// Vérifie qu'une adresse est valide
    fn check_bounds(&self, address: u32, size: usize) -> Result<()> {
        let addr = address as usize;
        if addr + size > self.size {
            Err(anyhow!(
                "Accès ROM hors limites: {:#08x} + {} > {:#08x}",
                address,
                size,
                self.size
            ))
        } else {
            Ok(())
        }
    }

    /// Recherche un pattern de bytes dans la ROM
    pub fn find_pattern(&self, pattern: &[u8]) -> Vec<usize> {
        let mut matches = Vec::new();

        if pattern.is_empty() || pattern.len() > self.data.len() {
            return matches;
        }

        for i in 0..=(self.data.len() - pattern.len()) {
            if self.data[i..i + pattern.len()] == *pattern {
                matches.push(i);
            }
        }

        matches
    }

    /// Obtient des informations sur la ROM
    pub fn get_info(&self) -> RomInfo {
        RomInfo {
            name: self.name.clone(),
            size: self.size,
            checksum: self.checksum,
            is_valid: self.verify_integrity(),
        }
    }
}

impl MemoryInterface for Rom {
    fn read_u8(&self, address: u32) -> Result<u8> {
        self.check_bounds(address, 1)?;
        Ok(self.data[address as usize])
    }

    fn read_u16(&self, address: u32) -> Result<u16> {
        self.check_bounds(address, 2)?;
        let addr = address as usize;

        // Lecture little-endian
        let low = self.data[addr] as u16;
        let high = self.data[addr + 1] as u16;
        Ok(low | (high << 8))
    }

    fn read_u32(&self, address: u32) -> Result<u32> {
        self.check_bounds(address, 4)?;
        let addr = address as usize;

        // Lecture little-endian
        let b0 = self.data[addr] as u32;
        let b1 = self.data[addr + 1] as u32;
        let b2 = self.data[addr + 2] as u32;
        let b3 = self.data[addr + 3] as u32;
        Ok(b0 | (b1 << 8) | (b2 << 16) | (b3 << 24))
    }

    fn write_u8(&mut self, _address: u32, _value: u8) -> Result<()> {
        // Les ROMs sont en lecture seule
        Err(anyhow!("Tentative d'écriture dans une ROM"))
    }

    fn write_u16(&mut self, _address: u32, _value: u16) -> Result<()> {
        Err(anyhow!("Tentative d'écriture dans une ROM"))
    }

    fn write_u32(&mut self, _address: u32, _value: u32) -> Result<()> {
        Err(anyhow!("Tentative d'écriture dans une ROM"))
    }

    fn read_block(&self, address: u32, size: usize) -> Result<Vec<u8>> {
        self.check_bounds(address, size)?;
        let addr = address as usize;
        Ok(self.data[addr..addr + size].to_vec())
    }

    fn write_block(&mut self, _address: u32, _data: &[u8]) -> Result<()> {
        Err(anyhow!("Tentative d'écriture dans une ROM"))
    }

    fn fill(&mut self, _address: u32, _size: usize, _value: u8) -> Result<()> {
        Err(anyhow!("Tentative d'écriture dans une ROM"))
    }
}

/// Informations sur une ROM
#[derive(Debug, Clone)]
pub struct RomInfo {
    /// Nom de la ROM
    pub name: String,

    /// Taille en octets
    pub size: usize,

    /// Checksum
    pub checksum: u32,

    /// Indique si la ROM est valide
    pub is_valid: bool,
}

/// Collection de ROMs avec gestion des dépendances
#[derive(Debug)]
pub struct RomSet {
    /// ROMs du jeu
    roms: std::collections::HashMap<String, Rom>,

    /// Métadonnées du jeu
    game_info: GameInfo,
}

impl RomSet {
    /// Crée un nouveau set de ROMs vide
    pub fn new(game_info: GameInfo) -> Self {
        Self {
            roms: std::collections::HashMap::new(),
            game_info,
        }
    }

    /// Ajoute une ROM au set
    pub fn add_rom(&mut self, name: String, rom: Rom) {
        self.roms.insert(name, rom);
    }

    /// Obtient une ROM par son nom
    pub fn get_rom(&self, name: &str) -> Option<&Rom> {
        self.roms.get(name)
    }

    /// Vérifie que toutes les ROMs requises sont présentes
    pub fn verify_completeness(&self) -> Result<()> {
        for required_rom in &self.game_info.required_roms {
            if !self.roms.contains_key(required_rom) {
                return Err(anyhow!("ROM manquante: {}", required_rom));
            }
        }
        Ok(())
    }

    /// Vérifie l'intégrité de toutes les ROMs
    pub fn verify_integrity(&self) -> Result<()> {
        for (name, rom) in &self.roms {
            if !rom.verify_integrity() {
                return Err(anyhow!("ROM corrompue: {}", name));
            }
        }
        Ok(())
    }

    /// Obtient les informations du jeu
    pub fn game_info(&self) -> &GameInfo {
        &self.game_info
    }

    /// Liste toutes les ROMs chargées
    pub fn list_roms(&self) -> Vec<(&String, &Rom)> {
        self.roms.iter().collect()
    }
}

/// Informations sur un jeu Model 2
#[derive(Debug, Clone)]
pub struct GameInfo {
    /// Nom du jeu
    pub name: String,

    /// Nom court/identifiant
    pub short_name: String,

    /// Année de sortie
    pub year: u16,

    /// Éditeur
    pub publisher: String,

    /// ROMs requises
    pub required_roms: Vec<String>,

    /// ROMs optionnelles
    pub optional_roms: Vec<String>,

    /// Configuration spéciale requise
    pub special_config: Option<String>,
}

impl GameInfo {
    /// Crée des informations de jeu par défaut
    pub fn unknown() -> Self {
        Self {
            name: "Jeu inconnu".to_string(),
            short_name: "unknown".to_string(),
            year: 1994,
            publisher: "SEGA".to_string(),
            required_roms: vec!["program.rom".to_string(), "graphics.rom".to_string()],
            optional_roms: vec!["audio.rom".to_string()],
            special_config: None,
        }
    }

    /// Crée les informations pour Virtua Fighter 2
    pub fn virtua_fighter_2() -> Self {
        Self {
            name: "Virtua Fighter 2".to_string(),
            short_name: "vf2".to_string(),
            year: 1994,
            publisher: "SEGA".to_string(),
            required_roms: vec![
                "epr-17662.ic31".to_string(),
                "epr-17663.ic28".to_string(),
                "mpr-17650.ic14".to_string(),
                "mpr-17647.ic18".to_string(),
            ],
            optional_roms: vec!["epr-17664.ic4".to_string()],
            special_config: Some("fighter_config".to_string()),
        }
    }

    /// Crée les informations pour Daytona USA
    pub fn daytona_usa() -> Self {
        Self {
            name: "Daytona USA".to_string(),
            short_name: "daytona".to_string(),
            year: 1994,
            publisher: "SEGA".to_string(),
            required_roms: vec![
                "epr-16722a.ic31".to_string(),
                "epr-16723a.ic28".to_string(),
                "mpr-16710.ic18".to_string(),
                "mpr-16708.ic14".to_string(),
            ],
            optional_roms: vec!["epr-16724.ic4".to_string()],
            special_config: Some("racing_config".to_string()),
        }
    }
}
