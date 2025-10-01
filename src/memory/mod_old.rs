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

use anyhow::Result;
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
    
    /// Zones ROM
    pub roms: HashMap<String, Rom>,
    
    /// Mapping des adresses mémoire
    pub memory_map: MemoryMap,
    
    /// Cache pour optimiser les accès
    cache: RefCell<MemoryCache>,
}

impl Model2Memory {
    /// Crée une nouvelle instance du système mémoire Model 2
    pub fn new() -> Self {
        Self {
            main_ram: Ram::new(crate::MAIN_RAM_SIZE),
            video_ram: Ram::new(crate::VIDEO_RAM_SIZE),
            audio_ram: Ram::new(crate::AUDIO_RAM_SIZE),
            roms: HashMap::new(),
            memory_map: MemoryMap::new_model2(),
            cache: RefCell::new(MemoryCache::new()),
        }
    }

    /// Charge une ROM dans la mémoire
    pub fn load_rom(&mut self, name: String, data: Vec<u8>) -> Result<()> {
        let rom = Rom::new(data);
        self.roms.insert(name, rom);
        Ok(())
    }

    /// Efface le cache mémoire
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Résout une adresse physique vers son type et offset
    fn resolve_address(&self, address: u32) -> Option<(MemoryRegion, u32)> {
        self.memory_map.resolve(address)
    }
}

impl MemoryInterface for Model2Memory {
    fn read_u8(&self, address: u32) -> Result<u8> {
        // Vérifier le cache d'abord
        if let Some(value) = self.cache.get_u8(address) {
            return Ok(value);
        }

        let result = if let Some((region, offset)) = self.resolve_address(address) {
            match region {
                MemoryRegion::MainRam => self.main_ram.read_u8(offset),
                MemoryRegion::VideoRam => self.video_ram.read_u8(offset),
                MemoryRegion::AudioRam => self.audio_ram.read_u8(offset),
                MemoryRegion::ProgramRom => {
                    if let Some(rom) = self.roms.get("program") {
                        rom.read_u8(offset)
                    } else {
                        Ok(0xFF) // Valeur par défaut si ROM non chargée
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

            let result = if let Some((region, offset)) = self.resolve_address(address) {
                match region {
                    MemoryRegion::MainRam => self.main_ram.read_u16(offset),
                    MemoryRegion::VideoRam => self.video_ram.read_u16(offset),
                    MemoryRegion::AudioRam => self.audio_ram.read_u16(offset),
                    MemoryRegion::ProgramRom => {
                        if let Some(rom) = self.roms.get("program") {
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
                    MemoryRegion::IoRegisters => Ok(0x0000),
                }
            } else {
                Ok(0xFFFF)
            };

            if let Ok(value) = result {
                self.cache.set_u16(address, value);
            }

            result
        } else {
            // Lecture non alignée - utiliser deux lectures d'octets
            let low = self.read_u8(address)? as u16;
            let high = self.read_u8(address + 1)? as u16;
            Ok(low | (high << 8)) // Little endian
        }
    }

    fn read_u32(&self, address: u32) -> Result<u32> {
        // Optimisation pour les accès alignés
        if address % 4 == 0 {
            if let Ok(cache) = self.cache.try_borrow() {
                if let Some(value) = cache.get_u32(address) {
                    return Ok(value);
                }
            }
            }

            let result = if let Some((region, offset)) = self.resolve_address(address) {
                match region {
                    MemoryRegion::MainRam => self.main_ram.read_u32(offset),
                    MemoryRegion::VideoRam => self.video_ram.read_u32(offset),
                    MemoryRegion::AudioRam => self.audio_ram.read_u32(offset),
                    MemoryRegion::ProgramRom => {
                        if let Some(rom) = self.roms.get("program") {
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
                    MemoryRegion::IoRegisters => Ok(0x00000000),
                }
            } else {
                Ok(0xFFFFFFFF)
            };

            if let Ok(value) = result {
                if let Ok(mut cache) = self.cache.try_borrow_mut() {
                        cache.set_u32(address, value);
                    }
            }

            result
        } else {
            // Lecture non alignée
            let b0 = self.read_u8(address)? as u32;
            let b1 = self.read_u8(address + 1)? as u32;
            let b2 = self.read_u8(address + 2)? as u32;
            let b3 = self.read_u8(address + 3)? as u32;
            Ok(b0 | (b1 << 8) | (b2 << 16) | (b3 << 24)) // Little endian
        }
    }

    fn write_u8(&mut self, address: u32, value: u8) -> Result<()> {
        // Invalider le cache pour cette adresse
        self.cache.invalidate(address);

        if let Some((region, offset)) = self.resolve_address(address) {
            match region {
                MemoryRegion::MainRam => self.main_ram.write_u8(offset, value),
                MemoryRegion::VideoRam => self.video_ram.write_u8(offset, value),
                MemoryRegion::AudioRam => self.audio_ram.write_u8(offset, value),
                MemoryRegion::ProgramRom |
                MemoryRegion::GraphicsRom |
                MemoryRegion::AudioRom => {
                    // Les ROMs sont en lecture seule - ignorer silencieusement
                    Ok(())
                },
                MemoryRegion::IoRegisters => {
                    // Traitement des registres I/O (à implémenter)
                    Ok(())
                },
            }
        } else {
            // Écriture dans une zone non mappée - ignorer
            Ok(())
        }
    }

    fn write_u16(&mut self, address: u32, value: u16) -> Result<()> {
        self.cache.invalidate(address);
        self.cache.invalidate(address + 1);

        if address % 2 == 0 {
            // Écriture alignée
            if let Some((region, offset)) = self.resolve_address(address) {
                match region {
                    MemoryRegion::MainRam => self.main_ram.write_u16(offset, value),
                    MemoryRegion::VideoRam => self.video_ram.write_u16(offset, value),
                    MemoryRegion::AudioRam => self.audio_ram.write_u16(offset, value),
                    MemoryRegion::ProgramRom |
                    MemoryRegion::GraphicsRom |
                    MemoryRegion::AudioRom => Ok(()),
                    MemoryRegion::IoRegisters => Ok(()),
                }
            } else {
                Ok(())
            }
        } else {
            // Écriture non alignée
            self.write_u8(address, value as u8)?;
            self.write_u8(address + 1, (value >> 8) as u8)?;
            Ok(())
        }
    }

    fn write_u32(&mut self, address: u32, value: u32) -> Result<()> {
        self.cache.invalidate(address);
        self.cache.invalidate(address + 1);
        self.cache.invalidate(address + 2);
        self.cache.invalidate(address + 3);

        if address % 4 == 0 {
            // Écriture alignée
            if let Some((region, offset)) = self.resolve_address(address) {
                match region {
                    MemoryRegion::MainRam => self.main_ram.write_u32(offset, value),
                    MemoryRegion::VideoRam => self.video_ram.write_u32(offset, value),
                    MemoryRegion::AudioRam => self.audio_ram.write_u32(offset, value),
                    MemoryRegion::ProgramRom |
                    MemoryRegion::GraphicsRom |
                    MemoryRegion::AudioRom => Ok(()),
                    MemoryRegion::IoRegisters => Ok(()),
                }
            } else {
                Ok(())
            }
        } else {
            // Écriture non alignée
            self.write_u8(address, value as u8)?;
            self.write_u8(address + 1, (value >> 8) as u8)?;
            self.write_u8(address + 2, (value >> 16) as u8)?;
            self.write_u8(address + 3, (value >> 24) as u8)?;
            Ok(())
        }
    }
}

impl Default for Model2Memory {
    fn default() -> Self {
        Self::new()
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
        if self.entries.len() >= self.max_entries {
            // Éviction simple : supprimer une entrée aléatoire
            if let Some(&key) = self.entries.keys().next() {
                self.entries.remove(&key);
            }
        }
        self.entries.insert(address, entry);
    }

    fn invalidate(&mut self, address: u32) {
        self.entries.remove(&address);
    }

    fn clear(&mut self) {
        self.entries.clear();
    }
}