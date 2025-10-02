//! Base de données des jeux SEGA Model 2

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Informations sur un jeu Model 2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfo {
    /// Nom du jeu
    pub name: String,

    /// Nom alternatif/court
    pub short_name: String,

    /// Développeur
    pub developer: String,

    /// Année de sortie
    pub year: u16,

    /// Région (JP, US, EU, etc.)
    pub region: String,

    /// Version du jeu
    pub version: String,

    /// Liste des ROMs requises avec leurs checksums
    pub required_roms: Vec<RomInfo>,

    /// Liste des ROMs optionnelles
    pub optional_roms: Vec<RomInfo>,

    /// Configuration système spécifique
    pub system_config: SystemConfig,

    /// Description du jeu
    pub description: String,
}

/// Information sur une ROM individuelle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RomInfo {
    /// Nom du fichier ROM
    pub filename: String,

    /// Type de ROM (program, graphics, sound, etc.)
    pub rom_type: RomType,

    /// Taille attendue en octets
    pub size: usize,

    /// Checksum CRC32
    pub crc32: u32,

    /// Hash MD5
    pub md5: String,

    /// Adresse de chargement en mémoire
    pub load_address: u32,

    /// Banque mémoire
    pub bank: u8,

    /// Obligatoire ou optionnel
    pub required: bool,
}

/// Types de ROM
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RomType {
    /// Programme principal (CPU)
    Program,

    /// Données graphiques (GPU)
    Graphics,

    /// Données audio (SCSP)
    Sound,

    /// Données de géométrie 3D
    Geometry,

    /// Textures
    Texture,

    /// Samples audio
    Samples,

    /// Données de configuration
    Config,

    /// Microcode pour processeurs spécialisés
    Microcode,

    /// Données diverses
    Data,
}

/// Configuration système spécifique au jeu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// Fréquence CPU en Hz
    pub cpu_frequency: u32,

    /// Résolution d'affichage
    pub display_resolution: (u32, u32),

    /// Fréquence de rafraîchissement
    pub refresh_rate: f32,

    /// Configuration audio
    pub audio_config: AudioConfig,

    /// Configuration graphique spéciale
    pub graphics_config: GraphicsConfig,

    /// Contrôles supportés
    pub supported_controls: Vec<String>,
}

/// Configuration audio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Fréquence d'échantillonnage
    pub sample_rate: u32,

    /// Nombre de canaux
    pub channels: u8,

    /// Utilise le DSP SCSP
    pub use_scsp: bool,
}

/// Configuration graphique
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsConfig {
    /// Support texture mapping
    pub texture_mapping: bool,

    /// Support transparency
    pub transparency: bool,

    /// Support anti-aliasing
    pub antialiasing: bool,

    /// Nombre de plans de texture
    pub texture_planes: u8,
}

/// Base de données des jeux Model 2
pub struct GameDatabase {
    games: HashMap<String, GameInfo>,
}

impl GameDatabase {
    /// Crée une nouvelle base de données
    pub fn new() -> Self {
        let mut db = Self {
            games: HashMap::new(),
        };

        // Ajouter les jeux connus
        db.add_known_games();
        db
    }

    /// Trouve un jeu par nom
    pub fn find_game(&self, name: &str) -> Option<&GameInfo> {
        // Recherche directe
        if let Some(game) = self.games.get(name) {
            return Some(game);
        }

        // Recherche par nom court
        for game in self.games.values() {
            if game.short_name == name {
                return Some(game);
            }
        }

        // Recherche partielle (case insensitive)
        let name_lower = name.to_lowercase();
        for game in self.games.values() {
            if game.name.to_lowercase().contains(&name_lower)
                || game.short_name.to_lowercase().contains(&name_lower)
            {
                return Some(game);
            }
        }

        None
    }

    /// Liste tous les jeux disponibles
    pub fn list_games(&self) -> Vec<&GameInfo> {
        self.games.values().collect()
    }

    /// Ajoute un jeu à la base de données
    pub fn add_game(&mut self, game: GameInfo) {
        self.games.insert(game.short_name.clone(), game);
    }

    /// Met à jour les checksums d'une ROM dans la base de données
    pub fn update_rom_checksums(
        &mut self,
        game_name: &str,
        rom_filename: &str,
        crc32: u32,
        md5: String,
    ) -> bool {
        if let Some(game) = self.games.get_mut(game_name) {
            // Chercher dans les ROMs requises
            for rom in &mut game.required_roms {
                if rom.filename == rom_filename {
                    rom.crc32 = crc32;
                    rom.md5 = md5;
                    return true;
                }
            }

            // Chercher dans les ROMs optionnelles
            for rom in &mut game.optional_roms {
                if rom.filename == rom_filename {
                    rom.crc32 = crc32;
                    rom.md5 = md5;
                    return true;
                }
            }
        }
        false
    }

    /// Met à jour les checksums depuis un ensemble de ROMs chargées
    pub fn update_checksums_from_loaded_roms(
        &mut self,
        game_name: &str,
        loaded_roms: &std::collections::HashMap<String, super::loader::LoadedRom>,
    ) {
        for (filename, loaded_rom) in loaded_roms {
            if loaded_rom.validation.is_valid {
                self.update_rom_checksums(
                    game_name,
                    filename,
                    loaded_rom.validation.calculated_crc32,
                    loaded_rom.validation.calculated_md5.clone(),
                );
            }
        }
    }

    /// Charge la base de données depuis un fichier JSON
    pub fn load_from_file(&mut self, path: &str) -> anyhow::Result<()> {
        let content = std::fs::read_to_string(path)?;
        let games: Vec<GameInfo> = serde_json::from_str(&content)?;

        for game in games {
            self.add_game(game);
        }

        Ok(())
    }

    /// Sauvegarde la base de données dans un fichier JSON
    pub fn save_to_file(&self, path: &str) -> anyhow::Result<()> {
        let games: Vec<&GameInfo> = self.games.values().collect();
        let content = serde_json::to_string_pretty(&games)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Ajoute les jeux Model 2 connus
    fn add_known_games(&mut self) {
        // Virtua Fighter 2
        self.add_game(GameInfo {
            name: "Virtua Fighter 2".to_string(),
            short_name: "vf2".to_string(),
            developer: "Sega AM2".to_string(),
            year: 1994,
            region: "World".to_string(),
            version: "2.1".to_string(),
            required_roms: vec![
                RomInfo {
                    filename: "epr-17574.30".to_string(),
                    rom_type: RomType::Program,
                    size: 524288, // 512KB
                    crc32: 0x00000000, // Placeholder - will be updated when real ROM is loaded
                    md5: "".to_string(), // Placeholder - will be updated when real ROM is loaded
                    load_address: 0x00000000,
                    bank: 0,
                    required: true,
                },
                RomInfo {
                    filename: "epr-18022.ic2".to_string(),
                    rom_type: RomType::Program,
                    size: 65536, // 64KB
                    crc32: 0x00000000, // Placeholder
                    md5: "".to_string(), // Placeholder
                    load_address: 0x00080000,
                    bank: 0,
                    required: true,
                },
            ],
            optional_roms: vec![],
            system_config: SystemConfig {
                cpu_frequency: 25_000_000,
                display_resolution: (640, 480),
                refresh_rate: 60.0,
                audio_config: AudioConfig {
                    sample_rate: 44100,
                    channels: 2,
                    use_scsp: true,
                },
                graphics_config: GraphicsConfig {
                    texture_mapping: true,
                    transparency: true,
                    antialiasing: false,
                    texture_planes: 4,
                },
                supported_controls: vec!["joystick".to_string(), "6buttons".to_string()],
            },
            description: "Revolutionary 3D fighting game featuring realistic character models and fluid animation.".to_string(),
        });

        // Daytona USA
        self.add_game(GameInfo {
            name: "Daytona USA".to_string(),
            short_name: "daytona".to_string(),
            developer: "Sega AM2".to_string(),
            year: 1993,
            region: "World".to_string(),
            version: "1.0".to_string(),
            required_roms: vec![RomInfo {
                filename: "epr-16724a.6".to_string(),
                rom_type: RomType::Program,
                size: 524288,        // 512KB
                crc32: 0x00000000,   // Placeholder
                md5: "".to_string(), // Placeholder
                load_address: 0x00000000,
                bank: 0,
                required: true,
            }],
            optional_roms: vec![],
            system_config: SystemConfig {
                cpu_frequency: 25_000_000,
                display_resolution: (640, 480),
                refresh_rate: 60.0,
                audio_config: AudioConfig {
                    sample_rate: 44100,
                    channels: 2,
                    use_scsp: true,
                },
                graphics_config: GraphicsConfig {
                    texture_mapping: true,
                    transparency: true,
                    antialiasing: true,
                    texture_planes: 6,
                },
                supported_controls: vec!["steering".to_string(), "pedals".to_string()],
            },
            description: "Groundbreaking 3D racing game featuring the Daytona Speedway."
                .to_string(),
        });

        // Virtua Cop
        self.add_game(GameInfo {
            name: "Virtua Cop".to_string(),
            short_name: "vcop".to_string(),
            developer: "Sega AM2".to_string(),
            year: 1994,
            region: "World".to_string(),
            version: "1.0".to_string(),
            required_roms: vec![RomInfo {
                filename: "epr-17168a.6".to_string(),
                rom_type: RomType::Program,
                size: 524288,        // 512KB
                crc32: 0x00000000,   // Placeholder
                md5: "".to_string(), // Placeholder
                load_address: 0x00000000,
                bank: 0,
                required: true,
            }],
            optional_roms: vec![],
            system_config: SystemConfig {
                cpu_frequency: 25_000_000,
                display_resolution: (640, 480),
                refresh_rate: 60.0,
                audio_config: AudioConfig {
                    sample_rate: 44100,
                    channels: 2,
                    use_scsp: true,
                },
                graphics_config: GraphicsConfig {
                    texture_mapping: true,
                    transparency: true,
                    antialiasing: false,
                    texture_planes: 4,
                },
                supported_controls: vec!["lightgun".to_string()],
            },
            description: "Revolutionary light gun shooter with polygonal graphics.".to_string(),
        });
    }
}

impl Default for GameDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_database() {
        let db = GameDatabase::new();

        // Test de recherche
        assert!(db.find_game("vf2").is_some());
        assert!(db.find_game("Virtua Fighter").is_some());
        assert!(db.find_game("unknown_game").is_none());

        // Test de liste
        let games = db.list_games();
        assert!(games.len() >= 3);
    }

    #[test]
    fn test_rom_info() {
        let rom_info = RomInfo {
            filename: "test.bin".to_string(),
            rom_type: RomType::Program,
            size: 1024,
            crc32: 0x12345678,
            md5: "test".to_string(),
            load_address: 0x1000,
            bank: 1,
            required: true,
        };

        assert_eq!(rom_info.rom_type, RomType::Program);
        assert!(rom_info.required);
    }
}
