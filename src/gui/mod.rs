//! Interface graphique de l'émulateur

use anyhow::Result;
use winit::{
    event::{Event, WindowEvent, ElementState},
    event_loop::EventLoop,
    window::WindowBuilder,
    keyboard::{KeyCode, PhysicalKey},
};
use crate::{
    cpu::NecV60,
    memory::Model2Memory,
    gpu::Model2Gpu,
    audio::ScspAudio,
    input::InputManager,
    config::EmulatorConfig,
    rom::RomLoader,
};

/// Application principale de l'émulateur
pub struct EmulatorApp<'window> {
    pub cpu: NecV60,
    pub memory: Model2Memory,
    pub gpu: Option<Model2Gpu<'window>>,
    pub audio: ScspAudio,
    pub input: InputManager,
    pub config: EmulatorConfig,
    pub rom_loader: RomLoader,
    pub running: bool,
    pub paused: bool,
}

impl<'window> EmulatorApp<'window> {
    pub fn new(rom_path: Option<String>) -> Result<Self> {
        let config = EmulatorConfig::load_or_default("config.toml");
        let memory = Model2Memory::new();
        let rom_loader = RomLoader::new();
        
        // Charger la ROM si fournie
        if let Some(path) = rom_path {
            println!("Tentative de chargement de la ROM: {}", path);
            // TODO: Charger et intégrer la ROM
        }
        
        Ok(Self {
            cpu: NecV60::new(),
            memory,
            gpu: None,
            audio: ScspAudio::new()?,
            input: InputManager::new(),
            config,
            rom_loader,
            running: true,
            paused: false,
        })
    }
    
    pub fn run(mut self) -> Result<()> {
        let event_loop = EventLoop::new()?;
        let window = WindowBuilder::new()
            .with_title("Pixel Model 2 Rust - Émulateur SEGA Model 2")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
            .build(&event_loop)?;
        
        // Initialiser le GPU
        let gpu = pollster::block_on(Model2Gpu::new(&window))?;
        self.gpu = Some(gpu);
        
        event_loop.run(move |event, elwt| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                    },
                    WindowEvent::KeyboardInput { event, .. } => {
                        if let PhysicalKey::Code(keycode) = event.physical_key {
                            self.input.handle_key(keycode, event.state);
                            
                            // Touches spéciales de l'émulateur
                            if event.state == ElementState::Pressed {
                                match keycode {
                                    KeyCode::Escape => {
                                        elwt.exit();
                                    },
                                    KeyCode::KeyP => {
                                        self.paused = !self.paused;
                                        println!("Émulation {}", if self.paused { "pausée" } else { "reprise" });
                                    },
                                    KeyCode::KeyR => {
                                        self.cpu.reset();
                                        println!("Émulateur réinitialisé");
                                    },
                                    _ => {}
                                }
                            }
                        }
                    },
                    WindowEvent::Resized(physical_size) => {
                        if let Some(gpu) = &mut self.gpu {
                            gpu.renderer.resize(physical_size);
                        }
                    },
                    _ => {}
                },
                Event::AboutToWait => {
                    if self.running && !self.paused {
                        // Exécuter un frame d'émulation
                        if let Err(e) = self.run_frame() {
                            eprintln!("Erreur d'émulation: {}", e);
                        }
                    }
                    window.request_redraw();
                },
                Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                    if let Some(gpu) = &mut self.gpu {
                        let _ = gpu.begin_frame();
                        let _ = gpu.end_frame();
                    }
                },
                _ => {}
            }
        })?;
        Ok(())
    }
    
    fn run_frame(&mut self) -> Result<()> {
        // Exécuter des cycles CPU
        const CYCLES_PER_FRAME: u32 = crate::MAIN_CPU_FREQUENCY / 60; // 60 FPS
        self.cpu.run_cycles(CYCLES_PER_FRAME, &mut self.memory)?;
        
        Ok(())
    }
    
    pub fn load_rom(&mut self, game_name: &str) -> Result<()> {
        let rom_set = self.rom_loader.load_game(game_name)?;
        
        // Charger les ROMs dans la mémoire
        for (name, rom) in rom_set.list_roms() {
            println!("Chargement de la ROM: {} ({} octets)", name, rom.size());
            // TODO: Charger dans les bonnes zones mémoire selon le type
        }
        
        println!("Jeu chargé: {}", rom_set.game_info().name);
        Ok(())
    }
}