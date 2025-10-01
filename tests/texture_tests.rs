//! Tests complets du système de textures SEGA Model 2
//! 
//! Valide le décodage authentique des formats SEGA et l'intégration WGPU

use pixel_model2_rust::gpu::texture::{
    TextureManager, SegaTextureFormat, TextureDecodeParams
};
use std::sync::Arc;

/// Configuration mock WGPU pour les tests
async fn create_mock_wgpu() -> (Arc<wgpu::Device>, Arc<wgpu::Queue>) {
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .expect("Failed to find an appropriate adapter");

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults(),

            },
            None,
        )
        .await
        .expect("Failed to create device");

    (Arc::new(device), Arc::new(queue))
}

#[tokio::test]
async fn test_texture_manager_creation() {
    let (device, queue) = create_mock_wgpu().await;
    let texture_manager = TextureManager::new(device, queue);
    
    println!("✅ TextureManager créé avec succès");
    assert!(texture_manager.get_texture(0).is_none());
}

#[tokio::test]
async fn test_load_simple_rgba8_texture() {
    let (device, queue) = create_mock_wgpu().await;
    let mut texture_manager = TextureManager::new(device, queue);
    
    // Données de test : texture 2x2 RGBA8 rouge
    let rgba_data = vec![
        255, 0, 0, 255,  // Pixel rouge
        255, 0, 0, 255,  // Pixel rouge
        255, 0, 0, 255,  // Pixel rouge
        255, 0, 0, 255,  // Pixel rouge
    ];
    
    let result = texture_manager.load_texture(1, &rgba_data, 2, 2);
    assert!(result.is_ok(), "Erreur lors du chargement texture RGBA8: {:?}", result.err());
    
    let texture = texture_manager.get_texture(1);
    assert!(texture.is_some(), "Texture non trouvée après chargement");
    
    let texture_data = texture.unwrap();
    assert_eq!(texture_data.width, 2);
    assert_eq!(texture_data.height, 2);
    assert_eq!(texture_data.format, SegaTextureFormat::Rgba8888);
    
    println!("✅ Texture RGBA8 2x2 chargée avec succès");
}

#[tokio::test]
async fn test_sega_palette4bpp_decoding() {
    let (device, queue) = create_mock_wgpu().await;
    let mut texture_manager = TextureManager::new(device, queue);
    
    // Données 4bpp test : 4x4 pixels = 8 bytes (2 pixels par byte)
    let palette_data = vec![
        0x01, 0x23,  // Pixels: 1,0,3,2
        0x45, 0x67,  // Pixels: 5,4,7,6  
        0x89, 0xAB,  // Pixels: 9,8,B,A
        0xCD, 0xEF,  // Pixels: D,C,F,E
    ];
    
    let params = TextureDecodeParams {
        width: 4,
        height: 4,
        format: SegaTextureFormat::Palette4bpp,
        palette_offset: Some(0),
        data_offset: 0,
        stride: Some(2), // 2 bytes par ligne (4 pixels / 2)
    };
    
    let result = texture_manager.load_texture_from_rom(2, &palette_data, params);
    assert!(result.is_ok(), "Erreur décodage 4bpp: {:?}", result.err());
    
    let texture = texture_manager.get_texture(2).unwrap();
    assert_eq!(texture.width, 4);
    assert_eq!(texture.height, 4);
    assert_eq!(texture.format, SegaTextureFormat::Palette4bpp);
    
    println!("✅ Texture 4bpp Palette 4x4 décodée avec succès");
}

#[tokio::test]
async fn test_sega_rgb565_decoding() {
    let (device, queue) = create_mock_wgpu().await;
    let mut texture_manager = TextureManager::new(device, queue);
    
    // Données RGB565 test : 2x2 = 4 pixels = 8 bytes
    let rgb565_data = vec![
        0x00, 0xF8,  // Rouge pur : 0b11111000_00000000 = 63488
        0xE0, 0x07,  // Vert pur  : 0b00000111_11100000 = 2016  
        0x1F, 0x00,  // Bleu pur  : 0b00000000_00011111 = 31
        0xFF, 0xFF,  // Blanc     : 0b11111111_11111111 = 65535
    ];
    
    let params = TextureDecodeParams {
        width: 2,
        height: 2,
        format: SegaTextureFormat::Rgb565,
        palette_offset: None,
        data_offset: 0,
        stride: Some(4), // 2 bytes par pixel * 2 pixels = 4 bytes par ligne
    };
    
    let result = texture_manager.load_texture_from_rom(3, &rgb565_data, params);
    assert!(result.is_ok(), "Erreur décodage RGB565: {:?}", result.err());
    
    let texture = texture_manager.get_texture(3).unwrap();
    assert_eq!(texture.width, 2);
    assert_eq!(texture.height, 2);
    assert_eq!(texture.format, SegaTextureFormat::Rgb565);
    
    println!("✅ Texture RGB565 2x2 décodée avec succès");
}

#[tokio::test]
async fn test_sega_rgba4444_decoding() {
    let (device, queue) = create_mock_wgpu().await;
    let mut texture_manager = TextureManager::new(device, queue);
    
    // Données RGBA4444 test : 2x1 = 2 pixels = 4 bytes
    let rgba4444_data = vec![
        0x0F, 0xF0,  // Rouge semi-transparent : 0xF00F
        0x0F, 0x0F,  // Bleu opaque : 0x0F0F
    ];
    
    let params = TextureDecodeParams {
        width: 2,
        height: 1,
        format: SegaTextureFormat::Rgba4444,
        palette_offset: None,
        data_offset: 0,
        stride: Some(4), // 2 bytes par pixel * 2 pixels = 4 bytes
    };
    
    let result = texture_manager.load_texture_from_rom(4, &rgba4444_data, params);
    assert!(result.is_ok(), "Erreur décodage RGBA4444: {:?}", result.err());
    
    let texture = texture_manager.get_texture(4).unwrap();
    assert_eq!(texture.width, 2);
    assert_eq!(texture.height, 1);
    assert_eq!(texture.format, SegaTextureFormat::Rgba4444);
    
    println!("✅ Texture RGBA4444 2x1 décodée avec succès");
}

#[tokio::test]
async fn test_multiple_textures_management() {
    let (device, queue) = create_mock_wgpu().await;
    let mut texture_manager = TextureManager::new(device, queue);
    
    // Charger plusieurs textures de formats différents
    let rgba_data = vec![255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255]; // 2x2 rouge
    let rgb565_data = vec![0x00, 0xF8, 0xE0, 0x07]; // 2x1 rouge/vert
    
    // Texture 1 : RGBA8888
    texture_manager.load_texture(10, &rgba_data, 2, 2).unwrap();
    
    // Texture 2 : RGB565
    let params = TextureDecodeParams {
        width: 2, height: 1,
        format: SegaTextureFormat::Rgb565,
        palette_offset: None,
        data_offset: 0,
        stride: Some(4),
    };
    texture_manager.load_texture_from_rom(20, &rgb565_data, params).unwrap();
    
    // Vérifier les deux textures
    assert!(texture_manager.get_texture(10).is_some());
    assert!(texture_manager.get_texture(20).is_some());
    assert!(texture_manager.get_texture(99).is_none());
    
    // Vérifier les bind groups
    assert!(texture_manager.get_bind_group(10).is_some());
    assert!(texture_manager.get_bind_group(20).is_some());
    assert!(texture_manager.get_bind_group(99).is_none());
    
    println!("✅ Gestion multi-textures validée");
}

#[tokio::test]
async fn test_texture_bind_groups() {
    let (device, queue) = create_mock_wgpu().await;
    let mut texture_manager = TextureManager::new(device, queue);
    
    let rgba_data: Vec<u8> = (0..64).map(|i| if i % 4 == 1 { 255 } else if i % 4 == 3 { 255 } else { 0 }).collect(); // 4x4 vert
    texture_manager.load_texture(100, &rgba_data, 4, 4).unwrap();
    
    let bind_group = texture_manager.get_bind_group(100);
    assert!(bind_group.is_some(), "Bind group non créé");
    
    println!("✅ Bind groups WGPU créés correctement");
}

/// Test de performance et limites
#[tokio::test]
async fn test_large_texture_handling() {
    let (device, queue) = create_mock_wgpu().await;
    let mut texture_manager = TextureManager::new(device, queue);
    
    // Texture "large" 64x64 RGBA8
    let size = 64 * 64 * 4;
    let large_data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
    
    let result = texture_manager.load_texture(500, &large_data, 64, 64);
    assert!(result.is_ok(), "Erreur texture large: {:?}", result.err());
    
    let texture = texture_manager.get_texture(500).unwrap();
    assert_eq!(texture.width, 64);
    assert_eq!(texture.height, 64);
    
    println!("✅ Texture large 64x64 gérée correctement");
}