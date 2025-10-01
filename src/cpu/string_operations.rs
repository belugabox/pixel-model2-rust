//! Instructions de manipulation de chaînes NEC V60

use super::registers::ProcessorStatusWord;
use crate::memory::MemoryInterface;
use anyhow::Result;

/// Résultat d'une opération sur chaîne
#[derive(Debug)]
pub struct StringResult {
    pub bytes_processed: u32,
    pub equal: bool,
    pub found: bool,
    pub source_exhausted: bool,
    pub destination_exhausted: bool,
}

impl StringResult {
    /// Met à jour le mot d'état du processeur
    pub fn update_psw(&self, psw: &mut ProcessorStatusWord) {
        psw.set(ProcessorStatusWord::ZERO, self.equal || self.found);
        psw.set(ProcessorStatusWord::CARRY, self.source_exhausted);
        psw.set(ProcessorStatusWord::OVERFLOW, self.destination_exhausted);
    }
}

/// Unité de manipulation de chaînes
pub struct StringUnit;

impl StringUnit {
    /// Copie de chaîne (STRING_MOVE)
    pub fn string_move<M>(
        memory: &mut M,
        source: u32,
        destination: u32,
        max_length: u32,
        element_size: u8,
    ) -> Result<StringResult>
    where
        M: MemoryInterface,
    {
        let mut bytes_processed = 0;
        let mut current_src = source;
        let mut current_dst = destination;

        for _ in 0..max_length {
            let value = match element_size {
                1 => memory.read_u8(current_src)? as u32,
                2 => memory.read_u16(current_src)? as u32,
                4 => memory.read_u32(current_src)?,
                _ => return Err(anyhow::anyhow!("Taille d'élément non supportée: {}", element_size)),
            };

            match element_size {
                1 => memory.write_u8(current_dst, value as u8)?,
                2 => memory.write_u16(current_dst, value as u16)?,
                4 => memory.write_u32(current_dst, value)?,
                _ => unreachable!(),
            }

            bytes_processed += element_size as u32;
            current_src += element_size as u32;
            current_dst += element_size as u32;

            // Arrêt si on trouve un terminateur nul
            if value == 0 {
                break;
            }
        }

        Ok(StringResult {
            bytes_processed,
            equal: true,
            found: false,
            source_exhausted: bytes_processed == max_length * element_size as u32,
            destination_exhausted: false,
        })
    }

    /// Comparaison de chaînes (STRING_COMPARE)
    pub fn string_compare<M>(
        memory: &M,
        source1: u32,
        source2: u32,
        max_length: u32,
        element_size: u8,
    ) -> Result<StringResult>
    where
        M: MemoryInterface,
    {
        let mut bytes_processed = 0;
        let mut current_src1 = source1;
        let mut current_src2 = source2;
        let mut equal = true;

        for _ in 0..max_length {
            let value1 = match element_size {
                1 => memory.read_u8(current_src1)? as u32,
                2 => memory.read_u16(current_src1)? as u32,
                4 => memory.read_u32(current_src1)?,
                _ => return Err(anyhow::anyhow!("Taille d'élément non supportée: {}", element_size)),
            };

            let value2 = match element_size {
                1 => memory.read_u8(current_src2)? as u32,
                2 => memory.read_u16(current_src2)? as u32,
                4 => memory.read_u32(current_src2)?,
                _ => unreachable!(),
            };

            bytes_processed += element_size as u32;
            current_src1 += element_size as u32;
            current_src2 += element_size as u32;

            if value1 != value2 {
                equal = false;
                break;
            }

            // Arrêt si les deux chaînes se terminent
            if value1 == 0 && value2 == 0 {
                break;
            }
        }

        Ok(StringResult {
            bytes_processed,
            equal,
            found: false,
            source_exhausted: bytes_processed == max_length * element_size as u32,
            destination_exhausted: false,
        })
    }

    /// Recherche dans une chaîne (STRING_SCAN)
    pub fn string_scan<M>(
        memory: &M,
        source: u32,
        target_value: u32,
        max_length: u32,
        element_size: u8,
    ) -> Result<StringResult>
    where
        M: MemoryInterface,
    {
        let mut bytes_processed = 0;
        let mut current_src = source;
        let mut found = false;

        for _ in 0..max_length {
            let value = match element_size {
                1 => memory.read_u8(current_src)? as u32,
                2 => memory.read_u16(current_src)? as u32,
                4 => memory.read_u32(current_src)?,
                _ => return Err(anyhow::anyhow!("Taille d'élément non supportée: {}", element_size)),
            };

            bytes_processed += element_size as u32;
            current_src += element_size as u32;

            if value == target_value {
                found = true;
                break;
            }

            // Arrêt si on trouve un terminateur nul
            if value == 0 {
                break;
            }
        }

        Ok(StringResult {
            bytes_processed,
            equal: false,
            found,
            source_exhausted: bytes_processed == max_length * element_size as u32,
            destination_exhausted: false,
        })
    }

    /// Remplissage de mémoire (STRING_FILL)
    pub fn string_fill<M>(
        memory: &mut M,
        destination: u32,
        fill_value: u32,
        count: u32,
        element_size: u8,
    ) -> Result<StringResult>
    where
        M: MemoryInterface,
    {
        let mut bytes_processed = 0;
        let mut current_dst = destination;

        for _ in 0..count {
            match element_size {
                1 => memory.write_u8(current_dst, fill_value as u8)?,
                2 => memory.write_u16(current_dst, fill_value as u16)?,
                4 => memory.write_u32(current_dst, fill_value)?,
                _ => return Err(anyhow::anyhow!("Taille d'élément non supportée: {}", element_size)),
            }

            bytes_processed += element_size as u32;
            current_dst += element_size as u32;
        }

        Ok(StringResult {
            bytes_processed,
            equal: true,
            found: false,
            source_exhausted: false,
            destination_exhausted: false,
        })
    }

    /// Longueur de chaîne (STRING_LENGTH)
    pub fn string_length<M>(
        memory: &M,
        source: u32,
        max_length: u32,
        element_size: u8,
    ) -> Result<u32>
    where
        M: MemoryInterface,
    {
        let mut length = 0;
        let mut current_src = source;

        for _ in 0..max_length {
            let value = match element_size {
                1 => memory.read_u8(current_src)? as u32,
                2 => memory.read_u16(current_src)? as u32,
                4 => memory.read_u32(current_src)?,
                _ => return Err(anyhow::anyhow!("Taille d'élément non supportée: {}", element_size)),
            };

            if value == 0 {
                break;
            }

            length += 1;
            current_src += element_size as u32;
        }

        Ok(length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::ram::Ram;

    #[test]
    fn test_string_move() {
        let mut memory = Ram::new(0x10000);
        
        // Source: "Hello"
        memory.write_u8(0x1000, b'H').unwrap();
        memory.write_u8(0x1001, b'e').unwrap();
        memory.write_u8(0x1002, b'l').unwrap();
        memory.write_u8(0x1003, b'l').unwrap();
        memory.write_u8(0x1004, b'o').unwrap();
        memory.write_u8(0x1005, 0).unwrap(); // Terminateur

        let result = StringUnit::string_move(&mut memory, 0x1000, 0x2000, 10, 1).unwrap();
        
        assert_eq!(result.bytes_processed, 6); // 5 caractères + terminateur
        assert_eq!(memory.read_u8(0x2000).unwrap(), b'H');
        assert_eq!(memory.read_u8(0x2004).unwrap(), b'o');
        assert_eq!(memory.read_u8(0x2005).unwrap(), 0);
    }

    #[test]
    fn test_string_compare() {
        let mut memory = Ram::new(0x10000);
        
        // Deux chaînes identiques
        let text = b"Test";
        for (i, &byte) in text.iter().enumerate() {
            memory.write_u8(0x1000 + i as u32, byte).unwrap();
            memory.write_u8(0x2000 + i as u32, byte).unwrap();
        }
        memory.write_u8(0x1004, 0).unwrap();
        memory.write_u8(0x2004, 0).unwrap();

        let result = StringUnit::string_compare(&memory, 0x1000, 0x2000, 10, 1).unwrap();
        assert!(result.equal);
        assert_eq!(result.bytes_processed, 5); // 4 caractères + terminateur
    }

    #[test]
    fn test_string_scan() {
        let mut memory = Ram::new(0x10000);
        
        // Chaîne: "Hello"
        let text = b"Hello";
        for (i, &byte) in text.iter().enumerate() {
            memory.write_u8(0x1000 + i as u32, byte).unwrap();
        }
        memory.write_u8(0x1005, 0).unwrap();

        // Chercher 'l'
        let result = StringUnit::string_scan(&memory, 0x1000, b'l' as u32, 10, 1).unwrap();
        assert!(result.found);
        assert_eq!(result.bytes_processed, 3); // H, e, l (trouvé au 3ème)
    }
}