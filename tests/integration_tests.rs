//! Tests d'intégration pour l'émulateur Model 2//! Tests d'intégration pour l'émulateur Model 2



use pixel_model2_rust::*;use pixel_model2_rust::*;



#[test]#[test]            Vertex3D {

fn test_cpu_basic_operations() {                position: Vec3::new(0.0, 0.0, 0.0),

    use cpu::registers::*;                normal: Vec3::new(0.0, 0.0, 1.0),

    use cpu::arithmetic::*;                 tex_coords: [0.0, 0.0],

                    color: [1.0, 1.0, 1.0, 1.0],

    // Test basique d'addition                fog_coord: 0.0,

    let result = add_integers(5, 3);                specular: [0.0, 0.0, 0.0],

    match result {            },            Vertex3D {

        ArithmeticResult::Integer(val) => assert_eq!(val, 8),                position: Vec3::new(1.0, 0.0, 0.0),

        _ => panic!("Expected integer result"),                normal: Vec3::new(0.0, 0.0, 1.0),

    }                tex_coords: [1.0, 0.0],

                    color: [1.0, 1.0, 1.0, 1.0],

    println!("✅ Test CPU: addition basique OK");                fog_coord: 0.0,

}                specular: [0.0, 0.0, 0.0],

            },            Vertex3D {

#[test]                position: Vec3::new(0.5, 1.0, 0.0),

fn test_memory_interface() {                normal: Vec3::new(0.0, 0.0, 1.0),

    use memory::ram::VirtualRam;                tex_coords: [0.5, 1.0],

                    color: [1.0, 1.0, 1.0, 1.0],

    let mut ram = VirtualRam::new(1024);                fog_coord: 0.0,

                    specular: [0.0, 0.0, 0.0],

    // Test lecture/écriture            },alization() {

    ram.write_u32(0x100, 0xDEADBEEF).expect("Failed to write");    let cpu = cpu::NecV60::new();

    let value = ram.read_u32(0x100).expect("Failed to read");    assert_eq!(cpu.cycle_count, 0);

        assert!(!cpu.halted);

    assert_eq!(value, 0xDEADBEEF);}

    println!("✅ Test Memory: lecture/écriture OK");

}#[test]

fn test_memory_initialization() {

#[test]    let memory = memory::Model2Memory::new();

fn test_rom_system_integration() {    

    use rom::database::Model2GameDatabase;    // Test de lecture dans une zone non initialisée

        let result = memory.read_u32(0x00000000);

    let mut database = Model2GameDatabase::new();    assert!(result.is_ok());

        assert_eq!(result.unwrap(), 0);

    // Test base de données vide}

    assert_eq!(database.get_all_games().len(), 0);

    #[test]

    println!("✅ Test ROM: système de base OK");fn test_memory_read_write() {

}    let mut memory = memory::Model2Memory::new();

    

#[test]    // Test d'écriture et lecture

fn test_gpu_geometry_processor() {    let test_address = 0x00001000;

    use gpu::geometry::*;    let test_value = 0x12345678;

    use glam::Vec3;    

        memory.write_u32(test_address, test_value).unwrap();

    let mut processor = GeometryProcessor::new(640, 480);    let read_value = memory.read_u32(test_address).unwrap();

        

    // Test de transformation identité    assert_eq!(read_value, test_value);

    let triangle = Triangle3D {}

        vertices: [

            Vertex3D {#[test]

                position: Vec3::new(0.0, 0.0, 0.0),fn test_config_serialization() {

                normal: Vec3::new(0.0, 0.0, 1.0),    let config = config::EmulatorConfig::default();

                tex_coords: [0.0, 0.0],    

                color: [1.0, 1.0, 1.0, 1.0],    // Test de sérialisation TOML

                fog_coord: 0.0,    let toml_string = toml::to_string(&config).unwrap();

                specular: [0.0, 0.0, 0.0],    assert!(toml_string.contains("resolution"));

            },    assert!(toml_string.contains("volume"));

            Vertex3D {    

                position: Vec3::new(1.0, 0.0, 0.0),    // Test de désérialisation

                normal: Vec3::new(0.0, 0.0, 1.0),    let deserialized: config::EmulatorConfig = toml::from_str(&toml_string).unwrap();

                tex_coords: [1.0, 0.0],    assert_eq!(deserialized.video.resolution, config.video.resolution);

                color: [1.0, 1.0, 1.0, 1.0],    assert_eq!(deserialized.audio.volume, config.audio.volume);

                fog_coord: 0.0,}

                specular: [0.0, 0.0, 0.0],

            },#[test]

            Vertex3D {fn test_rom_loading() {

                position: Vec3::new(0.5, 1.0, 0.0),    use memory::rom::*;

                normal: Vec3::new(0.0, 0.0, 1.0),    

                tex_coords: [0.5, 1.0],    // Créer une ROM factice

                color: [1.0, 1.0, 1.0, 1.0],    let test_data = vec![0x12, 0x34, 0x56, 0x78];

                fog_coord: 0.0,    let rom = Rom::new(test_data.clone());

                specular: [0.0, 0.0, 0.0],    

            },    assert_eq!(rom.size(), 4);

        ],    assert_eq!(rom.read_u8(0).unwrap(), 0x12);

        texture_id: None,    assert_eq!(rom.read_u32(0).unwrap(), 0x78563412); // Little endian

        material_id: 0,    

        flags: TriangleFlags::empty(),    // Test de vérification d'intégrité

    };    assert!(rom.verify_integrity());

}

    // Test transformation basique

    let result = processor.transform_triangle(&triangle);#[test]

    assert!(result.is_ok(), "Transformation failed");fn test_input_manager() {

        let input = input::InputManager::new();

    let transformed = result.unwrap();    

    assert_eq!(transformed.vertices.len(), 3);    // Test initial

        assert!(!input.player1.up);

    // Les positions doivent être transformées en clip space    assert!(!input.player1.punch);

    assert_ne!(transformed.vertices[0].world_position, triangle.vertices[0].position);    

        // Note: Test de gestion des touches désactivé temporairement

    println!("✅ Test GPU: transformation géométrique OK");    // à cause de la déprécation de VirtualKeyCode dans winit

}    println!("Input manager initialized successfully");
}

#[test] 
fn test_geometry_processor() {
    use gpu::geometry::*;
    use glam::Vec3;
    
    let mut processor = GeometryProcessor::new(640, 480);
    
    // Test de transformation identité
    let triangle = Triangle3D {
        vertices: [
            Vertex3D {
                position: Vec3::new(0.0, 0.0, 0.0),
                normal: Vec3::new(0.0, 1.0, 0.0),
                tex_coords: [0.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex3D {
                position: Vec3::new(1.0, 0.0, 0.0),
                normal: Vec3::new(0.0, 1.0, 0.0),
                tex_coords: [1.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex3D {
                position: Vec3::new(0.0, 1.0, 0.0),
                normal: Vec3::new(0.0, 1.0, 0.0),
                tex_coords: [0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ],
        texture_id: None,
        material_id: 0,
    };
    
    // Avec les matrices identité, le triangle ne devrait pas changer
    let transformed = processor.transform_triangle(&triangle).unwrap();
    assert_eq!(transformed.vertices[0].position, triangle.vertices[0].position);
}