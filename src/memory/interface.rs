//! Interface mémoire commune

use anyhow::Result;

/// Trait définissant l'interface commune pour tous les types de mémoire
pub trait MemoryInterface {
    /// Lit un octet à l'adresse spécifiée
    fn read_u8(&self, address: u32) -> Result<u8>;

    /// Lit un mot de 16 bits à l'adresse spécifiée
    fn read_u16(&self, address: u32) -> Result<u16>;

    /// Lit un mot de 32 bits à l'adresse spécifiée
    fn read_u32(&self, address: u32) -> Result<u32>;

    /// Écrit un octet à l'adresse spécifiée
    fn write_u8(&mut self, address: u32, value: u8) -> Result<()>;

    /// Écrit un mot de 16 bits à l'adresse spécifiée
    fn write_u16(&mut self, address: u32, value: u16) -> Result<()>;

    /// Écrit un mot de 32 bits à l'adresse spécifiée
    fn write_u32(&mut self, address: u32, value: u32) -> Result<()>;

    /// Lit un bloc de données
    fn read_block(&self, address: u32, size: usize) -> Result<Vec<u8>> {
        let mut data = Vec::with_capacity(size);
        for i in 0..size {
            data.push(self.read_u8(address + i as u32)?);
        }
        Ok(data)
    }

    /// Écrit un bloc de données
    fn write_block(&mut self, address: u32, data: &[u8]) -> Result<()> {
        for (i, &byte) in data.iter().enumerate() {
            self.write_u8(address + i as u32, byte)?;
        }
        Ok(())
    }

    /// Copie des données d'une adresse à une autre
    fn copy(&mut self, src: u32, dst: u32, size: usize) -> Result<()> {
        // Optimisation : copie par blocs pour éviter les lectures/écritures individuelles
        let block_size = 1024;
        let mut remaining = size;
        let mut src_addr = src;
        let mut dst_addr = dst;

        while remaining > 0 {
            let chunk_size = remaining.min(block_size);
            let data = self.read_block(src_addr, chunk_size)?;
            self.write_block(dst_addr, &data)?;

            remaining -= chunk_size;
            src_addr += chunk_size as u32;
            dst_addr += chunk_size as u32;
        }

        Ok(())
    }

    /// Remplit une région mémoire avec une valeur
    fn fill(&mut self, address: u32, size: usize, value: u8) -> Result<()> {
        for i in 0..size {
            self.write_u8(address + i as u32, value)?;
        }
        Ok(())
    }
}
