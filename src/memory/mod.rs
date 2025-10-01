//! Système de gestion mémoire pour le SEGA Model 2
//! 
//! Le Model 2 possède plusieurs types de mémoire mappée :
//! - RAM principale (8MB)
//! - VRAM (4MB) 
//! - RAM audio (512KB)
//! - Zones ROM
//! - Registres I/O

pub mod interface;
pub mod mapping;
pub mod ram;
pub mod rom;

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::cell::RefCell;

pub use interface::*;
pub use mapping::*;
pub use ram::*;
pub use rom::*;

/// Registres I/O du SEGA Model 2
#[derive(Debug, Clone)]
pub struct IoRegisters {
    /// Registre de contrôle des interruptions (0xC0000000)
    pub interrupt_control: u32,
    
    /// Registre de statut des interruptions (0xC0000004)
    pub interrupt_status: u32,
    
    /// Timer principal (0xC0000010)
    pub timer_main: u32,
    
    /// Timer de sous-système (0xC0000014)
    pub timer_sub: u32,
    
    /// Registre de contrôle GPU (0xC0000020)
    pub gpu_control: u32,
    
    /// Registre de statut GPU (0xC0000024)
    pub gpu_status: u32,
    
    /// Registre de commande GPU (0xC0000028)
    pub gpu_command: u32,
    
    /// Registre de contrôle audio (0xC0000030)
    pub audio_control: u32,
    
    /// Registre d'entrée (0xC0000040)
    pub input_data: u32,
    
    /// Registre de contrôle d'entrée (0xC0000044)
    pub input_control: u32,
    
    /// Compteur de cycles CPU pour timing
    cycle_counter: u64,
}

impl IoRegisters {
    pub fn new() -> Self {
        Self {
            interrupt_control: 0,
            interrupt_status: 0,
            timer_main: 0,
            timer_sub: 0,
            gpu_control: 0,
            gpu_status: 0x00000001, // GPU prêt
            gpu_command: 0,
            audio_control: 0,
            input_data: 0,
            input_control: 0,
            cycle_counter: 0,
        }
    }
    
    /// Lit un registre I/O
    pub fn read_register(&self, offset: u32) -> u32 {
        match offset {
            0x00 => self.interrupt_control,
            0x04 => self.interrupt_status,
            0x10 => self.timer_main,
            0x14 => self.timer_sub,
            0x20 => self.gpu_control,
            0x24 => self.gpu_status,
            0x28 => self.gpu_command,
            0x30 => self.audio_control,
            0x40 => self.input_data,
            0x44 => self.input_control,
            _ => 0x00000000,
        }
    }
    
    /// Écrit dans un registre I/O
    pub fn write_register(&mut self, offset: u32, value: u32) -> Option<GpuCommand> {
        match offset {
            0x00 => self.interrupt_control = value,
            0x04 => self.interrupt_status = value,
            0x10 => self.timer_main = value,
            0x14 => self.timer_sub = value,
            0x20 => self.gpu_control = value,
            0x24 => self.gpu_status = value,
            0x28 => {
                self.gpu_command = value;
                // Pour l'instant, traiter les commandes GPU simples
                // TODO: Implémenter un système de commandes plus sophistiqué
                return Some(self.decode_gpu_command(value));
            },
            0x30 => self.audio_control = value,
            0x40 => self.input_data = value,
            0x44 => self.input_control = value,
            _ => {} // Ignorer les registres inconnus
        }
        None
    }
    
    /// Décode une commande GPU (version étendue)
    fn decode_gpu_command(&self, command: u32) -> GpuCommand {
        // Extraire le type de commande des bits de poids fort
        let cmd_type = (command >> 24) & 0xFF;
        
        match cmd_type {
            0x00 => {
                // Clear screen - commande simple
                let r = ((command >> 16) & 0xFF) as f32 / 255.0;
                let g = ((command >> 8) & 0xFF) as f32 / 255.0;
                let b = (command & 0xFF) as f32 / 255.0;
                let a = 1.0; // Alpha par défaut
                GpuCommand::ClearScreen { 
                    color: [r, g, b, a], 
                    depth: 1.0, 
                    stencil: 0 
                }
            },
            0x01 => {
                // Set render state
                let state_bits = (command >> 16) & 0xFF;
                let enabled = (command & 0x01) != 0;
                let state_type = match state_bits {
                    0x01 => RenderStateType::ZBuffer,
                    0x02 => RenderStateType::Texturing,
                    0x04 => RenderStateType::Lighting,
                    0x08 => RenderStateType::Transparency,
                    0x10 => RenderStateType::AlphaTest,
                    0x20 => RenderStateType::Fog,
                    0x40 => RenderStateType::Wireframe,
                    0x80 => RenderStateType::BackfaceCulling,
                    _ => RenderStateType::ZBuffer, // Défaut
                };
                GpuCommand::SetRenderState { state: state_type, enabled }
            },
            0x02 => {
                // Load texture (placeholder - nécessiterait plus de données)
                GpuCommand::LoadTexture { 
                    id: (command >> 16) & 0xFF, 
                    data: vec![], // Données vides pour l'instant
                    width: 64, 
                    height: 64 
                }
            },
            0x10 => {
                // Set model matrix (placeholder - nécessiterait lecture de données supplémentaires)
                GpuCommand::SetModelMatrix([
                    1.0, 0.0, 0.0, 0.0,
                    0.0, 1.0, 0.0, 0.0,
                    0.0, 0.0, 1.0, 0.0,
                    0.0, 0.0, 0.0, 1.0,
                ])
            },
            0x11 => {
                // Set view matrix (placeholder)
                GpuCommand::SetViewMatrix([
                    1.0, 0.0, 0.0, 0.0,
                    0.0, 1.0, 0.0, 0.0,
                    0.0, 0.0, 1.0, 0.0,
                    0.0, 0.0, -2.0, 1.0,
                ])
            },
            0x12 => {
                // Set projection matrix (placeholder)
                GpuCommand::SetProjectionMatrix([
                    1.0, 0.0, 0.0, 0.0,
                    0.0, 1.0, 0.0, 0.0,
                    0.0, 0.0, 1.0, 0.0,
                    0.0, 0.0, 0.0, 1.0,
                ])
            },
            _ => {
                // Commande inconnue - utiliser clear screen par défaut
                println!("GPU: Commande inconnue {:08X}, utilisation de ClearScreen par défaut", command);
                GpuCommand::ClearScreen { 
                    color: [0.0, 0.0, 0.0, 1.0], 
                    depth: 1.0, 
                    stencil: 0 
                }
            }
        }
    }
    
    /// Met à jour les timers et autres registres périodiques
    pub fn update(&mut self, cycles: u32, cpu: &mut crate::cpu::NecV60) {
        self.cycle_counter = self.cycle_counter.wrapping_add(cycles as u64);
        
        // Mise à jour des timers (simplifiée)
        self.timer_main = self.timer_main.wrapping_add(cycles);
        self.timer_sub = self.timer_sub.wrapping_add(cycles / 4); // Timer plus lent
        
        // Générer des interruptions périodiques (VBLANK à ~60Hz)
        if self.cycle_counter % (25_000_000 / 60) == 0 {
            self.interrupt_status |= 0x00000001; // VBLANK interrupt
            cpu.queue_interrupt(crate::cpu::Interrupt::VBlank);
        }
    }
}

/// Types de commandes GPU pour SEGA Model 2
#[derive(Debug, Clone)]
pub enum GpuCommand {
    /// Définit une matrice de modèle
    SetModelMatrix([f32; 16]),
    
    /// Définit une matrice de vue
    SetViewMatrix([f32; 16]),
    
    /// Définit une matrice de projection
    SetProjectionMatrix([f32; 16]),
    
    /// Définit la matrice de texture
    SetTextureMatrix([f32; 16]),
    
    /// Charge une texture
    LoadTexture { id: u32, data: Vec<u8>, width: u32, height: u32 },
    
    /// Charge une texture depuis la ROM
    LoadTextureFromRom { id: u32, rom_offset: u32, width: u32, height: u32, format: TextureFormat },
    
    /// Dessine un triangle texturé
    DrawTriangle { vertices: [GpuVertex; 3], texture_id: Option<u32> },
    
    /// Dessine un quad texturé
    DrawQuad { vertices: [GpuVertex; 4], texture_id: Option<u32> },
    
    /// Dessine une ligne
    DrawLine { start: GpuVertex, end: GpuVertex },
    
    /// Définit l'état de rendu
    SetRenderState { state: RenderStateType, enabled: bool },
    
    /// Définit les paramètres d'éclairage
    SetLighting { light_id: u32, position: [f32; 3], color: [f32; 3], intensity: f32 },
    
    /// Définit les paramètres de brouillard
    SetFog { enabled: bool, start: f32, end: f32, color: [f32; 4], mode: FogMode },
    
    /// Définit le viewport
    SetViewport { x: u32, y: u32, width: u32, height: u32 },
    
    /// Définit les plans de clipping
    SetClipPlanes { near: f32, far: f32 },
    
    /// Efface l'écran ou le framebuffer
    ClearScreen { color: [f32; 4], depth: f32, stencil: u8 },
    
    /// Définit le mode de blending
    SetBlendMode { src_factor: BlendFactor, dst_factor: BlendFactor },
    
    /// Définit le test de profondeur
    SetDepthTest { enabled: bool, func: DepthFunc },
    
    /// Définit le culling
    SetCulling { mode: CullMode },
    
    /// Définit la couleur ambiante
    SetAmbientColor { color: [f32; 3] },
    
    /// Définit les paramètres de texture environment
    SetTextureEnvironment { env_mode: TexEnvMode, combine_rgb: TexCombineMode },
    
    /// Commence une liste de display
    BeginDisplayList { id: u32 },
    
    /// Termine une liste de display
    EndDisplayList { id: u32 },
    
    /// Exécute une liste de display
    ExecuteDisplayList { id: u32 },
    
    /// Définit les paramètres de transformation géométrique
    SetGeometryParams { scale: [f32; 3], rotation: [f32; 3], translation: [f32; 3] },
}

/// Formats de texture supportés par SEGA Model 2
#[derive(Debug, Clone, Copy)]
pub enum TextureFormat {
    Rgba8888,
    Rgb565,
    Rgba4444,
    Indexed4Bpp,
    Indexed8Bpp,
}

/// Types d'états de rendu
#[derive(Debug, Clone, Copy)]
pub enum RenderStateType {
    ZBuffer,
    Texturing,
    Lighting,
    Transparency,
    AlphaTest,
    Fog,
    Wireframe,
    BackfaceCulling,
}

/// Modes de brouillard
#[derive(Debug, Clone, Copy)]
pub enum FogMode {
    Linear,
    Exponential,
    ExponentialSquared,
}

/// Facteurs de blending
#[derive(Debug, Clone, Copy)]
pub enum BlendFactor {
    Zero,
    One,
    SrcColor,
    OneMinusSrcColor,
    DstColor,
    OneMinusDstColor,
    SrcAlpha,
    OneMinusSrcAlpha,
    DstAlpha,
    OneMinusDstAlpha,
}

/// Fonctions de test de profondeur
#[derive(Debug, Clone, Copy)]
pub enum DepthFunc {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}

/// Modes de culling
#[derive(Debug, Clone, Copy)]
pub enum CullMode {
    None,
    Front,
    Back,
    FrontAndBack,
}

/// Modes d'environnement de texture
#[derive(Debug, Clone, Copy)]
pub enum TexEnvMode {
    Modulate,
    Decal,
    Blend,
    Replace,
}

/// Modes de combinaison de texture
#[derive(Debug, Clone, Copy)]
pub enum TexCombineMode {
    Replace,
    Modulate,
    Add,
    AddSigned,
    Interpolate,
}

/// Représentation d'un vertex pour les commandes GPU
#[derive(Debug, Clone, Copy)]
pub struct GpuVertex {
    pub x: f32, pub y: f32, pub z: f32,
    pub r: f32, pub g: f32, pub b: f32, pub a: f32,
    pub u: f32, pub v: f32,
}

impl GpuVertex {
    pub fn new(x: f32, y: f32, z: f32, r: f32, g: f32, b: f32, a: f32, u: f32, v: f32) -> Self {
        Self { x, y, z, r, g, b, a, u, v }
    }
}

/// Bus mémoire principal du SEGA Model 2
#[derive(Debug)]
pub struct Model2Memory {
    /// RAM principale (8MB)
    pub main_ram: Ram,
    
    /// VRAM (4MB)
    pub video_ram: Ram,
    
    /// RAM audio (512KB)
    pub audio_ram: Ram,
    
    /// Gestionnaire de mappage mémoire
    pub mapping: MemoryMap,
    
    /// ROMs chargées
    pub roms: HashMap<String, Rom>,
    
    /// Cache des accès mémoire pour optimisation
    cache: RefCell<MemoryCache>,
    
    /// Activation du cache
    cache_enabled: bool,

    /// Registres I/O
    io_registers: IoRegisters,
    
    /// File de commandes GPU en attente
    gpu_command_queue: Vec<GpuCommand>,
}

impl Model2Memory {
    /// Crée un nouveau système mémoire Model 2
    pub fn new() -> Self {
        Self {
            main_ram: Ram::new(8 * 1024 * 1024), // 8MB
            video_ram: Ram::new(4 * 1024 * 1024), // 4MB
            audio_ram: Ram::new(512 * 1024), // 512KB
            mapping: MemoryMap::new_model2(),
            roms: HashMap::new(),
            cache: RefCell::new(MemoryCache::new()),
            cache_enabled: true,
            io_registers: IoRegisters::new(),
            gpu_command_queue: Vec::new(),
        }
    }
    
    /// Charge une ROM dans le système
    pub fn load_rom(&mut self, name: String, data: Vec<u8>) -> Result<()> {
        let rom = Rom::new(data);
        self.roms.insert(name, rom);
        Ok(())
    }
    
    /// Vide le cache mémoire
    pub fn clear_cache(&mut self) {
        if let Ok(mut cache) = self.cache.try_borrow_mut() {
            cache.clear();
        }
    }
    
    /// Met à jour les registres I/O (appelé périodiquement)
    pub fn update_io_registers(&mut self, cycles: u32, cpu: &mut crate::cpu::NecV60) {
        self.io_registers.update(cycles, cpu);
    }
    
    /// Enfile une commande GPU
    pub fn enqueue_gpu_command(&mut self, command: GpuCommand) {
        self.gpu_command_queue.push(command);
    }
    
    /// Traite toutes les commandes GPU en attente
    pub fn process_gpu_commands(&mut self) -> Vec<GpuCommand> {
        let commands = self.gpu_command_queue.clone();
        self.gpu_command_queue.clear();
        commands
    }
    
    /// Obtient le nombre de commandes GPU en attente
    pub fn gpu_command_count(&self) -> usize {
        self.gpu_command_queue.len()
    }
}

impl MemoryInterface for Model2Memory {
    fn read_u8(&self, address: u32) -> Result<u8> {
        // Vérifier le cache d'abord
        if self.cache_enabled {
            if let Ok(cache) = self.cache.try_borrow() {
                if let Some(value) = cache.get_u8(address) {
                    return Ok(value);
                }
            }
        }
        
        // Déterminer la région mémoire et l'offset
        let result = if let Some((region, offset)) = self.mapping.resolve(address) {
            match region {
                MemoryRegion::MainRam => self.main_ram.read_u8(offset),
                MemoryRegion::VideoRam => self.video_ram.read_u8(offset),
                MemoryRegion::AudioRam => self.audio_ram.read_u8(offset),
                MemoryRegion::ProgramRom => {
                    if let Some(rom) = self.roms.get("main") {
                        rom.read_u8(offset)
                    } else {
                        Ok(0xFF)
                    }
                },
                MemoryRegion::GraphicsRom => {
                    if let Some(rom) = self.roms.get("graphics") {
                        rom.read_u8(offset)
                    } else {
                        Ok(0xFF)
                    }
                },
                MemoryRegion::AudioRom => {
                    if let Some(rom) = self.roms.get("audio") {
                        rom.read_u8(offset)
                    } else {
                        Ok(0xFF)
                    }
                },
                MemoryRegion::IoRegisters => {
                    // Lecture des registres I/O
                    Ok(self.io_registers.read_register(offset) as u8)
                },
            }
        } else {
            Ok(0xFF) // Lecture dans une zone non mappée
        };

        // Mettre en cache le résultat si valide
        if let Ok(value) = result {
            if let Ok(mut cache) = self.cache.try_borrow_mut() {
                cache.set_u8(address, value);
            }
        }

        result
    }

    fn read_u16(&self, address: u32) -> Result<u16> {
        // Optimisation : lecture directe pour les accès alignés
        if address % 2 == 0 {
            if let Ok(cache) = self.cache.try_borrow() {
                if let Some(value) = cache.get_u16(address) {
                    return Ok(value);
                }
            }
        }
        
        // Déterminer la région mémoire et l'offset
        let result = if let Some((region, offset)) = self.mapping.resolve(address) {
            match region {
                MemoryRegion::MainRam => self.main_ram.read_u16(offset),
                MemoryRegion::VideoRam => self.video_ram.read_u16(offset),
                MemoryRegion::AudioRam => self.audio_ram.read_u16(offset),
                MemoryRegion::ProgramRom => {
                    if let Some(rom) = self.roms.get("main") {
                        rom.read_u16(offset)
                    } else {
                        Ok(0xFFFF)
                    }
                },
                MemoryRegion::GraphicsRom => {
                    if let Some(rom) = self.roms.get("graphics") {
                        rom.read_u16(offset)
                    } else {
                        Ok(0xFFFF)
                    }
                },
                MemoryRegion::AudioRom => {
                    if let Some(rom) = self.roms.get("audio") {
                        rom.read_u16(offset)
                    } else {
                        Ok(0xFFFF)
                    }
                },
                MemoryRegion::IoRegisters => {
                    // Lecture des registres I/O
                    Ok(self.io_registers.read_register(offset) as u16)
                },
            }
        } else {
            Ok(0xFFFF) // Lecture dans une zone non mappée
        };

        // Mettre en cache le résultat si valide
        if let Ok(value) = result {
            if let Ok(mut cache) = self.cache.try_borrow_mut() {
                cache.set_u16(address, value);
            }
        }

        result
    }

    fn read_u32(&self, address: u32) -> Result<u32> {
        // Optimisation : lecture directe pour les accès alignés
        if address % 4 == 0 {
            if let Ok(cache) = self.cache.try_borrow() {
                if let Some(value) = cache.get_u32(address) {
                    return Ok(value);
                }
            }
        }
        
        // Déterminer la région mémoire et l'offset
        let result = if let Some((region, offset)) = self.mapping.resolve(address) {
            match region {
                MemoryRegion::MainRam => self.main_ram.read_u32(offset),
                MemoryRegion::VideoRam => self.video_ram.read_u32(offset),
                MemoryRegion::AudioRam => self.audio_ram.read_u32(offset),
                MemoryRegion::ProgramRom => {
                    if let Some(rom) = self.roms.get("main") {
                        rom.read_u32(offset)
                    } else {
                        Ok(0xFFFFFFFF)
                    }
                },
                MemoryRegion::GraphicsRom => {
                    if let Some(rom) = self.roms.get("graphics") {
                        rom.read_u32(offset)
                    } else {
                        Ok(0xFFFFFFFF)
                    }
                },
                MemoryRegion::AudioRom => {
                    if let Some(rom) = self.roms.get("audio") {
                        rom.read_u32(offset)
                    } else {
                        Ok(0xFFFFFFFF)
                    }
                },
                MemoryRegion::IoRegisters => {
                    // Lecture des registres I/O
                    Ok(self.io_registers.read_register(offset))
                },
            }
        } else {
            Ok(0xFFFFFFFF) // Lecture dans une zone non mappée
        };

        // Mettre en cache le résultat si valide
        if let Ok(value) = result {
            if let Ok(mut cache) = self.cache.try_borrow_mut() {
                cache.set_u32(address, value);
            }
        }

        result
    }

    fn write_u8(&mut self, address: u32, value: u8) -> Result<()> {
        // Déterminer la région mémoire et l'offset
        if let Some((region, offset)) = self.mapping.resolve(address) {
            match region {
                MemoryRegion::MainRam => self.main_ram.write_u8(offset, value),
                MemoryRegion::VideoRam => self.video_ram.write_u8(offset, value),
                MemoryRegion::AudioRam => self.audio_ram.write_u8(offset, value),
                MemoryRegion::ProgramRom | MemoryRegion::GraphicsRom | MemoryRegion::AudioRom => {
                    // Les ROMs sont en lecture seule
                    Err(anyhow!("Tentative d'écriture en ROM à l'adresse {:08X}", address))
                },
                MemoryRegion::IoRegisters => {
                    // Écriture dans les registres I/O
                    self.io_registers.write_register(offset, value as u32);
                    Ok(())
                },
            }
        } else {
            // Écriture dans une zone non mappée - ignorer silencieusement
            Ok(())
        }
    }

    fn write_u16(&mut self, address: u32, value: u16) -> Result<()> {
        // Alignement vérifié
        if address % 2 != 0 {
            return Err(anyhow!("Écriture u16 non alignée à l'adresse {:08X}", address));
        }
        
        // Déterminer la région mémoire et l'offset
        if let Some((region, offset)) = self.mapping.resolve(address) {
            match region {
                MemoryRegion::MainRam => self.main_ram.write_u16(offset, value),
                MemoryRegion::VideoRam => self.video_ram.write_u16(offset, value),
                MemoryRegion::AudioRam => self.audio_ram.write_u16(offset, value),
                MemoryRegion::ProgramRom | MemoryRegion::GraphicsRom | MemoryRegion::AudioRom => {
                    // Les ROMs sont en lecture seule
                    Err(anyhow!("Tentative d'écriture en ROM à l'adresse {:08X}", address))
                },
                MemoryRegion::IoRegisters => {
                    // Écriture dans les registres I/O
                    self.io_registers.write_register(offset, value as u32);
                    Ok(())
                },
            }
        } else {
            // Écriture dans une zone non mappée - ignorer silencieusement
            Ok(())
        }
    }

    fn write_u32(&mut self, address: u32, value: u32) -> Result<()> {
        // Alignement vérifié
        if address % 4 != 0 {
            return Err(anyhow!("Écriture u32 non alignée à l'adresse {:08X}", address));
        }
        
        // Déterminer la région mémoire et l'offset
        if let Some((region, offset)) = self.mapping.resolve(address) {
            match region {
                MemoryRegion::MainRam => self.main_ram.write_u32(offset, value),
                MemoryRegion::VideoRam => self.video_ram.write_u32(offset, value),
                MemoryRegion::AudioRam => self.audio_ram.write_u32(offset, value),
                MemoryRegion::ProgramRom | MemoryRegion::GraphicsRom | MemoryRegion::AudioRom => {
                    // Les ROMs sont en lecture seule
                    Err(anyhow!("Tentative d'écriture en ROM à l'adresse {:08X}", address))
                },
                MemoryRegion::IoRegisters => {
                    // Écriture dans les registres I/O
                    if let Some(gpu_command) = self.io_registers.write_register(offset, value) {
                        self.enqueue_gpu_command(gpu_command);
                    }
                    Ok(())
                },
            }
        } else {
            // Écriture dans une zone non mappée - ignorer silencieusement
            Ok(())
        }
    }
}

/// Cache mémoire simple pour optimiser les performances
#[derive(Debug)]
struct MemoryCache {
    entries: HashMap<u32, CacheEntry>,
    max_entries: usize,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    value: u32,
    size: u8, // 1, 2, ou 4 octets
}

impl MemoryCache {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            max_entries: 1024, // Limiter la taille du cache
        }
    }

    fn get_u8(&self, address: u32) -> Option<u8> {
        self.entries.get(&address)
            .filter(|entry| entry.size == 1)
            .map(|entry| entry.value as u8)
    }

    fn get_u16(&self, address: u32) -> Option<u16> {
        self.entries.get(&address)
            .filter(|entry| entry.size == 2)
            .map(|entry| entry.value as u16)
    }

    fn get_u32(&self, address: u32) -> Option<u32> {
        self.entries.get(&address)
            .filter(|entry| entry.size == 4)
            .map(|entry| entry.value)
    }

    fn set_u8(&mut self, address: u32, value: u8) {
        self.insert_entry(address, CacheEntry { value: value as u32, size: 1 });
    }

    fn set_u16(&mut self, address: u32, value: u16) {
        self.insert_entry(address, CacheEntry { value: value as u32, size: 2 });
    }

    fn set_u32(&mut self, address: u32, value: u32) {
        self.insert_entry(address, CacheEntry { value, size: 4 });
    }

    fn insert_entry(&mut self, address: u32, entry: CacheEntry) {
        // Éviction si le cache est plein
        if self.entries.len() >= self.max_entries {
            // Stratégie simple : vider la moitié du cache
            let keys: Vec<u32> = self.entries.keys().take(self.max_entries / 2).cloned().collect();
            for key in keys {
                self.entries.remove(&key);
            }
        }
        
        self.entries.insert(address, entry);
    }

    fn clear(&mut self) {
        self.entries.clear();
    }
}