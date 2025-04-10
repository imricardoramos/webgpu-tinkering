use std::{borrow::Cow, collections::HashMap, num::NonZero};

use bytemuck::{Pod, Zeroable};
use eframe::wgpu::{self, util::DeviceExt};
use image::{EncodableLayout, RgbaImage};
use indexmap::IndexMap;
use itertools::Itertools;
use wgpu::TextureUsages;

use crate::{
    models::{Camera, Model},
    winit_app::Scene,
};

pub async fn init() -> (wgpu::Instance, wgpu::Adapter, wgpu::Device, wgpu::Queue) {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..Default::default()
    });
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            ..Default::default()
        })
        .await
        .unwrap();
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::TEXTURE_BINDING_ARRAY
                    | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING,
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();
    dbg!(&device.features());
    (instance, adapter, device, queue)
}
pub struct Renderer<'a> {
    device: Cow<'a, wgpu::Device>,
    queue: Cow<'a, wgpu::Queue>,
    surface: Option<wgpu::Surface<'a>>,
    bind_group_layout: wgpu::BindGroupLayout,
    pub render_pipeline: wgpu::RenderPipeline,
}
impl<'a> Renderer<'a> {
    pub fn new(
        device: Cow<'a, wgpu::Device>,
        queue: Cow<'a, wgpu::Queue>,
        textures_count: usize,
    ) -> Self {
        let shader_module = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bind group layout"),
            entries: &[
                // Storage Buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Texture Array
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: Some(NonZero::new(textures_count as u32).unwrap()),
                },
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Uniform Buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("pipeline layout descriptor"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render pipeline descriptor"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<VertexData>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        // Position
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        // Normal
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: std::mem::size_of::<[f32; 3]>() as u64,
                            shader_location: 1,
                        },
                        // UV
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: std::mem::size_of::<[f32; 3 + 3]>() as u64,
                            shader_location: 2,
                        },
                        // Model Index
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32,
                            offset: std::mem::size_of::<[f32; 3 + 3 + 2]>() as u64,
                            shader_location: 3,
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                //cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            device,
            queue,
            surface: None,
            bind_group_layout,
            render_pipeline,
        }
    }
    pub fn add_surface(&mut self, size: [u32; 2], surface: wgpu::Surface<'a>) {
        surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                width: size[0],
                height: size[1],
                present_mode: wgpu::PresentMode::Fifo,
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
            },
        );
        self.surface = Some(surface);
    }

    pub fn create_resources(
        &self,
        surface_size: [u32; 2],
        camera: &Camera,
        textures_map: &IndexMap<String, RgbaImage>,
        models: &[Model],
    ) -> (
        Vec<Vec<wgpu::Buffer>>,
        Vec<Vec<wgpu::Buffer>>,
        wgpu::BindGroup,
        wgpu::TextureView,
    ) {
        let device = &self.device;
        let mut vertex_buffers = vec![];
        let mut index_buffers = vec![];
        let mut tms_flat = vec![];
        let mut texture_views = vec![];

        for (i, model) in models.iter().enumerate() {
            let model_vertex_buffers = model
                .vertex_data(i)
                .iter()
                .map(|mesh_vertex_data| {
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("buffer init descriptor (vertex)"),
                        contents: bytemuck::cast_slice(mesh_vertex_data.as_slice()),
                        usage: wgpu::BufferUsages::VERTEX,
                    })
                })
                .collect_vec();
            let model_index_buffers = model
                .meshes
                .iter()
                .map(|mesh| {
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("buffer init descriptor (indices)"),
                        contents: bytemuck::cast_slice(mesh.indices.as_slice()),
                        usage: wgpu::BufferUsages::INDEX,
                    })
                })
                .collect_vec();

            vertex_buffers.push(model_vertex_buffers);
            index_buffers.push(model_index_buffers);
            tms_flat.extend_from_slice(model.tm().as_slice())
        }
        for image in textures_map.values() {
            let texture_size = wgpu::Extent3d {
                width: image.width(),
                height: image.height(),
                depth_or_array_layers: 1,
            };
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("texture descriptor (texture)"),
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfoBase {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                image.as_bytes(),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * image.width()),
                    rows_per_image: Some(image.height()),
                },
                texture_size,
            );
            texture_views.push(texture.create_view(&wgpu::TextureViewDescriptor::default()));
        }
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        //dbg!(&tms_flat);
        let storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("buffer init descriptor (storage)"),
            contents: bytemuck::cast_slice(&tms_flat),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("buffer init descriptor (uniform)"),
            contents: bytemuck::cast_slice(camera.tm().as_slice()),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture descriptor (depth)"),
            size: wgpu::Extent3d {
                width: surface_size[0],
                height: surface_size[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind group descriptor"),
            layout: &self.bind_group_layout,
            entries: &[
                // Storage Buffer
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &storage_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                // Texture Array
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureViewArray(
                        &texture_views.iter().collect_vec(),
                    ),
                },
                // Sampler
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                // Uniform Buffer
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &uniform_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        (
            vertex_buffers,
            index_buffers,
            bind_group,
            depth_texture_view,
        )
    }
    pub fn render(&self, surface_size: [u32; 2], scene: &Scene) -> Result<(), wgpu::SurfaceError> {
        if let Some(surface) = &self.surface {
            let (vertex_buffers, index_buffers, bind_group, depth_texture_view) = self
                .create_resources(
                    surface_size,
                    &scene.camera,
                    &scene.textures_map,
                    &scene.models,
                );
            let output_texture = surface.get_current_texture()?;
            let view = output_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &depth_texture_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, Some(&bind_group), &[]);
                for model_idx in 0..scene.models.len() {
                    for mesh_idx in 0..scene.models[model_idx].meshes.len() {
                        let vertex_buffer = &vertex_buffers[model_idx][mesh_idx];
                        let index_buffer = &index_buffers[model_idx][mesh_idx];
                        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                        render_pass
                            .set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(
                            0..scene.models[model_idx].meshes[mesh_idx].indices.len() as u32,
                            0,
                            0..1,
                        );
                    }
                }
            }
            self.queue.submit(std::iter::once(encoder.finish()));
            output_texture.present();
        };
        Ok(())
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct VertexData {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub model_idx: u32,
}
