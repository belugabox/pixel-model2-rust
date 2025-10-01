//! Système de textures SEGA Model 2
//! 
//! Implémente le chargement et la gestion des textures avec support des formats
//! propriétaires SEGA : 4bpp, 8bpp, 16bpp avec palettes.

use anyhow::Result;
use wgpu::*;
use std::collections::HashMap;
use std::sync::Arc;

/// Gestionnaire de textures avec support des formats SEGA
pub struct TextureManager {
    textures: HashMap<u32, TextureData>,
    palettes: HashMap<u32, PaletteData>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    bind_group_layout: BindGroupLayout,
    sampler: Sampler,
}

/// Données d'une texture
#[derive(Debug)]
pub struct TextureData {
    pub texture: Texture,
    pub view: TextureView,
    pub bind_group: BindGroup,
    pub width: u32,
    pub height: u32,
    pub format: SegaTextureFormat,
    pub palette_id: Option<u32>,
}

/// Formats de texture SEGA Model 2
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SegaTextureFormat {
    /// 4 bits par pixel avec palette 16 couleurs
    Palette4bpp,
    /// 8 bits par pixel avec palette 256 couleurs
    Palette8bpp,
    /// 16 bits par pixel RGB565 direct
    Rgb565,
    /// 16 bits par pixel RGBA4444
    Rgba4444,
    /// 32 bits par pixel RGBA8888 (rare sur Model 2)
    Rgba8888,
}

/// Données de palette pour textures indexées
#[derive(Debug, Clone)]
pub struct PaletteData {
    pub colors: Vec<[u8; 4]>, // RGBA
    pub format: PaletteFormat,
}

/// Format de palette
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaletteFormat {
    /// 16 couleurs pour 4bpp
    Palette16,
    /// 256 couleurs pour 8bpp
    Palette256,
}

/// Texture brute avant conversion GPU
#[derive(Debug, Clone)]
pub struct RawTexture {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: SegaTextureFormat,
    pub palette_id: Option<u32>,
}

/// Paramètres de décodage de texture
#[derive(Debug, Clone)]
pub struct TextureDecodeParams {
    pub width: u32,
    pub height: u32,
    pub format: SegaTextureFormat,
    pub palette_offset: Option<usize>,
    pub data_offset: usize,
    pub stride: Option<u32>, // Pour textures non-power-of-2
}

impl TextureManager {
    pub fn new(device: Arc<Device>, queue: Arc<Queue>) -> Self {
        // Créer le bind group layout pour les textures
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("texture_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        
        // Créer le sampler avec paramètres SEGA Model 2
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            ..Default::default()
        });
        
        Self {
            textures: HashMap::new(),
            palettes: HashMap::new(),
            device,
            queue,
            bind_group_layout,
            sampler,
        }
    }
    
    /// Charge une texture simple (pour compatibilité)
    pub fn load_texture(&mut self, id: u32, data: &[u8], width: u32, height: u32) -> Result<()> {
        // Crée une texture RGBA8 basique depuis les données brutes
        let params = TextureDecodeParams {
            width,
            height,
            format: SegaTextureFormat::Rgba8888,
            palette_offset: None,
            data_offset: 0,
            stride: Some(width * 4),
        };
        
        self.load_texture_from_rom(id, data, params)
    }

    /// Charge une texture depuis des données ROM avec décodage automatique
    pub fn load_texture_from_rom(&mut self, id: u32, rom_data: &[u8], params: TextureDecodeParams) -> Result<()> {
        // Décoder la texture selon le format SEGA
        let raw_texture = self.decode_sega_texture(rom_data, &params)?;
        
        // Convertir en RGBA8 pour wgpu
        let rgba_data = self.convert_to_rgba8(&raw_texture)?;
        
        // Créer la texture wgpu
        let texture = self.device.create_texture(&TextureDescriptor {
            label: Some(&format!("SEGA Texture {}", id)),
            size: Extent3d { 
                width: raw_texture.width, 
                height: raw_texture.height, 
                depth_or_array_layers: 1 
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });
        
        // Copier les données converties
        self.queue.write_texture(
            ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &rgba_data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * raw_texture.width),
                rows_per_image: Some(raw_texture.height),
            },
            Extent3d { 
                width: raw_texture.width, 
                height: raw_texture.height, 
                depth_or_array_layers: 1 
            },
        );
        
        // Créer une vue texture
        let view = texture.create_view(&TextureViewDescriptor::default());
        
        // Créer le bind group avec la vraie layout
        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: Some(&format!("SEGA Texture {} Bind Group", id)),
            layout: &self.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        
        // Stocker la texture décodée avec tous les champs
        self.textures.insert(id, TextureData {
            texture,
            view,
            bind_group,
            width: raw_texture.width,
            height: raw_texture.height,
            format: params.format,
            palette_id: params.palette_offset.map(|offset| offset as u32),
        });
        
        Ok(())
    }
    
    pub fn get_texture(&self, id: u32) -> Option<&TextureData> {
        self.textures.get(&id)
    }

    pub fn get_bind_group(&self, texture_id: u32) -> Option<&BindGroup> {
        self.textures.get(&texture_id).map(|tex| &tex.bind_group)
    }

    /// Décode une texture SEGA depuis les données ROM
    fn decode_sega_texture(&self, rom_data: &[u8], params: &TextureDecodeParams) -> Result<RawTexture> {
        let data_start = params.data_offset;
        let texture_data = &rom_data[data_start..];

        match params.format {
            SegaTextureFormat::Palette4bpp => {
                self.decode_4bpp_indexed(texture_data, params)
            }
            SegaTextureFormat::Palette8bpp => {
                self.decode_8bpp_indexed(texture_data, params)
            }

            SegaTextureFormat::Rgb565 => {
                self.decode_rgb565(texture_data, params)
            }
            SegaTextureFormat::Rgba4444 => {
                self.decode_rgba4444(texture_data, params)
            }
            SegaTextureFormat::Rgba8888 => {
                self.decode_rgba8888(texture_data, params)
            }
        }
    }

    /// Décode texture 4bpp indexée avec palette
    fn decode_4bpp_indexed(&self, data: &[u8], params: &TextureDecodeParams) -> Result<RawTexture> {
        let pixel_count = (params.width * params.height) as usize;
        let mut pixels = Vec::with_capacity(pixel_count);

        for i in 0..(pixel_count + 1) / 2 {
            if i >= data.len() {
                break;
            }
            
            let byte = data[i];
            let pixel1 = (byte & 0x0F) as u8;        // 4 bits inférieurs
            let pixel2 = ((byte & 0xF0) >> 4) as u8; // 4 bits supérieurs
            
            pixels.push(pixel1);
            if pixels.len() < pixel_count {
                pixels.push(pixel2);
            }
        }

        Ok(RawTexture {
            width: params.width,
            height: params.height,
            format: params.format,
            data: pixels,
            palette_id: params.palette_offset.map(|offset| offset as u32),
        })
    }

    /// Décode texture 8bpp indexée avec palette
    fn decode_8bpp_indexed(&self, data: &[u8], params: &TextureDecodeParams) -> Result<RawTexture> {
        let pixel_count = (params.width * params.height) as usize;
        let pixels = data[..pixel_count.min(data.len())].to_vec();

        Ok(RawTexture {
            width: params.width,
            height: params.height,
            format: params.format,
            data: pixels,
            palette_id: params.palette_offset.map(|offset| offset as u32),
        })
    }

    /// Décode texture 16bpp directe
    fn decode_16bpp_direct(&self, data: &[u8], params: &TextureDecodeParams) -> Result<RawTexture> {
        let pixel_count = (params.width * params.height) as usize;
        let mut pixels = Vec::with_capacity(pixel_count * 2);

        for i in 0..pixel_count {
            let byte_idx = i * 2;
            if byte_idx + 1 < data.len() {
                let lo = data[byte_idx];
                let hi = data[byte_idx + 1];
                pixels.push(lo);
                pixels.push(hi);
            }
        }

        Ok(RawTexture {
            width: params.width,
            height: params.height,
            format: params.format,
            data: pixels,
            palette_id: None, // Pas de palette pour les formats directs
        })
    }

    /// Décode texture RGB565
    fn decode_rgb565(&self, data: &[u8], params: &TextureDecodeParams) -> Result<RawTexture> {
        self.decode_16bpp_direct(data, params)
    }

    /// Décode texture RGBA4444
    fn decode_rgba4444(&self, data: &[u8], params: &TextureDecodeParams) -> Result<RawTexture> {
        self.decode_16bpp_direct(data, params)
    }

    /// Décode texture RGBA8888 directe
    fn decode_rgba8888(&self, data: &[u8], params: &TextureDecodeParams) -> Result<RawTexture> {
        let pixel_count = (params.width * params.height) as usize;
        let byte_count = pixel_count * 4; // 4 bytes par pixel RGBA
        let pixels = data[..byte_count.min(data.len())].to_vec();

        Ok(RawTexture {
            width: params.width,
            height: params.height,
            format: params.format,
            data: pixels,
            palette_id: None, // Pas de palette pour les formats directs
        })
    }

    /// Convertit une texture décodée en RGBA8 pour wgpu
    fn convert_to_rgba8(&self, raw_texture: &RawTexture) -> Result<Vec<u8>> {
        let pixel_count = (raw_texture.width * raw_texture.height) as usize;
        let mut rgba_data = Vec::with_capacity(pixel_count * 4);

        match raw_texture.format {
            SegaTextureFormat::Palette4bpp | SegaTextureFormat::Palette8bpp => {
                // Conversion avec palette (pour l'instant, palette par défaut)
                for &index in &raw_texture.data {
                    let color = self.get_palette_color(index, 0); // Palette 0 par défaut
                    rgba_data.extend_from_slice(&color);
                }
            }
            SegaTextureFormat::Rgb565 => {
                // Conversion RGB565 -> RGBA8
                for chunk in raw_texture.data.chunks(2) {
                    if chunk.len() == 2 {
                        let rgb565 = u16::from_le_bytes([chunk[0], chunk[1]]);
                        let r = ((rgb565 >> 11) & 0x1F) as u8;
                        let g = ((rgb565 >> 5) & 0x3F) as u8;
                        let b = (rgb565 & 0x1F) as u8;
                        
                        // Expansion 5/6 bits -> 8 bits
                        rgba_data.push((r << 3) | (r >> 2));
                        rgba_data.push((g << 2) | (g >> 4));
                        rgba_data.push((b << 3) | (b >> 2));
                        rgba_data.push(255); // Alpha opaque
                    }
                }
            }
            SegaTextureFormat::Rgba4444 => {
                // Conversion RGBA4444 -> RGBA8
                for chunk in raw_texture.data.chunks(2) {
                    if chunk.len() == 2 {
                        let rgba4444 = u16::from_le_bytes([chunk[0], chunk[1]]);
                        let r = ((rgba4444 >> 12) & 0x0F) as u8;
                        let g = ((rgba4444 >> 8) & 0x0F) as u8;
                        let b = ((rgba4444 >> 4) & 0x0F) as u8;
                        let a = (rgba4444 & 0x0F) as u8;
                        
                        // Expansion 4 bits -> 8 bits
                        rgba_data.push((r << 4) | r);
                        rgba_data.push((g << 4) | g);
                        rgba_data.push((b << 4) | b);
                        rgba_data.push((a << 4) | a);
                    }
                }
            }
            SegaTextureFormat::Rgba8888 => {
                // Format RGBA8888 direct
                rgba_data.extend_from_slice(&raw_texture.data);
            }
        }

        Ok(rgba_data)
    }

    /// Récupère une couleur de palette (implémentation basique)
    fn get_palette_color(&self, index: u8, _palette_id: u32) -> [u8; 4] {
        // Pour l'instant, palette basique arc-en-ciel
        let normalized_index = (index as f32) / 255.0;
        let hue = normalized_index * 360.0;
        
        // Conversion HSV vers RGB simplifiée
        let c = 1.0; // Saturation maximale
        let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
        
        let (r_prime, g_prime, b_prime) = match hue as i32 / 60 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        
        [
            (r_prime * 255.0) as u8,
            (g_prime * 255.0) as u8,
            (b_prime * 255.0) as u8,
            255, // Alpha opaque
        ]
    }
}