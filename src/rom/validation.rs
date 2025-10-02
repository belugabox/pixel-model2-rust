//! Système de validation et vérification des ROMs

use super::database::{GameInfo, RomInfo};
use anyhow::Result;
use crc32fast::Hasher;
use sha2::{Digest, Sha256};

/// Résultat de validation d'une ROM
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// ROM valide selon les critères
    pub is_valid: bool,

    /// Checksum CRC32 calculé
    pub calculated_crc32: u32,

    /// Hash MD5 calculé
    pub calculated_md5: String,

    /// Hash SHA256 calculé
    pub calculated_sha256: String,

    /// Taille du fichier
    pub file_size: usize,

    /// Erreurs détectées
    pub errors: Vec<ValidationError>,

    /// Avertissements
    pub warnings: Vec<String>,
}

/// Types d'erreurs de validation
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// CRC32 incorrect
    InvalidCrc32 { expected: u32, found: u32 },

    /// MD5 incorrect
    InvalidMd5 { expected: String, found: String },

    /// Taille incorrecte
    InvalidSize { expected: usize, found: usize },

    /// Fichier corrompu ou illisible
    CorruptedFile(String),

    /// Type de ROM incorrect
    InvalidRomType(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidCrc32 { expected, found } => {
                write!(
                    f,
                    "CRC32 incorrect: attendu {:#010x}, trouvé {:#010x}",
                    expected, found
                )
            }
            ValidationError::InvalidMd5 { expected, found } => {
                write!(f, "MD5 incorrect: attendu {}, trouvé {}", expected, found)
            }
            ValidationError::InvalidSize { expected, found } => {
                write!(
                    f,
                    "Taille incorrect: attendu {} octets, trouvé {} octets",
                    expected, found
                )
            }
            ValidationError::CorruptedFile(msg) => {
                write!(f, "Fichier corrompu: {}", msg)
            }
            ValidationError::InvalidRomType(msg) => {
                write!(f, "Type de ROM invalide: {}", msg)
            }
        }
    }
}

/// Validateur de ROMs
pub struct RomValidator;

impl RomValidator {
    /// Valide une ROM contre les informations attendues
    pub fn validate_rom(data: &[u8], expected: &RomInfo) -> ValidationResult {
        let mut result = ValidationResult {
            is_valid: true,
            calculated_crc32: 0,
            calculated_md5: String::new(),
            calculated_sha256: String::new(),
            file_size: data.len(),
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        // Calculer les checksums
        result.calculated_crc32 = Self::calculate_crc32(data);
        result.calculated_md5 = Self::calculate_md5(data);
        result.calculated_sha256 = Self::calculate_sha256(data);

        // Vérifier la taille
        if data.len() != expected.size {
            result.errors.push(ValidationError::InvalidSize {
                expected: expected.size,
                found: data.len(),
            });
            result.is_valid = false;
        }

        // Vérifier le CRC32 (seulement si ce n'est pas un placeholder)
        if expected.crc32 != 0x00000000 && result.calculated_crc32 != expected.crc32 {
            result.errors.push(ValidationError::InvalidCrc32 {
                expected: expected.crc32,
                found: result.calculated_crc32,
            });
            result.is_valid = false;
        } else if expected.crc32 == 0x00000000 {
            // Checksum placeholder - ajouter un avertissement
            result
                .warnings
                .push("Checksum CRC32 non défini dans la base de données".to_string());
        }

        // Vérifier le MD5 (seulement si ce n'est pas vide)
        if !expected.md5.is_empty() && result.calculated_md5 != expected.md5 {
            result.errors.push(ValidationError::InvalidMd5 {
                expected: expected.md5.clone(),
                found: result.calculated_md5.clone(),
            });
            result.is_valid = false;
        } else if expected.md5.is_empty() {
            // MD5 placeholder - ajouter un avertissement
            result
                .warnings
                .push("Hash MD5 non défini dans la base de données".to_string());
        }

        // Pour le développement, considérer valide si seulement la taille est correcte
        // et les checksums sont des placeholders
        if expected.crc32 == 0x00000000 && expected.md5.is_empty() && data.len() == expected.size {
            result.is_valid = true;
            result.errors.clear(); // Effacer les erreurs de checksum
        }

        // Vérifications spécifiques au type de ROM
        if let Some(warning) = Self::validate_rom_type_content(data, &expected.rom_type) {
            result.warnings.push(warning);
        }

        result
    }

    /// Valide un ensemble complet de ROMs pour un jeu
    pub fn validate_rom_set(
        rom_files: &[(String, Vec<u8>)],
        game_info: &GameInfo,
    ) -> Result<Vec<(String, ValidationResult)>> {
        let mut results = Vec::new();

        // Vérifier chaque ROM requise
        for required_rom in &game_info.required_roms {
            if let Some((_, data)) = rom_files
                .iter()
                .find(|(name, _)| name == &required_rom.filename)
            {
                let validation = Self::validate_rom(data, required_rom);
                results.push((required_rom.filename.clone(), validation));
            } else {
                // ROM manquante
                let missing_result = ValidationResult {
                    is_valid: false,
                    calculated_crc32: 0,
                    calculated_md5: String::new(),
                    calculated_sha256: String::new(),
                    file_size: 0,
                    errors: vec![ValidationError::CorruptedFile("ROM manquante".to_string())],
                    warnings: Vec::new(),
                };
                results.push((required_rom.filename.clone(), missing_result));
            }
        }

        // Vérifier les ROMs optionnelles si présentes
        for optional_rom in &game_info.optional_roms {
            if let Some((_, data)) = rom_files
                .iter()
                .find(|(name, _)| name == &optional_rom.filename)
            {
                let validation = Self::validate_rom(data, optional_rom);
                results.push((optional_rom.filename.clone(), validation));
            }
        }

        Ok(results)
    }

    /// Calcule le CRC32 d'un buffer
    pub fn calculate_crc32(data: &[u8]) -> u32 {
        let mut hasher = Hasher::new();
        hasher.update(data);
        hasher.finalize()
    }

    /// Calcule le hash MD5 d'un buffer
    pub fn calculate_md5(data: &[u8]) -> String {
        let mut hasher = md5::Context::new();
        hasher.consume(data);
        format!("{:x}", hasher.finalize())
    }

    /// Calcule le hash SHA256 d'un buffer
    pub fn calculate_sha256(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Détecte automatiquement le type de ROM basé sur le contenu
    pub fn detect_rom_type(data: &[u8], filename: &str) -> super::database::RomType {
        use super::database::RomType;

        let filename_lower = filename.to_lowercase();

        // Détection basée sur le nom de fichier
        if filename_lower.contains("prg") || filename_lower.contains("program") {
            return RomType::Program;
        }
        if filename_lower.contains("gfx")
            || filename_lower.contains("chr")
            || filename_lower.contains("spr")
        {
            return RomType::Graphics;
        }
        if filename_lower.contains("snd")
            || filename_lower.contains("pcm")
            || filename_lower.contains("wav")
        {
            return RomType::Sound;
        }

        // Détection basée sur le contenu
        if data.len() >= 4 {
            let header = &data[0..4];

            // Headers de fichiers connus
            match header {
                // En-tête ELF
                [0x7F, b'E', b'L', b'F'] => return RomType::Program,
                // En-tête NEC V60
                [0x4E, 0x45, 0x43, 0x56] => return RomType::Program,
                // Header de texture possible
                [0x54, 0x45, 0x58, 0x54] => return RomType::Texture,
                // Header audio WAV
                [b'R', b'I', b'F', b'F'] => return RomType::Sound,
                _ => {}
            }
        }

        // Heuristiques basées sur la taille
        match data.len() {
            // ROMs programme typiques (512KB - 2MB)
            0x80000..=0x200000 => RomType::Program,
            // Grandes ROMs graphiques (>2MB)
            size if size > 0x200000 => RomType::Graphics,
            // Petites ROMs diverses
            _ => RomType::Data,
        }
    }

    /// Valide le contenu spécifique selon le type de ROM
    fn validate_rom_type_content(
        data: &[u8],
        rom_type: &super::database::RomType,
    ) -> Option<String> {
        use super::database::RomType;

        match rom_type {
            RomType::Program => {
                // Vérifier que ce n'est pas une ROM vide
                if data.iter().all(|&b| b == 0x00 || b == 0xFF) {
                    return Some("ROM programme semble vide ou corrompue".to_string());
                }

                // Vérifier la présence d'instructions valides (heuristique)
                if data.len() >= 16 {
                    let mut valid_instructions = 0;
                    for chunk in data.chunks(4) {
                        if chunk.len() == 4 {
                            let opcode =
                                u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                            // Heuristique simple pour détecter des opcodes valides
                            if opcode != 0x00000000 && opcode != 0xFFFFFFFF {
                                valid_instructions += 1;
                            }
                        }
                    }

                    if valid_instructions < data.len() / 16 {
                        return Some("Contenu de la ROM programme suspect".to_string());
                    }
                }
            }

            RomType::Graphics => {
                // Vérifier que la ROM n'est pas complètement vide
                if data.iter().all(|&b| b == 0x00) {
                    return Some("ROM graphique semble vide".to_string());
                }

                // Vérifier une certaine entropie dans les données graphiques
                let entropy = Self::calculate_entropy(data);
                if entropy < 2.0 {
                    return Some("Données graphiques semblent peu variées".to_string());
                }
            }

            RomType::Sound => {
                // Vérifications similaires pour l'audio
                if data.iter().all(|&b| b == 0x00) {
                    return Some("ROM audio semble vide".to_string());
                }
            }

            _ => {}
        }

        None
    }

    /// Calcule l'entropie d'un buffer (mesure de la variété des données)
    fn calculate_entropy(data: &[u8]) -> f64 {
        let mut counts = [0u32; 256];

        // Compter les occurrences de chaque byte
        for &byte in data {
            counts[byte as usize] += 1;
        }

        // Calculer l'entropie
        let len = data.len() as f64;
        let mut entropy = 0.0;

        for count in counts.iter() {
            if *count > 0 {
                let p = *count as f64 / len;
                entropy -= p * p.log2();
            }
        }

        entropy
    }

    /// Génère un rapport de validation détaillé
    pub fn generate_validation_report(results: &[(String, ValidationResult)]) -> String {
        let mut report = String::new();
        report.push_str("=== RAPPORT DE VALIDATION ROM ===\n\n");

        let mut total_roms = 0;
        let mut valid_roms = 0;

        for (filename, result) in results {
            total_roms += 1;
            if result.is_valid {
                valid_roms += 1;
            }

            report.push_str(&format!("ROM: {}\n", filename));
            report.push_str(&format!(
                "  Statut: {}\n",
                if result.is_valid {
                    "VALIDE"
                } else {
                    "INVALIDE"
                }
            ));
            report.push_str(&format!("  Taille: {} octets\n", result.file_size));
            report.push_str(&format!("  CRC32: {:#010x}\n", result.calculated_crc32));
            report.push_str(&format!("  MD5: {}\n", result.calculated_md5));

            if !result.errors.is_empty() {
                report.push_str("  Erreurs:\n");
                for error in &result.errors {
                    report.push_str(&format!("    - {}\n", error));
                }
            }

            if !result.warnings.is_empty() {
                report.push_str("  Avertissements:\n");
                for warning in &result.warnings {
                    report.push_str(&format!("    - {}\n", warning));
                }
            }

            report.push('\n');
        }

        report.push_str(&format!(
            "RÉSUMÉ: {}/{} ROMs valides\n",
            valid_roms, total_roms
        ));

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rom::database::{RomInfo, RomType};

    #[test]
    fn test_crc32_calculation() {
        let data = b"Hello, World!";
        let crc = RomValidator::calculate_crc32(data);
        assert_ne!(crc, 0);
    }

    #[test]
    fn test_md5_calculation() {
        let data = b"Hello, World!";
        let md5 = RomValidator::calculate_md5(data);
        assert_eq!(md5.len(), 32); // MD5 is 32 hex characters
    }

    #[test]
    fn test_rom_validation_success() {
        let data = b"Test ROM data";
        let crc32 = RomValidator::calculate_crc32(data);
        let md5 = RomValidator::calculate_md5(data);

        let rom_info = RomInfo {
            filename: "test.bin".to_string(),
            rom_type: RomType::Program,
            size: data.len(),
            crc32,
            md5,
            load_address: 0x1000,
            bank: 0,
            required: true,
        };

        let result = RomValidator::validate_rom(data, &rom_info);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_rom_validation_failure() {
        let data = b"Test ROM data";

        let rom_info = RomInfo {
            filename: "test.bin".to_string(),
            rom_type: RomType::Program,
            size: data.len(),
            crc32: 0x12345678, // Wrong CRC
            md5: "wrong_md5".to_string(),
            load_address: 0x1000,
            bank: 0,
            required: true,
        };

        let result = RomValidator::validate_rom(data, &rom_info);
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_rom_type_detection() {
        let program_data = vec![0x4E, 0x45, 0x43, 0x56]; // "NECV" header
        assert_eq!(
            RomValidator::detect_rom_type(&program_data, "program.bin"),
            RomType::Program
        );

        let graphics_data = vec![0; 0x300000]; // Large ROM -> Graphics
        assert_eq!(
            RomValidator::detect_rom_type(&graphics_data, "graphics.bin"),
            RomType::Graphics
        );
    }

    #[test]
    fn test_entropy_calculation() {
        // Data with high entropy (random-like)
        let high_entropy_data: Vec<u8> = (0..256).map(|i| i as u8).collect();
        let entropy = RomValidator::calculate_entropy(&high_entropy_data);
        assert!(entropy > 7.0); // Should be close to 8.0 for perfect distribution

        // Data with low entropy (repetitive)
        let low_entropy_data = vec![0x55; 1000];
        let entropy = RomValidator::calculate_entropy(&low_entropy_data);
        assert!(entropy < 1.0);
    }
}
