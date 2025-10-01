//! Moteur de rendu moderne utilisant wgpu pour émuler le GPU Model 2

use wgpu::*;
use wgpu::util::DeviceExt;
use winit::window::Window;
use anyhow::{Result, anyhow};
use std::sync::Arc;

/// Vertex simple pour le rendu sans textures
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SimpleVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

impl SimpleVertex {
    pub fn new(x: f32, y: f32, z: f32, r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            position: [x, y, z],
            color: [r, g, b, a],
        }
    }
}

/// Vertex pour le rendu avec textures
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TexturedVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub color: [f32; 4],
}

impl TexturedVertex {
    pub fn new(x: f32, y: f32, z: f32, u: f32, v: f32, r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            position: [x, y, z],
            tex_coords: [u, v],
            color: [r, g, b, a],
        }
    }
}

/// Matrices de transformation 3D
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Matrices {
    pub model: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
}

impl Default for Matrices {
    fn default() -> Self {
        Self {
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            view: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            projection: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

/// Rendu principal utilisant wgpu
pub struct WgpuRenderer {
    /// Instance wgpu
    pub instance: Instance,
    
    /// Fenêtre (pour maintenir la référence)
    pub window: Arc<Window>,
    
    /// Surface de rendu
    pub surface: Surface<'static>,
    
    /// Device wgpu
    pub device: Arc<Device>,
    
    /// Queue de commandes
    pub queue: Arc<Queue>,
    
    /// Configuration de surface
    pub surface_config: SurfaceConfiguration,
    
    /// Shader pour le rendu de triangles simples (sans textures)
    pub triangle_simple_shader: ShaderModule,
    
    /// Pipeline de rendu triangles simples
    pub triangle_simple_pipeline: RenderPipeline,
    
    /// Shader pour le rendu de triangles avec textures
    pub triangle_shader: ShaderModule,
    
    /// Shader pour le blit final
    pub blit_shader: ShaderModule,
    
    /// Pipeline de rendu triangles
    pub triangle_pipeline: RenderPipeline,
    
    /// Pipeline de blit
    pub blit_pipeline: RenderPipeline,
    
    /// Layout des bind groups pour les textures
    pub texture_bind_group_layout: BindGroupLayout,
    
    /// Layout des bind groups pour les matrices
    pub matrix_bind_group_layout: BindGroupLayout,
    
    /// Buffer pour les matrices de transformation
    pub matrix_buffer: Buffer,
    
    /// Bind group pour les matrices
    pub matrix_bind_group: BindGroup,
    
    /// Sampler pour les textures
    pub texture_sampler: Sampler,
}

impl WgpuRenderer {
    /// Crée un nouveau rendu wgpu
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let size = window.inner_size();
        
        // Créer l'instance wgpu
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            dx12_shader_compiler: Dx12Compiler::Fxc,
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });
        
        // Créer la surface - utiliser unsafe pour étendre la lifetime
        let surface = unsafe {
            std::mem::transmute::<Surface<'_>, Surface<'static>>(
                instance.create_surface(&*window)?
            )
        };
        
        // Demander un adaptateur
        let adapter = instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.ok_or_else(|| anyhow!("Impossible de trouver un adaptateur graphique"))?;
        
        // Créer le device et la queue
        let (device, queue) = adapter.request_device(&DeviceDescriptor {
            required_features: Features::empty(),
            required_limits: Limits::default(),
            label: None,
        }, None).await?;
        
        let device = Arc::new(device);
        let queue = Arc::new(queue);
        
        // Configuration de la surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
            
        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        
        surface.configure(&device, &surface_config);
        
        // Créer les shaders
        let triangle_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Triangle Shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/triangle.wgsl").into()),
        });
        
        let triangle_simple_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Triangle Simple Shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/triangle_simple.wgsl").into()),
        });
        
        let blit_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Blit Shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/blit.wgsl").into()),
        });
        
        // Créer le layout pour les textures
        let texture_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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
            label: Some("texture_bind_group_layout"),
        });
        
        // Créer le layout pour les matrices
        let matrix_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("matrix_bind_group_layout"),
        });
        
        // Créer le buffer pour les matrices
        let matrix_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Matrix Buffer"),
            contents: bytemuck::bytes_of(&Matrices::default()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        
        // Créer le bind group pour les matrices
        let matrix_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &matrix_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: matrix_buffer.as_entire_binding(),
                },
            ],
            label: Some("Matrix Bind Group"),
        });
        
        // Créer le sampler
        let texture_sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });
        
        // Créer les pipelines de rendu
        let triangle_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Triangle Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &matrix_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let triangle_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Triangle Pipeline"),
            layout: Some(&triangle_pipeline_layout),
            vertex: VertexState {
                module: &triangle_shader,
                entry_point: "vs_main",
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<TexturedVertex>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[
                        VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: VertexFormat::Float32x3,
                        },
                        VertexAttribute {
                            offset: std::mem::size_of::<[f32; 3]>() as BufferAddress,
                            shader_location: 1,
                            format: VertexFormat::Float32x2,
                        },
                        VertexAttribute {
                            offset: (std::mem::size_of::<[f32; 3]>() + std::mem::size_of::<[f32; 2]>()) as BufferAddress,
                            shader_location: 2,
                            format: VertexFormat::Float32x4,
                        },
                    ],
                }],
            },
            fragment: Some(FragmentState {
                module: &triangle_shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: surface_config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        
        // Pipeline pour triangles simples (sans textures)
        let triangle_simple_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Triangle Simple Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        
        let triangle_simple_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Triangle Simple Pipeline"),
            layout: Some(&triangle_simple_pipeline_layout),
            vertex: VertexState {
                module: &triangle_simple_shader,
                entry_point: "vs_main",
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<SimpleVertex>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[
                        VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: VertexFormat::Float32x3,
                        },
                        VertexAttribute {
                            offset: std::mem::size_of::<[f32; 3]>() as BufferAddress,
                            shader_location: 1,
                            format: VertexFormat::Float32x4,
                        },
                    ],
                }],
            },
            fragment: Some(FragmentState {
                module: &triangle_simple_shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: surface_config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        
        let blit_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Blit Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let blit_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Blit Pipeline"),
            layout: Some(&blit_pipeline_layout),
            vertex: VertexState {
                module: &blit_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &blit_shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: surface_config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        
        Ok(Self {
            instance,
            window,
            surface,
            device,
            queue,
            surface_config,
            triangle_simple_shader,
            triangle_simple_pipeline,
            triangle_shader,
            blit_shader,
            triangle_pipeline,
            blit_pipeline,
            texture_bind_group_layout,
            matrix_bind_group_layout,
            matrix_buffer,
            matrix_bind_group,
            texture_sampler,
        })
    }
    
    /// Redimensionner la surface
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }
    
    /// Rendu d'une frame
    pub fn render(&mut self) -> Result<()> {
        // Obtenir la texture de surface - gérer les erreurs de surface
        let output = match self.surface.get_current_texture() {
            Ok(output) => output,
            Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                // Reconfigurer la surface si elle est perdue ou obsolète
                self.surface.configure(&self.device, &self.surface_config);
                return Ok(());
            }
            Err(e) => return Err(anyhow!("Erreur de surface GPU: {:?}", e)),
        };
        let view = output.texture.create_view(&TextureViewDescriptor::default());
        
        // Créer l'encodeur de commandes
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        // Pass de rendu de base
        {
            let _render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Blit Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }
        
        // Soumettre les commandes
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        Ok(())
    }

    /// Rendre des triangles simples sans textures
    pub fn render_simple_triangles(&mut self, vertices: &[SimpleVertex]) -> Result<()> {
        if vertices.is_empty() || vertices.len() % 3 != 0 {
            return Ok(()); // Rien à rendre ou nombre de sommets invalide
        }

        // Créer un buffer pour les sommets
        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Simple Triangle Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: BufferUsages::VERTEX,
        });

        // Obtenir la texture de surface - gérer les erreurs de surface
        let output = match self.surface.get_current_texture() {
            Ok(output) => output,
            Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                // Reconfigurer la surface si elle est perdue ou obsolète
                self.surface.configure(&self.device, &self.surface_config);
                return Ok(());
            }
            Err(e) => return Err(anyhow!("Erreur de surface GPU: {:?}", e)),
        };
        let view = output.texture.create_view(&TextureViewDescriptor::default());

        // Créer l'encodeur de commandes
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Simple Triangle Render Encoder"),
        });

        // Pass de rendu
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Simple Triangle Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Configurer le pipeline
            render_pass.set_pipeline(&self.triangle_simple_pipeline);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));

            // Dessiner les triangles
            render_pass.draw(0..vertices.len() as u32, 0..1);
        }

        // Soumettre les commandes
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Rendre des triangles texturés
    pub fn render_textured_triangles(&mut self, vertices: &[TexturedVertex], texture_view: &TextureView, bind_group: &BindGroup) -> Result<()> {
        if vertices.is_empty() || vertices.len() % 3 != 0 {
            return Ok(()); // Rien à rendre ou nombre de sommets invalide
        }

        // Créer un buffer pour les sommets
        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Textured Triangle Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: BufferUsages::VERTEX,
        });

        // Obtenir la texture de surface - gérer les erreurs de surface
        let output = match self.surface.get_current_texture() {
            Ok(output) => output,
            Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                // Reconfigurer la surface si elle est perdue ou obsolète
                self.surface.configure(&self.device, &self.surface_config);
                return Ok(());
            }
            Err(e) => return Err(anyhow!("Erreur de surface GPU: {:?}", e)),
        };
        let view = output.texture.create_view(&TextureViewDescriptor::default());

        // Créer l'encodeur de commandes
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Textured Triangle Render Encoder"),
        });

        // Pass de rendu
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Textured Triangle Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Configurer le pipeline et les ressources
            render_pass.set_pipeline(&self.triangle_pipeline);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.set_bind_group(1, &self.matrix_bind_group, &[]);

            // Dessiner les triangles
            render_pass.draw(0..vertices.len() as u32, 0..1);
        }

        // Soumettre les commandes
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Créer un bind group pour une texture
    pub fn create_texture_bind_group(&self, texture_view: &TextureView) -> Result<BindGroup> {
        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&self.texture_sampler),
                },
            ],
            label: Some("Texture Bind Group"),
        });
        Ok(bind_group)
    }
    
    /// Mettre à jour les matrices de transformation
    pub fn update_matrices(&self, matrices: &Matrices) -> Result<()> {
        self.queue.write_buffer(&self.matrix_buffer, 0, bytemuck::bytes_of(matrices));
        Ok(())
    }
    
    /// Définir la matrice modèle
    pub fn set_model_matrix(&self, model: [[f32; 4]; 4]) -> Result<()> {
        let mut matrices = Matrices::default();
        // Lire les matrices actuelles
        // Pour simplifier, on recréé avec les valeurs par défaut et on met à jour seulement model
        matrices.model = model;
        self.update_matrices(&matrices)
    }
    
    /// Définir la matrice de vue
    pub fn set_view_matrix(&self, view: [[f32; 4]; 4]) -> Result<()> {
        let mut matrices = Matrices::default();
        matrices.view = view;
        self.update_matrices(&matrices)
    }
    
    /// Définir la matrice de projection
    pub fn set_projection_matrix(&self, projection: [[f32; 4]; 4]) -> Result<()> {
        let mut matrices = Matrices::default();
        matrices.projection = projection;
        self.update_matrices(&matrices)
    }
}