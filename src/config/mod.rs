//! Configuration de l'émulateur

use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::fs;

/// Configuration principale de l'émulateur
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulatorConfig {
    pub video: VideoConfig,
    pub audio: AudioConfig,
    pub input: InputConfig,
    pub emulation: EmulationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoConfig {
    pub resolution: String, // "496x384" ou "640x480"
    pub fullscreen: bool,
    pub vsync: bool,
    pub texture_filtering: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub enabled: bool,
    pub volume: f32,
    pub sample_rate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    pub player1_keys: PlayerKeyConfig,
    pub player2_keys: PlayerKeyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerKeyConfig {
    pub up: String,
    pub down: String,
    pub left: String,
    pub right: String,
    pub punch: String,
    pub kick: String,
    pub guard: String,
    pub start: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulationConfig {
    pub cpu_speed_multiplier: f32,
    pub accurate_timing: bool,
    pub debug_mode: bool,
}

impl Default for EmulatorConfig {
    fn default() -> Self {
        Self {
            video: VideoConfig {
                resolution: "496x384".to_string(),
                fullscreen: false,
                vsync: true,
                texture_filtering: "linear".to_string(),
            },
            audio: AudioConfig {
                enabled: true,
                volume: 1.0,
                sample_rate: 44100,
            },
            input: InputConfig {
                player1_keys: PlayerKeyConfig {
                    up: "W".to_string(),
                    down: "S".to_string(),
                    left: "A".to_string(),
                    right: "D".to_string(),
                    punch: "J".to_string(),
                    kick: "K".to_string(),
                    guard: "L".to_string(),
                    start: "Return".to_string(),
                },
                player2_keys: PlayerKeyConfig {
                    up: "Up".to_string(),
                    down: "Down".to_string(),
                    left: "Left".to_string(),
                    right: "Right".to_string(),
                    punch: "Numpad1".to_string(),
                    kick: "Numpad2".to_string(),
                    guard: "Numpad3".to_string(),
                    start: "NumpadEnter".to_string(),
                },
            },
            emulation: EmulationConfig {
                cpu_speed_multiplier: 1.0,
                accurate_timing: true,
                debug_mode: false,
            },
        }
    }
}

impl EmulatorConfig {
    pub fn load_from_file(path: &str) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: EmulatorConfig = toml::from_str(&contents)?;
        Ok(config)
    }
    
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let contents = toml::to_string_pretty(self)?;
        fs::write(path, contents)?;
        Ok(())
    }
    
    pub fn load_or_default(path: &str) -> Self {
        Self::load_from_file(path).unwrap_or_default()
    }
}