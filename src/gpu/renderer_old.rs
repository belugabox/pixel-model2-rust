//! Rendu moderne    /// Device wgpu
    pub device: Arc<Device>,
    
    /// Queue de commandes
    pub queue: Arc<Queue>,isant wgpu pour émuler le GPU Model 2

use anyhow::{Result, anyhow};
use wgpu::*;
use winit::window::Window;

/// Rendu principal utilisant wgpu
pub struct WgpuRenderer<'window> {
    /// Instance wgpu
    pub instance: Instance,
    
    /// Surface de rendu
    pub surface: Surface<'window>,
    
    /// Adaptateur graphique
    pub adapter: Adapter,
    
    /// Device logique
    pub device: Device,
    
    /// Queue de commandes
    pub queue: Queue,
    
    /// Configuration de surface
    pub surface_config: SurfaceConfiguration,
    
    /// Pipeline de rendu pour les triangles texturés
    pub triangle_pipeline: RenderPipeline,
    
    /// Pipeline pour la copie de framebuffer
    pub blit_pipeline: RenderPipeline,
    
    /// Buffer de vertices
    pub vertex_buffer: Buffer,
    
    /// Buffer d'indices
    pub index_buffer: Buffer,
    
    /// Bind group layout pour les textures
    pub texture_bind_group_layout: BindGroupLayout,
    
    /// Sampler pour les textures
    pub texture_sampler: Sampler,
}

impl<'window> WgpuRenderer<'window> {
    /// Crée un nouveau rendu wgpu
    pub async fn new(window: &'window Window) -> Result<Self> {
        let size = window.inner_size();
        
        // Créer l'instance wgpu
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            dx12_shader_compiler: Dx12Compiler::Fxc,
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });
        
        // Créer la surface
        let surface = instance.create_surface(window)?;
        
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
        
        let blit_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Blit Shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/blit.wgsl").into()),
        });
        
        // Layout pour les textures
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
            label: Some("Texture Bind Group Layout"),
        });
        
        // Pipeline de rendu pour les triangles
        let triangle_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Triangle Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let triangle_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Triangle Pipeline"),
            layout: Some(&triangle_pipeline_layout),
            vertex: VertexState {
                module: &triangle_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(FragmentState {
                module: &triangle_shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: surface_config.format,
                    blend: Some(BlendState::ALPHA_BLENDING),
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
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        
        // Pipeline pour la copie de framebuffer
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
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
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
        
        // Créer les buffers
        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: 65536, // 64KB pour les vertices
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let index_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Index Buffer"),
            size: 32768, // 32KB pour les indices
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Sampler pour les textures
        let texture_sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });
        
        Ok(Self {
            instance,
            surface,
            adapter,
            device,
            queue,
            surface_config,
            triangle_pipeline,
            blit_pipeline,
            vertex_buffer,
            index_buffer,
            texture_bind_group_layout,
            texture_sampler,
        })
    }
    
    /// Redimensionne le rendu
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        if width > 0 && height > 0 {
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
        }
        Ok(())
    }
    
    /// Présente un framebuffer sur la surface
    pub fn present_framebuffer(&self, framebuffer: &super::Framebuffer) -> Result<()> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        // Créer un bind group pour la texture du framebuffer
        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&framebuffer.color_texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&self.texture_sampler),
                },
            ],
            label: Some("Framebuffer Bind Group"),
        });
        
        {
            let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
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
            
            let mut render_pass = render_pass;
            render_pass.set_pipeline(&self.blit_pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..4, 0..1); // Quad plein écran
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        Ok(())
    }
}

/// Structure représentant un vertex 3D
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
    const ATTRIBS: [VertexAttribute; 3] = vertex_attr_array![
        0 => Float32x3,  // position
        1 => Float32x2,  // tex_coords
        2 => Float32x4,  // color
    ];
    
    pub fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}