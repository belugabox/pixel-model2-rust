//! Système de gestion mémoire pour le SEGA Model 2
//! 
//! Le Model 2 possède plusieurs types de mémoire mappée :
//! - RAM principale (8MB)
//! - VRAM (4MB) 
//! - RAM audio (512KB)
//! - Zones ROM
//! - Registres I/O

pub mod interface;
pub mod mapping;
pub mod ram;
pub mod rom;

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::cell::RefCell;

pub use interface::*;
pub use mapping::*;
pub use ram::*;
pub use rom::*;

/// Bus mémoire principal du SEGA Model 2
#[derive(Debug)]
pub struct Model2Memory {
    /// RAM principale (8MB)
    pub main_ram: Ram,
    
    /// VRAM (4MB)
    pub video_ram: Ram,
    
    /// RAM audio (512KB)
    pub audio_ram: Ram,
    
    /// Gestionnaire de mappage mémoire
    pub mapping: MemoryMap,
    
    /// ROMs chargées
    pub roms: HashMap<String, Rom>,
    
    /// Cache des accès mémoire pour optimisation
    cache: RefCell<MemoryCache>,
    
    /// Activation du cache
    cache_enabled: bool,
}

impl Model2Memory {
    /// Crée un nouveau système mémoire Model 2
    pub fn new() -> Self {
        Self {
            main_ram: Ram::new(8 * 1024 * 1024), // 8MB
            video_ram: Ram::new(4 * 1024 * 1024), // 4MB
            audio_ram: Ram::new(512 * 1024), // 512KB
            mapping: MemoryMap::new_model2(),
            roms: HashMap::new(),
            cache: RefCell::new(MemoryCache::new()),
            cache_enabled: true,
        }
    }
    
    /// Charge une ROM dans le système
    pub fn load_rom(&mut self, name: String, data: Vec<u8>) -> Result<()> {
        let rom = Rom::new(data);
        self.roms.insert(name, rom);
        Ok(())
    }
    
    /// Vide le cache mémoire
    pub fn clear_cache(&mut self) {
        if let Ok(mut cache) = self.cache.try_borrow_mut() {
            cache.clear();
        }
    }
}

impl MemoryInterface for Model2Memory {
    fn read_u8(&self, address: u32) -> Result<u8> {
        // Vérifier le cache d'abord
        if self.cache_enabled {
            if let Ok(cache) = self.cache.try_borrow() {
                if let Some(value) = cache.get_u8(address) {
                    return Ok(value);
                }
            }
        }
        
        // Déterminer la région mémoire et l'offset
        let result = if let Some((region, offset)) = self.mapping.resolve(address) {
            match region {
                MemoryRegion::MainRam => self.main_ram.read_u8(offset),
                MemoryRegion::VideoRam => self.video_ram.read_u8(offset),
                MemoryRegion::AudioRam => self.audio_ram.read_u8(offset),
                MemoryRegion::ProgramRom => {
                    if let Some(rom) = self.roms.get("main") {
                        rom.read_u8(offset)
                    } else {
                        Ok(0xFF)
                    }
                },
                MemoryRegion::GraphicsRom => {
                    if let Some(rom) = self.roms.get("graphics") {
                        rom.read_u8(offset)
                    } else {
                        Ok(0xFF)
                    }
                },
                MemoryRegion::AudioRom => {
                    if let Some(rom) = self.roms.get("audio") {
                        rom.read_u8(offset)
                    } else {
                        Ok(0xFF)
                    }
                },
                MemoryRegion::IoRegisters => {
                    // Simulation simple des registres I/O
                    Ok(0x00)
                },
            }
        } else {
            Ok(0xFF) // Lecture dans une zone non mappée
        };

        // Mettre en cache le résultat si valide
        if let Ok(value) = result {
            if let Ok(mut cache) = self.cache.try_borrow_mut() {
                cache.set_u8(address, value);
            }
        }

        result
    }

    fn read_u16(&self, address: u32) -> Result<u16> {
        // Optimisation : lecture directe pour les accès alignés
        if address % 2 == 0 {
            if let Ok(cache) = self.cache.try_borrow() {
                if let Some(value) = cache.get_u16(address) {
                    return Ok(value);
                }
            }
        }
        
        // Déterminer la région mémoire et l'offset
        let result = if let Some((region, offset)) = self.mapping.resolve(address) {
            match region {
                MemoryRegion::MainRam => self.main_ram.read_u16(offset),
                MemoryRegion::VideoRam => self.video_ram.read_u16(offset),
                MemoryRegion::AudioRam => self.audio_ram.read_u16(offset),
                MemoryRegion::ProgramRom => {
                    if let Some(rom) = self.roms.get("main") {
                        rom.read_u16(offset)
                    } else {
                        Ok(0xFFFF)
                    }
                },
                MemoryRegion::GraphicsRom => {
                    if let Some(rom) = self.roms.get("graphics") {
                        rom.read_u16(offset)
                    } else {
                        Ok(0xFFFF)
                    }
                },
                MemoryRegion::AudioRom => {
                    if let Some(rom) = self.roms.get("audio") {
                        rom.read_u16(offset)
                    } else {
                        Ok(0xFFFF)
                    }
                },
                MemoryRegion::IoRegisters => {
                    // Simulation simple des registres I/O
                    Ok(0x0000)
                },
            }
        } else {
            Ok(0xFFFF) // Lecture dans une zone non mappée
        };

        // Mettre en cache le résultat si valide
        if let Ok(value) = result {
            if let Ok(mut cache) = self.cache.try_borrow_mut() {
                cache.set_u16(address, value);
            }
        }

        result
    }

    fn read_u32(&self, address: u32) -> Result<u32> {
        // Optimisation : lecture directe pour les accès alignés
        if address % 4 == 0 {
            if let Ok(cache) = self.cache.try_borrow() {
                if let Some(value) = cache.get_u32(address) {
                    return Ok(value);
                }
            }
        }
        
        // Déterminer la région mémoire et l'offset
        let result = if let Some((region, offset)) = self.mapping.resolve(address) {
            match region {
                MemoryRegion::MainRam => self.main_ram.read_u32(offset),
                MemoryRegion::VideoRam => self.video_ram.read_u32(offset),
                MemoryRegion::AudioRam => self.audio_ram.read_u32(offset),
                MemoryRegion::ProgramRom => {
                    if let Some(rom) = self.roms.get("main") {
                        rom.read_u32(offset)
                    } else {
                        Ok(0xFFFFFFFF)
                    }
                },
                MemoryRegion::GraphicsRom => {
                    if let Some(rom) = self.roms.get("graphics") {
                        rom.read_u32(offset)
                    } else {
                        Ok(0xFFFFFFFF)
                    }
                },
                MemoryRegion::AudioRom => {
                    if let Some(rom) = self.roms.get("audio") {
                        rom.read_u32(offset)
                    } else {
                        Ok(0xFFFFFFFF)
                    }
                },
                MemoryRegion::IoRegisters => {
                    // Simulation simple des registres I/O
                    Ok(0x00000000)
                },
            }
        } else {
            Ok(0xFFFFFFFF) // Lecture dans une zone non mappée
        };

        // Mettre en cache le résultat si valide
        if let Ok(value) = result {
            if let Ok(mut cache) = self.cache.try_borrow_mut() {
                cache.set_u32(address, value);
            }
        }

        result
    }

    fn write_u8(&mut self, address: u32, value: u8) -> Result<()> {
        // Déterminer la région mémoire et l'offset
        if let Some((region, offset)) = self.mapping.resolve(address) {
            match region {
                MemoryRegion::MainRam => self.main_ram.write_u8(offset, value),
                MemoryRegion::VideoRam => self.video_ram.write_u8(offset, value),
                MemoryRegion::AudioRam => self.audio_ram.write_u8(offset, value),
                MemoryRegion::ProgramRom | MemoryRegion::GraphicsRom | MemoryRegion::AudioRom => {
                    // Les ROMs sont en lecture seule
                    Err(anyhow!("Tentative d'écriture en ROM à l'adresse {:08X}", address))
                },
                MemoryRegion::IoRegisters => {
                    // Gestion basique des registres I/O
                    // TODO: Implémenter la gestion spécifique des registres
                    Ok(())
                },
            }
        } else {
            // Écriture dans une zone non mappée - ignorer silencieusement
            Ok(())
        }
    }

    fn write_u16(&mut self, address: u32, value: u16) -> Result<()> {
        // Alignement vérifié
        if address % 2 != 0 {
            return Err(anyhow!("Écriture u16 non alignée à l'adresse {:08X}", address));
        }
        
        // Déterminer la région mémoire et l'offset
        if let Some((region, offset)) = self.mapping.resolve(address) {
            match region {
                MemoryRegion::MainRam => self.main_ram.write_u16(offset, value),
                MemoryRegion::VideoRam => self.video_ram.write_u16(offset, value),
                MemoryRegion::AudioRam => self.audio_ram.write_u16(offset, value),
                MemoryRegion::ProgramRom | MemoryRegion::GraphicsRom | MemoryRegion::AudioRom => {
                    // Les ROMs sont en lecture seule
                    Err(anyhow!("Tentative d'écriture en ROM à l'adresse {:08X}", address))
                },
                MemoryRegion::IoRegisters => {
                    // Gestion basique des registres I/O
                    Ok(())
                },
            }
        } else {
            // Écriture dans une zone non mappée - ignorer silencieusement
            Ok(())
        }
    }

    fn write_u32(&mut self, address: u32, value: u32) -> Result<()> {
        // Alignement vérifié
        if address % 4 != 0 {
            return Err(anyhow!("Écriture u32 non alignée à l'adresse {:08X}", address));
        }
        
        // Déterminer la région mémoire et l'offset
        if let Some((region, offset)) = self.mapping.resolve(address) {
            match region {
                MemoryRegion::MainRam => self.main_ram.write_u32(offset, value),
                MemoryRegion::VideoRam => self.video_ram.write_u32(offset, value),
                MemoryRegion::AudioRam => self.audio_ram.write_u32(offset, value),
                MemoryRegion::ProgramRom | MemoryRegion::GraphicsRom | MemoryRegion::AudioRom => {
                    // Les ROMs sont en lecture seule
                    Err(anyhow!("Tentative d'écriture en ROM à l'adresse {:08X}", address))
                },
                MemoryRegion::IoRegisters => {
                    // Gestion basique des registres I/O
                    Ok(())
                },
            }
        } else {
            // Écriture dans une zone non mappée - ignorer silencieusement
            Ok(())
        }
    }
}

/// Cache mémoire simple pour optimiser les performances
#[derive(Debug)]
struct MemoryCache {
    entries: HashMap<u32, CacheEntry>,
    max_entries: usize,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    value: u32,
    size: u8, // 1, 2, ou 4 octets
}

impl MemoryCache {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            max_entries: 1024, // Limiter la taille du cache
        }
    }

    fn get_u8(&self, address: u32) -> Option<u8> {
        self.entries.get(&address)
            .filter(|entry| entry.size == 1)
            .map(|entry| entry.value as u8)
    }

    fn get_u16(&self, address: u32) -> Option<u16> {
        self.entries.get(&address)
            .filter(|entry| entry.size == 2)
            .map(|entry| entry.value as u16)
    }

    fn get_u32(&self, address: u32) -> Option<u32> {
        self.entries.get(&address)
            .filter(|entry| entry.size == 4)
            .map(|entry| entry.value)
    }

    fn set_u8(&mut self, address: u32, value: u8) {
        self.insert_entry(address, CacheEntry { value: value as u32, size: 1 });
    }

    fn set_u16(&mut self, address: u32, value: u16) {
        self.insert_entry(address, CacheEntry { value: value as u32, size: 2 });
    }

    fn set_u32(&mut self, address: u32, value: u32) {
        self.insert_entry(address, CacheEntry { value, size: 4 });
    }

    fn insert_entry(&mut self, address: u32, entry: CacheEntry) {
        // Éviction si le cache est plein
        if self.entries.len() >= self.max_entries {
            // Stratégie simple : vider la moitié du cache
            let keys: Vec<u32> = self.entries.keys().take(self.max_entries / 2).cloned().collect();
            for key in keys {
                self.entries.remove(&key);
            }
        }
        
        self.entries.insert(address, entry);
    }

    fn clear(&mut self) {
        self.entries.clear();
    }
}