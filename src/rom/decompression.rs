//! Système de décompression pour les ROMs

use anyhow::{Result, anyhow};
use std::path::Path;
use std::io::{Read, BufReader};
use zip::ZipArchive;
use flate2::read::GzDecoder;

/// Types de compression supportés
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompressionType {
    /// Fichier non compressé
    None,
    
    /// Archive ZIP
    Zip,
    
    /// Compression GZIP
    Gzip,
    
    /// Archive 7-Zip (pour support futur)
    SevenZip,
    
    /// Archive RAR (pour support futur)
    Rar,
}

/// Résultat de décompression
#[derive(Debug)]
pub struct DecompressionResult {
    /// Fichiers extraits avec leur nom et contenu
    pub files: Vec<(String, Vec<u8>)>,
    
    /// Type de compression détecté
    pub compression_type: CompressionType,
    
    /// Taille totale décompressée
    pub total_size: usize,
}

/// Décompresseur de fichiers ROM
pub struct RomDecompressor;

impl RomDecompressor {
    /// Détecte le type de compression d'un fichier
    pub fn detect_compression_type(path: &Path) -> CompressionType {
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            match extension.to_lowercase().as_str() {
                "zip" => CompressionType::Zip,
                "gz" | "gzip" => CompressionType::Gzip,
                "7z" => CompressionType::SevenZip,
                "rar" => CompressionType::Rar,
                _ => CompressionType::None,
            }
        } else {
            CompressionType::None
        }
    }
    
    /// Décompresse un fichier selon son type
    pub fn decompress_file(path: &Path) -> Result<DecompressionResult> {
        let compression_type = Self::detect_compression_type(path);
        
        match compression_type {
            CompressionType::None => Self::load_raw_file(path),
            CompressionType::Zip => Self::decompress_zip(path),
            CompressionType::Gzip => Self::decompress_gzip(path),
            CompressionType::SevenZip => Err(anyhow!("Support 7-Zip non encore implémenté")),
            CompressionType::Rar => Err(anyhow!("Support RAR non encore implémenté")),
        }
    }
    
    /// Charge un fichier non compressé
    fn load_raw_file(path: &Path) -> Result<DecompressionResult> {
        let data = std::fs::read(path)?;
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let total_size = data.len();
        
        Ok(DecompressionResult {
            files: vec![(filename, data)],
            compression_type: CompressionType::None,
            total_size,
        })
    }
    
    /// Décompresse une archive ZIP
    fn decompress_zip(path: &Path) -> Result<DecompressionResult> {
        let file = std::fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut archive = ZipArchive::new(reader)?;
        
        let mut files = Vec::new();
        let mut total_size = 0;
        
        for i in 0..archive.len() {
            let mut zip_file = archive.by_index(i)?;
            
            // Ignorer les dossiers
            if zip_file.is_dir() {
                continue;
            }
            
            let mut contents = Vec::new();
            zip_file.read_to_end(&mut contents)?;
            
            let filename = zip_file.name().to_string();
            total_size += contents.len();
            
            files.push((filename, contents));
        }
        
        Ok(DecompressionResult {
            files,
            compression_type: CompressionType::Zip,
            total_size,
        })
    }
    
    /// Décompresse un fichier GZIP
    fn decompress_gzip(path: &Path) -> Result<DecompressionResult> {
        let file = std::fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut decoder = GzDecoder::new(reader);
        
        let mut contents = Vec::new();
        decoder.read_to_end(&mut contents)?;
        
        // Pour GZIP, on utilise le nom de fichier sans l'extension .gz
        let filename = path.file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let total_size = contents.len();
        
        Ok(DecompressionResult {
            files: vec![(filename, contents)],
            compression_type: CompressionType::Gzip,
            total_size,
        })
    }
    
    /// Filtre les fichiers ROM (ignore les fichiers système, readme, etc.)
    pub fn filter_rom_files(files: Vec<(String, Vec<u8>)>) -> Vec<(String, Vec<u8>)> {
        files.into_iter()
            .filter(|(filename, _)| Self::is_rom_file(filename))
            .collect()
    }
    
    /// Vérifie si un fichier est probablement une ROM
    fn is_rom_file(filename: &str) -> bool {
        let filename_lower = filename.to_lowercase();
        
        // Extensions ROM typiques
        let rom_extensions = [
            ".bin", ".rom", ".ic1", ".ic2", ".ic3", ".ic4", ".ic5", ".ic6", ".ic7", ".ic8",
            ".ic9", ".ic10", ".ic11", ".ic12", ".ic13", ".ic14", ".ic15", ".ic16",
            ".ic17", ".ic18", ".ic19", ".ic20", ".ic21", ".ic22", ".ic23", ".ic24",
            ".ic25", ".ic26", ".ic27", ".ic28", ".ic29", ".ic30", ".ic31", ".ic32",
            ".prg", ".gfx", ".snd", ".dat", ".chr", ".spr",
        ];
        
        for ext in &rom_extensions {
            if filename_lower.ends_with(ext) {
                return true;
            }
        }
        
        // Ignorer les fichiers système et documentation
        let ignore_patterns = [
            "readme", "license", "changelog", "history", "info", "nfo",
            ".txt", ".md", ".doc", ".pdf", ".url", ".lnk",
            "__macosx", ".ds_store", "thumbs.db",
        ];
        
        for pattern in &ignore_patterns {
            if filename_lower.contains(pattern) {
                return false;
            }
        }
        
        // Par défaut, considérer comme ROM si la taille est raisonnable
        true
    }
    
    /// Trie les fichiers ROM dans l'ordre logique (ic1, ic2, ic3, ...)
    pub fn sort_rom_files(mut files: Vec<(String, Vec<u8>)>) -> Vec<(String, Vec<u8>)> {
        files.sort_by(|a, b| {
            Self::extract_rom_number(&a.0).cmp(&Self::extract_rom_number(&b.0))
        });
        files
    }
    
    /// Extrait le numéro de ROM d'un nom de fichier (ex: "game.ic15" -> 15)
    fn extract_rom_number(filename: &str) -> u32 {
        let filename_lower = filename.to_lowercase();
        
        // Chercher un pattern comme "ic15", "rom3", "prg1", etc.
        if let Some(pos) = filename_lower.find("ic") {
            if let Ok(num) = filename_lower[pos+2..].split('.').next()
                .unwrap_or("0").parse::<u32>() {
                return num;
            }
        }
        
        // Autres patterns
        for prefix in &["rom", "prg", "gfx", "snd"] {
            if let Some(pos) = filename_lower.find(prefix) {
                let start = pos + prefix.len();
                if let Ok(num) = filename_lower[start..].chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>().parse::<u32>() {
                    return num;
                }
            }
        }
        
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_compression_detection() {
        assert_eq!(RomDecompressor::detect_compression_type(Path::new("test.zip")), CompressionType::Zip);
        assert_eq!(RomDecompressor::detect_compression_type(Path::new("test.bin")), CompressionType::None);
        assert_eq!(RomDecompressor::detect_compression_type(Path::new("test.gz")), CompressionType::Gzip);
    }

    #[test]
    fn test_rom_file_filtering() {
        assert!(RomDecompressor::is_rom_file("game.ic1"));
        assert!(RomDecompressor::is_rom_file("sound.bin"));
        assert!(!RomDecompressor::is_rom_file("readme.txt"));
        assert!(!RomDecompressor::is_rom_file("license.md"));
    }

    #[test]
    fn test_rom_number_extraction() {
        assert_eq!(RomDecompressor::extract_rom_number("game.ic15"), 15);
        assert_eq!(RomDecompressor::extract_rom_number("sound.rom3"), 3);
        assert_eq!(RomDecompressor::extract_rom_number("program.prg1"), 1);
        assert_eq!(RomDecompressor::extract_rom_number("unknown.dat"), 0);
    }

    #[test]
    fn test_rom_sorting() {
        let files = vec![
            ("game.ic15".to_string(), vec![1, 2, 3]),
            ("game.ic2".to_string(), vec![4, 5, 6]),
            ("game.ic1".to_string(), vec![7, 8, 9]),
        ];
        
        let sorted = RomDecompressor::sort_rom_files(files);
        assert_eq!(sorted[0].0, "game.ic1");
        assert_eq!(sorted[1].0, "game.ic2");
        assert_eq!(sorted[2].0, "game.ic15");
    }

    #[test]
    fn test_raw_file_loading() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let test_data = b"Hello, ROM world!";
        temp_file.write_all(test_data)?;
        
        let result = RomDecompressor::decompress_file(temp_file.path())?;
        
        assert_eq!(result.compression_type, CompressionType::None);
        assert_eq!(result.files.len(), 1);
        assert_eq!(result.files[0].1, test_data);
        assert_eq!(result.total_size, test_data.len());
        
        Ok(())
    }
}