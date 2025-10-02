//! Mapping mémoire du SEGA Model 2

/// Régions mémoire du Model 2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRegion {
    /// RAM principale (8MB)
    MainRam,

    /// VRAM (4MB)
    VideoRam,

    /// RAM audio (512KB)
    AudioRam,

    /// ROM du programme principal
    ProgramRom,

    /// ROM des graphiques
    GraphicsRom,

    /// ROM audio
    AudioRom,

    /// Registres d'entrée/sortie
    IoRegisters,
}

/// Entrée de mapping mémoire
#[derive(Debug, Clone)]
pub struct MemoryMapEntry {
    /// Adresse de début
    pub start: u32,

    /// Adresse de fin (exclusive)
    pub end: u32,

    /// Région mémoire correspondante
    pub region: MemoryRegion,

    /// Offset dans la région (pour gérer les miroirs)
    pub offset: u32,

    /// Taille de la région réelle
    pub size: u32,

    /// Indique si la région est accessible en écriture
    pub writable: bool,
}

impl MemoryMapEntry {
    /// Crée une nouvelle entrée de mapping
    pub fn new(
        start: u32,
        end: u32,
        region: MemoryRegion,
        offset: u32,
        size: u32,
        writable: bool,
    ) -> Self {
        Self {
            start,
            end,
            region,
            offset,
            size,
            writable,
        }
    }

    /// Vérifie si une adresse est dans cette région
    pub fn contains(&self, address: u32) -> bool {
        address >= self.start && address < self.end
    }

    /// Convertit une adresse globale en offset local avec gestion des miroirs
    pub fn to_local_offset(&self, address: u32) -> Option<u32> {
        if !self.contains(address) {
            return None;
        }

        let local_addr = address - self.start + self.offset;

        // Gestion des miroirs - l'adresse est repliée sur la taille réelle
        Some(local_addr % self.size)
    }
}

/// Table de mapping mémoire complète
#[derive(Debug)]
pub struct MemoryMap {
    entries: Vec<MemoryMapEntry>,
}

impl MemoryMap {
    /// Crée un nouveau mapping mémoire vide
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Crée le mapping mémoire standard du SEGA Model 2
    pub fn new_model2() -> Self {
        let mut map = Self::new();

        // RAM principale - 8MB à partir de 0x00000000
        // Avec miroirs pour compatibilité
        map.add_entry(MemoryMapEntry::new(
            0x00000000,
            0x00800000, // 8MB
            MemoryRegion::MainRam,
            0,
            0x00800000, // 8MB réels
            true,
        ));

        // Miroir de la RAM principale
        map.add_entry(MemoryMapEntry::new(
            0x00800000,
            0x01000000, // Miroir 8MB
            MemoryRegion::MainRam,
            0,
            0x00800000, // Taille réelle 8MB
            true,
        ));

        // ROM du programme principal - typiquement à 0x02000000
        map.add_entry(MemoryMapEntry::new(
            0x02000000,
            0x02800000, // 8MB d'espace ROM
            MemoryRegion::ProgramRom,
            0,
            0x00800000, // Taille max 8MB
            false,
        ));

        // VRAM - 4MB à partir de 0x10000000
        map.add_entry(MemoryMapEntry::new(
            0x10000000,
            0x10400000, // 4MB
            MemoryRegion::VideoRam,
            0,
            0x00400000, // 4MB réels
            true,
        ));

        // Miroir VRAM
        map.add_entry(MemoryMapEntry::new(
            0x10400000,
            0x10800000, // Miroir 4MB
            MemoryRegion::VideoRam,
            0,
            0x00400000, // Taille réelle 4MB
            true,
        ));

        // ROM graphiques - typiquement à 0x20000000
        map.add_entry(MemoryMapEntry::new(
            0x20000000,
            0x24000000, // 64MB d'espace pour les ROMs graphiques
            MemoryRegion::GraphicsRom,
            0,
            0x04000000, // Taille max 64MB
            false,
        ));

        // RAM audio - 512KB à partir de 0x30000000
        map.add_entry(MemoryMapEntry::new(
            0x30000000,
            0x30080000, // 512KB
            MemoryRegion::AudioRam,
            0,
            0x00080000, // 512KB réels
            true,
        ));

        // ROM audio - typiquement à 0x31000000
        map.add_entry(MemoryMapEntry::new(
            0x31000000,
            0x31800000, // 8MB d'espace pour ROM audio
            MemoryRegion::AudioRom,
            0,
            0x00800000, // Taille max 8MB
            false,
        ));

        // Registres I/O - zone haute de la mémoire
        map.add_entry(MemoryMapEntry::new(
            0xF0000000,
            0xF0001000, // 4KB de registres
            MemoryRegion::IoRegisters,
            0,
            0x00001000, // 4KB
            true,
        ));

        // Trier les entrées par adresse de début pour optimiser la recherche
        map.entries.sort_by_key(|entry| entry.start);

        map
    }

    /// Ajoute une entrée au mapping
    pub fn add_entry(&mut self, entry: MemoryMapEntry) {
        self.entries.push(entry);
        // Re-trier après ajout
        self.entries.sort_by_key(|entry| entry.start);
    }

    /// Résout une adresse vers sa région et son offset local
    pub fn resolve(&self, address: u32) -> Option<(MemoryRegion, u32)> {
        // Recherche binaire pour optimiser la performance
        match self.entries.binary_search_by(|entry| {
            if address < entry.start {
                std::cmp::Ordering::Greater
            } else if address >= entry.end {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        }) {
            Ok(index) => {
                let entry = &self.entries[index];
                entry
                    .to_local_offset(address)
                    .map(|offset| (entry.region, offset))
            }
            Err(_) => None,
        }
    }

    /// Vérifie si une adresse est accessible en écriture
    pub fn is_writable(&self, address: u32) -> bool {
        self.entries
            .iter()
            .find(|entry| entry.contains(address))
            .map(|entry| entry.writable)
            .unwrap_or(false)
    }

    /// Obtient des informations sur une région mémoire
    pub fn get_region_info(&self, address: u32) -> Option<&MemoryMapEntry> {
        self.entries.iter().find(|entry| entry.contains(address))
    }

    /// Liste toutes les régions mappées
    pub fn list_regions(&self) -> Vec<(MemoryRegion, u32, u32)> {
        self.entries
            .iter()
            .map(|entry| (entry.region, entry.start, entry.end))
            .collect()
    }
}

impl Default for MemoryMap {
    fn default() -> Self {
        Self::new()
    }
}
