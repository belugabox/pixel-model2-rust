use pixel_model2_rust::rom::Model2RomSystem;
use std::env;

fn main() -> anyhow::Result<()> {
    println!("=== Test de chargement ROM Pixel Model 2 ===\n");

    // Créer le système ROM
    let mut rom_system = Model2RomSystem::new();

    // Ajouter le répertoire roms/ comme chemin de recherche
    rom_system.add_search_path("./roms");

    // Générer un rapport de disponibilité
    let report = rom_system.generate_status_report()?;
    println!("{}", report);

    // Tester le chargement d'un jeu simple
    println!("\n=== Test de chargement Virtua Fighter 2 ===");

    match rom_system.rom_manager.load_game("vf2") {
        Ok(rom_set) => {
            println!("✅ Virtua Fighter 2 chargé avec succès !");
            println!("  ROMs chargées: {}", rom_set.roms.len());
            println!("  Ensemble valide: {}", rom_set.is_valid);
            println!("  Taille totale: {} octets", rom_set.memory_map.total_size);

            for (name, rom) in &rom_set.roms {
                println!("    {}: {} octets, valide: {}",
                        name, rom.data.len(), rom.validation.is_valid);
            }
        },
        Err(e) => {
            println!("❌ Échec du chargement de Virtua Fighter 2: {}", e);
        }
    }

    Ok(())
}