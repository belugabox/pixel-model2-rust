//! Système de rendu 3D pour l'émulation du GPU SEGA Model 2
//! 
//! Le Model 2 était pionnier dans le rendu 3D temps réel avec :
//! - Triangles texturés
//! - Z-buffering
//! - Éclairage Gouraud
//! - Transparence

pub mod renderer;
pub mod geometry;
pub mod texture;
pub mod shaders;
pub mod framebuffer;

use anyhow::Result;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::window::Window;

pub use renderer::*;
pub use geometry::*;
pub use texture::*;
pub use shaders::*;
pub use framebuffer::*;

/// Résolutions supportées par le Model 2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Model2Resolution {
    /// 496x384 - résolution standard
    Standard,
    /// 640x480 - résolution haute
    High,
}

impl Model2Resolution {
    /// Obtient les dimensions en pixels
    pub fn dimensions(self) -> (u32, u32) {
        match self {
            Model2Resolution::Standard => (496, 384),
            Model2Resolution::High => (640, 480),
        }
    }
    
    /// Obtient le ratio d'aspect
    pub fn aspect_ratio(self) -> f32 {
        let (w, h) = self.dimensions();
        w as f32 / h as f32
    }
}

/// Structure principale du GPU Model 2
pub struct Model2Gpu<'window> {
    /// Rendu moderne utilisant wgpu
    pub renderer: WgpuRenderer<'window>,
    
    /// Géométrie 3D en cours de traitement
    pub geometry_processor: GeometryProcessor,
    
    /// Gestionnaire de textures
    pub texture_manager: TextureManager,
    
    /// Framebuffer virtuel
    pub framebuffer: Framebuffer,
    
    /// Résolution courante
    pub resolution: Model2Resolution,
    
    /// Statistiques de rendu
    pub stats: RenderStats,
    
    /// Configuration de rendu
    pub config: RenderConfig,
}

impl<'window> Model2Gpu<'window> {
    /// Crée une nouvelle instance du GPU Model 2
    pub async fn new(window: &'window winit::window::Window) -> Result<Self> {
        let renderer = WgpuRenderer::new(window).await?;
        let (width, height) = Model2Resolution::Standard.dimensions();
        
        Ok(Self {
            geometry_processor: GeometryProcessor::new(width, height),
            texture_manager: TextureManager::new(renderer.device.clone(), renderer.queue.clone()),
            framebuffer: Framebuffer::new(&renderer.device, width, height),
            renderer,
            resolution: Model2Resolution::Standard,
            stats: RenderStats::new(),
            config: RenderConfig::default(),
        })
    }
    
    /// Redimensionne le GPU pour une nouvelle résolution
    pub fn resize(&mut self, resolution: Model2Resolution) -> Result<()> {
        self.resolution = resolution;
        let (width, height) = resolution.dimensions();
        self.framebuffer.resize(&self.renderer.device, width, height)?;
        self.renderer.resize(winit::dpi::PhysicalSize::new(width, height));
        Ok(())
    }
    
    /// Commence un nouveau frame de rendu
    pub fn begin_frame(&mut self) -> Result<()> {
        self.stats.begin_frame();
        self.framebuffer.clear();
        Ok(())
    }
    
    /// Termine le frame et l'affiche
    pub fn end_frame(&mut self) -> Result<()> {
        // Copier le framebuffer vers la surface
        self.renderer.render()?;
        self.stats.end_frame();
        Ok(())
    }
    
    /// Dessine un triangle 3D
    pub fn draw_triangle(&mut self, triangle: &Triangle3D) -> Result<()> {
        // Transformation et projection
        let transformed = self.geometry_processor.transform_triangle(triangle)?;
        
        // Rendu du triangle
        self.framebuffer.rasterize_triangle(&transformed, &self.texture_manager)?;
        
        self.stats.triangles_drawn += 1;
        Ok(())
    }
    
    /// Charge une texture
    pub fn load_texture(&mut self, id: u32, data: &[u8], width: u32, height: u32) -> Result<()> {
        self.texture_manager.load_texture(id, data, width, height)?;
        Ok(())
    }
    
    /// Met à jour les matrices de transformation
    pub fn set_matrices(&mut self, view: glam::Mat4, projection: glam::Mat4) {
        self.geometry_processor.set_view_matrix(view);
        self.geometry_processor.set_projection_matrix(projection);
    }
    
    /// Active/désactive des fonctionnalités de rendu
    pub fn set_render_state(&mut self, state: RenderState, enabled: bool) {
        match state {
            RenderState::ZBuffer => self.config.z_buffer_enabled = enabled,
            RenderState::Texturing => self.config.texturing_enabled = enabled,
            RenderState::Lighting => self.config.lighting_enabled = enabled,
            RenderState::Transparency => self.config.transparency_enabled = enabled,
        }
    }
    
    /// Obtient les statistiques de rendu
    pub fn get_stats(&self) -> &RenderStats {
        &self.stats
    }
}

/// États de rendu configurables
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderState {
    ZBuffer,
    Texturing,
    Lighting,
    Transparency,
}

/// Configuration de rendu
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Z-buffer activé
    pub z_buffer_enabled: bool,
    
    /// Textures activées
    pub texturing_enabled: bool,
    
    /// Éclairage activé
    pub lighting_enabled: bool,
    
    /// Transparence activée
    pub transparency_enabled: bool,
    
    /// Filtre de texture
    pub texture_filter: TextureFilter,
    
    /// Qualité de rendu
    pub render_quality: RenderQuality,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            z_buffer_enabled: true,
            texturing_enabled: true,
            lighting_enabled: true,
            transparency_enabled: true,
            texture_filter: TextureFilter::Linear,
            render_quality: RenderQuality::High,
        }
    }
}

/// Types de filtrage de texture
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFilter {
    Nearest,
    Linear,
    Bilinear,
}

/// Niveaux de qualité de rendu
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderQuality {
    Low,
    Medium,
    High,
    Ultra,
}

/// Statistiques de rendu pour le débogage et l'optimisation
#[derive(Debug, Clone)]
pub struct RenderStats {
    /// Nombre de frames rendues
    pub frames_rendered: u64,
    
    /// Nombre de triangles dessinés dans le frame courant
    pub triangles_drawn: u32,
    
    /// Nombre de pixels dessinés
    pub pixels_drawn: u64,
    
    /// Temps de rendu du dernier frame (en microsecondes)
    pub last_frame_time_us: u64,
    
    /// FPS moyen
    pub average_fps: f32,
    
    /// Temps de début du frame courant
    frame_start_time: std::time::Instant,
    
    /// Historique des temps de frame
    frame_times: std::collections::VecDeque<u64>,
}

impl RenderStats {
    fn new() -> Self {
        Self {
            frames_rendered: 0,
            triangles_drawn: 0,
            pixels_drawn: 0,
            last_frame_time_us: 0,
            average_fps: 0.0,
            frame_start_time: std::time::Instant::now(),
            frame_times: std::collections::VecDeque::with_capacity(60),
        }
    }
    
    fn begin_frame(&mut self) {
        self.frame_start_time = std::time::Instant::now();
        self.triangles_drawn = 0;
    }
    
    fn end_frame(&mut self) {
        let frame_time = self.frame_start_time.elapsed().as_micros() as u64;
        self.last_frame_time_us = frame_time;
        self.frames_rendered += 1;
        
        // Maintenir un historique des 60 derniers frames
        self.frame_times.push_back(frame_time);
        if self.frame_times.len() > 60 {
            self.frame_times.pop_front();
        }
        
        // Calculer les FPS moyens
        if !self.frame_times.is_empty() {
            let avg_time = self.frame_times.iter().sum::<u64>() / self.frame_times.len() as u64;
            self.average_fps = if avg_time > 0 {
                1_000_000.0 / avg_time as f32
            } else {
                0.0
            };
        }
    }
    
    /// Obtient les FPS instantanés
    pub fn instant_fps(&self) -> f32 {
        if self.last_frame_time_us > 0 {
            1_000_000.0 / self.last_frame_time_us as f32
        } else {
            0.0
        }
    }
    
    /// Obtient le temps de frame en millisecondes
    pub fn frame_time_ms(&self) -> f32 {
        self.last_frame_time_us as f32 / 1000.0
    }
}