//! Pixel Model 2 Rust - Émulateur SEGA Model 2
//! 
//! Cette bibliothèque fournit tous les composants nécessaires pour émuler
//! le système d'arcade SEGA Model 2, incluant le CPU, GPU, audio et plus.

pub mod cpu;
pub mod memory;
// pub mod gpu; // Temporarily disabled
pub mod audio; // Re-enabled after fixing compilation issues
pub mod input;
pub mod rom;
// pub mod gui; // Temporairement désactivé à cause des problèmes de lifetime
pub mod config;

pub use cpu::*;
pub use memory::*;
// pub use gpu::*; // Temporarily disabled
pub use audio::*; // Re-enabled
pub use input::*;
pub use rom::*;
// pub use gui::*; // Temporairement désactivé
pub use config::*;

/// Version de l'émulateur
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Fréquence du CPU principal (NEC V60) en Hz
pub const MAIN_CPU_FREQUENCY: u32 = 25_000_000; // 25MHz

/// Fréquence du CPU audio (68000) en Hz  
pub const AUDIO_CPU_FREQUENCY: u32 = 11_300_000; // 11.3MHz

/// Taille de la RAM principale
pub const MAIN_RAM_SIZE: usize = 8 * 1024 * 1024; // 8MB

/// Taille de la VRAM
pub const VIDEO_RAM_SIZE: usize = 4 * 1024 * 1024; // 4MB

/// Taille de la RAM audio
pub const AUDIO_RAM_SIZE: usize = 512 * 1024; // 512KB