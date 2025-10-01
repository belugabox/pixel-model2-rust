//! Gestion des textures

use anyhow::{Result, anyhow};
use wgpu::*;
use std::collections::HashMap;
use std::sync::Arc;

/// Gestionnaire de textures
pub struct TextureManager {
    textures: HashMap<u32, TextureData>,
    device: Arc<Device>,
    queue: Arc<Queue>,
}

/// Données d'une texture
#[derive(Debug)]
pub struct TextureData {
    pub texture: Texture,
    pub view: TextureView,
    pub bind_group: BindGroup,
    pub width: u32,
    pub height: u32,
}

impl TextureManager {
    pub fn new(device: Arc<Device>, queue: Arc<Queue>) -> Self {
        Self {
            textures: HashMap::new(),
            device,
            queue,
        }
    }
    
    pub fn load_texture(&mut self, id: u32, data: &[u8], width: u32, height: u32) -> Result<()> {
        // Créer la texture wgpu
        let texture = self.device.create_texture(&TextureDescriptor {
            label: Some(&format!("Texture {}", id)),
            size: Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });
        
        // Copier les données
        self.queue.write_texture(
            ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            Extent3d { width, height, depth_or_array_layers: 1 },
        );
        
        let view = texture.create_view(&TextureViewDescriptor::default());
        
        // Créer le bind group (placeholder)
        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            layout: &self.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[],
                label: None,
            }),
            entries: &[],
            label: None,
        });
        
        self.textures.insert(id, TextureData {
            texture,
            view,
            bind_group,
            width,
            height,
        });
        
        Ok(())
    }
    
    pub fn get_texture(&self, id: u32) -> Option<&TextureData> {
        self.textures.get(&id)
    }
}