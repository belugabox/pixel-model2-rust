//! Moteur de rendu moderne utilisant wgpu pour émuler le GPU Model 2

use wgpu::*;
use winit::window::Window;
use anyhow::{Result, anyhow};
use std::sync::Arc;

/// Rendu principal utilisant wgpu
pub struct WgpuRenderer<'window> {
    /// Instance wgpu
    pub instance: Instance,
    
    /// Surface de rendu
    pub surface: Surface<'window>,
    
    /// Device wgpu
    pub device: Arc<Device>,
    
    /// Queue de commandes
    pub queue: Arc<Queue>,
    
    /// Configuration de surface
    pub surface_config: SurfaceConfiguration,
    
    /// Shader pour le rendu de triangles 3D
    pub triangle_shader: ShaderModule,
    
    /// Shader pour le blit final
    pub blit_shader: ShaderModule,
    
    /// Pipeline de rendu triangles
    pub triangle_pipeline: RenderPipeline,
    
    /// Pipeline de blit
    pub blit_pipeline: RenderPipeline,
    
    /// Layout des bind groups pour les textures
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
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let triangle_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Triangle Pipeline"),
            layout: Some(&triangle_pipeline_layout),
            vertex: VertexState {
                module: &triangle_shader,
                entry_point: "vs_main",
                buffers: &[],
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
            surface,
            device,
            queue,
            surface_config,
            triangle_shader,
            blit_shader,
            triangle_pipeline,
            blit_pipeline,
            texture_bind_group_layout,
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
    pub fn render(&self) -> Result<()> {
        // Obtenir la texture de surface
        let output = self.surface.get_current_texture()?;
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
}