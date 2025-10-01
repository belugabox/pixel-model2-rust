//! Interface graphique de l'émulateur

use std::sync::Arc;
use anyhow::Result;
use winit::{
    event::{Event, WindowEvent, ElementState},
    event_loop::EventLoop,
    window::WindowBuilder,
    keyboard::{KeyCode, PhysicalKey},
};
use crate::{
    cpu::NecV60,
    memory::{Model2Memory, interface::MemoryInterface, GpuCommand},
    gpu::Model2Gpu,
    audio::ScspAudio,
    input::InputManager,
    config::EmulatorConfig,
    rom::Model2RomSystem,
};

/// Application principale de l'émulateur
pub struct EmulatorApp {
    pub cpu: NecV60,
    pub memory: Model2Memory,
    pub audio: ScspAudio,
    pub input: InputManager,
    pub config: EmulatorConfig,
    pub rom_system: Model2RomSystem,
    pub running: bool,
    pub paused: bool,
}

/// État de l'application pour gérer les lifetimes correctement
pub struct AppState {
    pub app: EmulatorApp,
}

impl AppState {
    pub fn new(app: EmulatorApp) -> Self {
        Self { app }
    }
    
    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                // Nous ne pouvons pas appeler elwt.exit() ici sans elwt
                self.app.running = false;
            },
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(keycode) = event.physical_key {
                    self.app.input.handle_key(keycode, event.state);
                    
                    // Touches spéciales de l'émulateur
                    if event.state == ElementState::Pressed {
                        match keycode {
                            KeyCode::Escape => {
                                self.app.running = false;
                            },
                            KeyCode::KeyP => {
                                self.app.paused = !self.app.paused;
                                println!("Émulation {}", if self.app.paused { "pausée" } else { "reprise" });
                            },
                            KeyCode::KeyR => {
                                self.app.cpu.reset();
                                println!("Émulateur réinitialisé");
                            },
                            KeyCode::KeyL => {
                                // Essayer de charger un jeu de test
                                let _ = self.app.load_rom("daytona-usa");
                            },
                            _ => {}
                        }
                    }
                }
            },
            _ => {}
        }
    }
    
    pub fn run_frame(&mut self, mut gpu: Option<&mut Model2Gpu>) -> Result<()> {
        if self.app.running && !self.app.paused {
            // Exécuter un frame d'émulation
            const CYCLES_PER_FRAME: u32 = pixel_model2_rust::MAIN_CPU_FREQUENCY / 60; // 60 FPS
            let executed_cycles = self.app.cpu.run_cycles(CYCLES_PER_FRAME, &mut self.app.memory)?;
            
            // Mettre à jour les registres I/O avec les cycles exécutés
            self.app.memory.update_io_registers(executed_cycles, &mut self.app.cpu);
            
            // Traiter les commandes GPU par lots
            let command_batches = self.app.memory.process_gpu_commands();
            if !command_batches.is_empty() {
                if let Some(gpu_ref) = gpu.as_mut() {
                    self.process_gpu_command_batch(&command_batches, gpu_ref)?;
                } else {
                    println!("GPU: {} commandes reçues mais GPU non initialisé", command_batches.len());
                }
            }
            
            // Forcer le vidage du buffer à la fin du frame pour synchronisation
            let remaining_commands = self.app.memory.flush_gpu_command_buffer();
            if !remaining_commands.is_empty() {
                if let Some(gpu_ref) = gpu.as_mut() {
                    self.process_gpu_command_batch(&remaining_commands, gpu_ref)?;
                }
            }
            
            // Synchroniser les autres composants (GPU, audio, etc.)
            // TODO: Implémenter une synchronisation temporelle précise
            
            // Statistiques de performance
            if executed_cycles > 0 {
                let fps = 60.0 * (executed_cycles as f32 / CYCLES_PER_FRAME as f32);
                let buffer_stats = self.app.memory.gpu_command_buffer.stats();
                println!("GPU Buffer: {} lots traités, taille moyenne {:.1}, max {}", 
                        buffer_stats.batches_processed, buffer_stats.average_batch_size, buffer_stats.max_batch_size);
            }
        }
        Ok(())
    }
    
    /// Traite une commande GPU
    fn process_gpu_command(&mut self, command: &GpuCommand, gpu: &mut Model2Gpu) -> Result<()> {
        match command {
            GpuCommand::ClearScreen { color, depth: _, stencil: _ } => {
                // Pour Model2Gpu, nous utilisons begin_frame/end_frame pour gérer le clear
                gpu.begin_frame()?;
                // Note: Le clear est géré automatiquement par begin_frame
                println!("GPU: Clear screen avec couleur [{:.2}, {:.2}, {:.2}, {:.2}]", 
                        color[0], color[1], color[2], color[3]);
            },
            GpuCommand::SetModelMatrix(matrix) => {
                // Convertir le tableau en Mat4 de glam
                let mat = glam::Mat4::from_cols_array(matrix);
                gpu.geometry_processor.set_model_matrix(mat);
                println!("GPU: Set model matrix");
            },
            GpuCommand::SetViewMatrix(matrix) => {
                let mat = glam::Mat4::from_cols_array(matrix);
                gpu.geometry_processor.set_view_matrix(mat);
                println!("GPU: Set view matrix");
            },
            GpuCommand::SetProjectionMatrix(matrix) => {
                let mat = glam::Mat4::from_cols_array(matrix);
                gpu.geometry_processor.set_projection_matrix(mat);
                println!("GPU: Set projection matrix");
            },
            GpuCommand::LoadTexture { id, data, width, height } => {
                gpu.load_texture(*id, data, *width, *height)?;
                println!("GPU: Load texture {} ({}x{})", id, width, height);
            },
            GpuCommand::DrawTriangle { vertices, texture_id } => {
                // Convertir en Triangle3D
                let triangle = self.convert_gpu_vertices_to_triangle(vertices, *texture_id);
                gpu.draw_triangle(&triangle)?;
                println!("GPU: Draw triangle");
            },
            GpuCommand::SetRenderState { state, enabled } => {
                // Convertir RenderStateType en RenderState
                let render_state = match state {
                    crate::memory::RenderStateType::ZBuffer => crate::gpu::RenderState::ZBuffer,
                    crate::memory::RenderStateType::Texturing => crate::gpu::RenderState::Texturing,
                    crate::memory::RenderStateType::Lighting => crate::gpu::RenderState::Lighting,
                    crate::memory::RenderStateType::Transparency => crate::gpu::RenderState::Transparency,
                    _ => crate::gpu::RenderState::ZBuffer, // Défaut
                };
                gpu.set_render_state(render_state, *enabled);
                println!("GPU: Set render state {:?} -> {}", state, enabled);
            },
            _ => {
                println!("GPU: Commande non implémentée: {:?}", command);
            }
        }
        Ok(())
    }
    
    /// Traite un lot de commandes GPU de manière optimisée
    fn process_gpu_command_batch(&mut self, commands: &[GpuCommand], gpu: &mut Model2Gpu) -> Result<()> {
        println!("GPU: Traitement d'un lot de {} commandes", commands.len());
        
        // Traiter les commandes par lot pour de meilleures performances
        for command in commands {
            self.process_gpu_command(command, gpu)?;
        }
        
        Ok(())
    }
    
    /// Convertit des GpuVertex en Triangle3D
    fn convert_gpu_vertices_to_triangle(&self, vertices: &[crate::memory::GpuVertex; 3], texture_id: Option<u32>) -> crate::gpu::geometry::Triangle3D {
        use crate::gpu::geometry::{Triangle3D, Vertex3D, TriangleFlags};
        use glam::Vec3;
        
        let verts = [
            Vertex3D {
                position: Vec3::new(vertices[0].x, vertices[0].y, vertices[0].z),
                normal: Vec3::new(0.0, 0.0, 1.0), // Normale par défaut
                tex_coords: [vertices[0].u, vertices[0].v],
                color: [vertices[0].r, vertices[0].g, vertices[0].b, vertices[0].a],
                fog_coord: 0.0,
                specular: [0.0, 0.0, 0.0],
            },
            Vertex3D {
                position: Vec3::new(vertices[1].x, vertices[1].y, vertices[1].z),
                normal: Vec3::new(0.0, 0.0, 1.0),
                tex_coords: [vertices[1].u, vertices[1].v],
                color: [vertices[1].r, vertices[1].g, vertices[1].b, vertices[1].a],
                fog_coord: 0.0,
                specular: [0.0, 0.0, 0.0],
            },
            Vertex3D {
                position: Vec3::new(vertices[2].x, vertices[2].y, vertices[2].z),
                normal: Vec3::new(0.0, 0.0, 1.0),
                tex_coords: [vertices[2].u, vertices[2].v],
                color: [vertices[2].r, vertices[2].g, vertices[2].b, vertices[2].a],
                fog_coord: 0.0,
                specular: [0.0, 0.0, 0.0],
            },
        ];
        
        Triangle3D {
            vertices: verts,
            texture_id,
            material_id: 0,
            flags: TriangleFlags::default(),
        }
    }
}

impl EmulatorApp {
    pub fn new(rom_path: Option<String>) -> Result<Self> {
        let config = EmulatorConfig::load_or_default("config.toml");
        let memory = Model2Memory::new();
        let mut rom_system = Model2RomSystem::new();

        // Ajouter plusieurs chemins de recherche pour les ROMs
        rom_system.add_search_path("./roms");
        rom_system.add_search_path("../roms");
        rom_system.add_search_path("../../roms");
        rom_system.add_search_path("./roms/model2");
        rom_system.add_search_path("../roms/model2");
        rom_system.add_search_path("../../roms/model2");

        // Charger la ROM si fournie
        if let Some(path) = rom_path {
            println!("Tentative de chargement de la ROM: {}", path);
            // TODO: Charger et intégrer la ROM
        }

        Ok(Self {
            cpu: NecV60::new(),
            memory,
            audio: ScspAudio::new()?,
            input: InputManager::new(),
            config,
            rom_system,
            running: true,
            paused: false,
        })
    }
    
    pub fn run(self) -> Result<()> {
        let event_loop = EventLoop::new()?;
        let window = Arc::new(WindowBuilder::new()
            .with_title("Pixel Model 2 Rust - Émulateur SEGA Model 2")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
            .build(&event_loop)?);
        
        let mut app_state = AppState::new(self);
        
        // Créer le GPU avant la boucle d'événements
        let mut gpu: Option<Model2Gpu> = None;
        {
            let window_ref = window.clone();
            match pollster::block_on(Model2Gpu::new(window_ref)) {
                Ok(g) => {
                    gpu = Some(g);
                    println!("Model2 GPU initialisé avec succès");
                },
                Err(e) => {
                    eprintln!("Erreur d'initialisation GPU: {}", e);
                }
            }
        }
        
        event_loop.run(move |event, elwt| {
            match event {
                Event::WindowEvent { event, .. } => {
                    app_state.handle_window_event(&event);
                    
                    // Gérer les événements GPU
                    if let Some(ref mut gpu) = gpu {
                        match event {
                            WindowEvent::Resized(physical_size) => {
                                // Pour l'instant, garder la résolution standard
                                let _ = gpu.resize(crate::gpu::Model2Resolution::Standard);
                            },
                            WindowEvent::RedrawRequested => {
                                if let Err(e) = gpu.end_frame() {
                                    eprintln!("Erreur GPU end_frame: {}", e);
                                }
                            },
                            _ => {}
                        }
                    }
                    
                    // Quitter si demandé
                    if !app_state.app.running {
                        elwt.exit();
                    }
                },
                Event::AboutToWait => {
                    if let Err(e) = app_state.run_frame(gpu.as_mut()) {
                        eprintln!("Erreur d'émulation: {}", e);
                    }
                    
                    // Redessiner
                    if gpu.is_some() {
                        window.request_redraw();
                    }
                },
                _ => {}
            }
        })?;
        Ok(())
    }
    
    pub fn load_rom(&mut self, game_name: &str) -> Result<()> {
        println!("Chargement du jeu: {}", game_name);
        
        // Charger et mapper le jeu dans la mémoire principale
        self.rom_system.load_and_map_game(game_name, &mut self.memory)?;
        
        // Générer un rapport d'état
        let report = self.rom_system.generate_status_report()?;
        println!("Rapport de chargement ROM:\n{}", report);
        
        // Réinitialiser le CPU après le chargement des ROMs
        self.cpu.reset();
        
        // Initialiser le PC avec l'adresse de reset (typiquement dans la ROM programme)
        // Pour SEGA Model 2, le reset vector est généralement à l'adresse 0x00000004
        if let Ok(reset_vector) = self.memory.read_u32(0x00000004) {
            self.cpu.registers.pc = reset_vector;
            println!("PC initialisé à l'adresse de reset: {:#08X}", reset_vector);
        } else {
            println!("Avertissement: Impossible de lire le vecteur de reset, PC laissé à 0");
        }
        
        println!("Jeu '{}' chargé avec succès!", game_name);
        Ok(())
    }
}