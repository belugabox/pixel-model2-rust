//! Tests d'intégration pour l'émulateur Model 2

use pixel_model2_rust::*;

/// Test basique d'initialisation de la mémoire
#[test]
fn test_memory_initialization() {
    let memory = memory::Model2Memory::new();

    // Test de lecture dans une zone non initialisée
    let result = memory.read_u32(0x00000000);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

/// Test de lecture/écriture mémoire basique
#[test]
fn test_memory_read_write() {
    let mut memory = memory::Model2Memory::new();

    // Test d'écriture et lecture
    let test_address = 0x00001000;
    let test_value = 0x12345678;

    memory.write_u32(test_address, test_value).unwrap();
    let read_value = memory.read_u32(test_address).unwrap();
    assert_eq!(read_value, test_value);
}

/// Test d'initialisation du CPU
#[test]
fn test_cpu_initialization() {
    let cpu = cpu::NecV60::new();

    // Vérifier l'état initial
    assert_eq!(cpu.cycle_count, 0);
    assert!(!cpu.halted);
}

/// Test de chargement de ROM basique
#[test]
fn test_rom_loading() {
    use memory::rom::*;

    // Créer une ROM factice
    let test_data = vec![0x12, 0x34, 0x56, 0x78];
    let rom = Rom::new(test_data.clone());

    assert_eq!(rom.size(), 4);
    assert_eq!(rom.read_u8(0).unwrap(), 0x12);
    assert_eq!(rom.read_u32(0).unwrap(), 0x78563412); // Little endian

    // Test de vérification d'intégrité
    assert!(rom.verify_integrity());
}

/// Test de sérialisation de configuration
#[test]
fn test_config_serialization() {
    let config = config::EmulatorConfig::default();

    // Test de sérialisation TOML
    let toml_string = toml::to_string(&config).unwrap();
    assert!(toml_string.contains("resolution"));
    assert!(toml_string.contains("volume"));

    // Test de désérialisation
    let deserialized: config::EmulatorConfig = toml::from_str(&toml_string).unwrap();
    assert_eq!(deserialized.video.resolution, config.video.resolution);
    assert_eq!(deserialized.audio.volume, config.audio.volume);
}

/// Test d'initialisation du gestionnaire d'entrée
#[test]
fn test_input_manager() {
    let input = input::InputManager::new();

    // Test initial
    assert!(!input.player1.up);
    assert!(!input.player1.punch);

    // Note: Test de gestion des touches désactivé temporairement
    // à cause de la déprécation de VirtualKeyCode dans winit
    println!("Input manager initialized successfully");
}

/// Test de base de données ROM vide
#[test]
fn test_rom_database_empty() {
    use rom::database::GameDatabase;

    let database = GameDatabase::new();

    // Test base de données avec jeux connus
    let games = database.list_games();
    assert!(games.len() > 0); // Devrait contenir les jeux connus

    println!("✅ Test ROM: système de base OK");
}