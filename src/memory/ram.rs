//! Implémentation de la mémoire RAM

use super::interface::MemoryInterface;
use anyhow::{Result, anyhow};

/// Structure représentant une zone de RAM
#[derive(Debug, Clone)]
pub struct Ram {
    /// Données de la mémoire
    data: Vec<u8>,
    
    /// Taille de la mémoire en octets
    size: usize,
    
    /// Statistiques d'accès pour l'optimisation
    stats: AccessStats,
}

impl Ram {
    /// Crée une nouvelle zone RAM de la taille spécifiée
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
            size,
            stats: AccessStats::new(),
        }
    }
    
    /// Crée une RAM initialisée avec des données
    pub fn from_data(data: Vec<u8>) -> Self {
        let size = data.len();
        Self {
            data,
            size,
            stats: AccessStats::new(),
        }
    }
    
    /// Obtient la taille de la RAM
    pub fn size(&self) -> usize {
        self.size
    }
    
    /// Efface le contenu de la RAM
    pub fn clear(&mut self) {
        self.data.fill(0);
        self.stats.reset();
    }
    
    /// Charge des données dans la RAM à partir d'un offset
    pub fn load_data(&mut self, offset: usize, data: &[u8]) -> Result<()> {
        if offset + data.len() > self.size {
            return Err(anyhow!("Tentative de chargement au-delà de la taille de la RAM"));
        }
        
        self.data[offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }
    
    /// Obtient les statistiques d'accès
    pub fn get_stats(&self) -> &AccessStats {
        &self.stats
    }
    
    /// Vérifie qu'une adresse est valide
    fn check_bounds(&self, address: u32, size: usize) -> Result<()> {
        let addr = address as usize;
        if addr + size > self.size {
            Err(anyhow!("Accès mémoire hors limites: {:#08x} + {} > {:#08x}", 
                       address, size, self.size))
        } else {
            Ok(())
        }
    }
}

impl MemoryInterface for Ram {
    fn read_u8(&self, address: u32) -> Result<u8> {
        self.check_bounds(address, 1)?;
        let value = self.data[address as usize];
        
        // Mise à jour des statistiques (en version non-mutable, on skip)
        // self.stats.record_read(1);
        
        Ok(value)
    }
    
    fn read_u16(&self, address: u32) -> Result<u16> {
        self.check_bounds(address, 2)?;
        let addr = address as usize;
        
        // Lecture little-endian
        let low = self.data[addr] as u16;
        let high = self.data[addr + 1] as u16;
        let value = low | (high << 8);
        
        Ok(value)
    }
    
    fn read_u32(&self, address: u32) -> Result<u32> {
        self.check_bounds(address, 4)?;
        let addr = address as usize;
        
        // Lecture little-endian
        let b0 = self.data[addr] as u32;
        let b1 = self.data[addr + 1] as u32;
        let b2 = self.data[addr + 2] as u32;
        let b3 = self.data[addr + 3] as u32;
        let value = b0 | (b1 << 8) | (b2 << 16) | (b3 << 24);
        
        Ok(value)
    }
    
    fn write_u8(&mut self, address: u32, value: u8) -> Result<()> {
        self.check_bounds(address, 1)?;
        self.data[address as usize] = value;
        self.stats.record_write(1);
        
        Ok(())
    }
    
    fn write_u16(&mut self, address: u32, value: u16) -> Result<()> {
        self.check_bounds(address, 2)?;
        let addr = address as usize;
        
        // Écriture little-endian
        self.data[addr] = value as u8;
        self.data[addr + 1] = (value >> 8) as u8;
        self.stats.record_write(2);
        
        Ok(())
    }
    
    fn write_u32(&mut self, address: u32, value: u32) -> Result<()> {
        self.check_bounds(address, 4)?;
        let addr = address as usize;
        
        // Écriture little-endian
        self.data[addr] = value as u8;
        self.data[addr + 1] = (value >> 8) as u8;
        self.data[addr + 2] = (value >> 16) as u8;
        self.data[addr + 3] = (value >> 24) as u8;
        self.stats.record_write(4);
        
        Ok(())
    }
    
    fn read_block(&self, address: u32, size: usize) -> Result<Vec<u8>> {
        self.check_bounds(address, size)?;
        let addr = address as usize;
        
        Ok(self.data[addr..addr + size].to_vec())
    }
    
    fn write_block(&mut self, address: u32, data: &[u8]) -> Result<()> {
        self.check_bounds(address, data.len())?;
        let addr = address as usize;
        
        self.data[addr..addr + data.len()].copy_from_slice(data);
        self.stats.record_write(data.len());
        
        Ok(())
    }
    
    fn fill(&mut self, address: u32, size: usize, value: u8) -> Result<()> {
        self.check_bounds(address, size)?;
        let addr = address as usize;
        
        self.data[addr..addr + size].fill(value);
        self.stats.record_write(size);
        
        Ok(())
    }
}

/// Statistiques d'accès mémoire pour l'optimisation
#[derive(Debug, Clone)]
pub struct AccessStats {
    /// Nombre total de lectures
    pub reads: u64,
    
    /// Nombre total d'écritures
    pub writes: u64,
    
    /// Octets lus au total
    pub bytes_read: u64,
    
    /// Octets écrits au total
    pub bytes_written: u64,
    
    /// Zones mémoire les plus fréquemment accédées (adresse -> compteur)
    pub hot_spots: std::collections::HashMap<u32, u32>,
}

impl AccessStats {
    fn new() -> Self {
        Self {
            reads: 0,
            writes: 0,
            bytes_read: 0,
            bytes_written: 0,
            hot_spots: std::collections::HashMap::new(),
        }
    }
    
    fn record_read(&mut self, bytes: usize) {
        self.reads += 1;
        self.bytes_read += bytes as u64;
    }
    
    fn record_write(&mut self, bytes: usize) {
        self.writes += 1;
        self.bytes_written += bytes as u64;
    }
    
    fn record_access(&mut self, address: u32) {
        // Regrouper par blocs de 64 octets pour éviter trop d'entrées
        let block_addr = address & !0x3F;
        *self.hot_spots.entry(block_addr).or_insert(0) += 1;
    }
    
    fn reset(&mut self) {
        self.reads = 0;
        self.writes = 0;
        self.bytes_read = 0;
        self.bytes_written = 0;
        self.hot_spots.clear();
    }
    
    /// Calcule le ratio lecture/écriture
    pub fn read_write_ratio(&self) -> f64 {
        if self.writes == 0 {
            if self.reads == 0 { 0.0 } else { f64::INFINITY }
        } else {
            self.reads as f64 / self.writes as f64
        }
    }
    
    /// Obtient les zones les plus accédées
    pub fn get_hottest_regions(&self, count: usize) -> Vec<(u32, u32)> {
        let mut regions: Vec<_> = self.hot_spots.iter().collect();
        regions.sort_by(|a, b| b.1.cmp(a.1));
        regions.into_iter()
            .take(count)
            .map(|(&addr, &count)| (addr, count))
            .collect()
    }
}