use anyhow::Result;
use log::info;
use std::env;

mod cpu;
mod memory;
// mod gpu; // Temporarily disabled
// mod audio; // Temporarily disabled
mod input;
mod rom;
// mod gui; // Temporarily disabled
mod config;

use pixel_model2_rust::gui::EmulatorApp;

fn main() -> Result<()> {
    // Initialiser le logging
    env_logger::init();
    info!("Démarrage de Pixel Model 2 Rust Emulator");

    // Parser les arguments de ligne de commande
    let args: Vec<String> = env::args().collect();
    let mut rom_path: Option<String> = None;

    // Traitement simple des arguments
    for i in 1..args.len() {
        if args[i] == "--rom" && i + 1 < args.len() {
            rom_path = Some(args[i + 1].clone());
        }
    }

    // Créer et lancer l'application
    let app = EmulatorApp::new(rom_path)?;
    app.run()?;

    Ok(())
}