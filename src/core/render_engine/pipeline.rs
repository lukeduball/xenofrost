use glam::{Mat4, Vec2};

use crate::core::{utilities::include_str_from_project_path, world::{query_resource, resource::Resource, World}};

use super::{camera::CameraBindGroupLayout, mesh::{AtlasVertex, ModelVertex, Vertex}, texture::TextureBindGroupLayout, RenderEngine};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct InstanceTransform {
    pub model: Mat4,
}

impl InstanceTransform {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceTransform>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ]
        }
    }
}

#[derive(Resource)]
pub struct Pipeline2D {
    pub pipeline: wgpu::RenderPipeline
}

impl Pipeline2D {
    pub fn new(world: &mut World) -> Self {
        let render_engine = query_resource!(world, RenderEngine).unwrap();
        let camera_bind_group_layout = query_resource!(world, CameraBindGroupLayout).unwrap_or_else(|| {
            let camera_bind_group_layout_rs = CameraBindGroupLayout::new(&render_engine);
            world.add_resource(camera_bind_group_layout_rs);
            let camera_bind_group_layout_handle = query_resource!(world, CameraBindGroupLayout);
            camera_bind_group_layout_handle.unwrap()
        });
        let texture_bind_group_layout = query_resource!(world, TextureBindGroupLayout).unwrap_or_else(|| {
            let texture_bind_group_layout_rs = TextureBindGroupLayout::new(&render_engine);
            world.add_resource(texture_bind_group_layout_rs);
            let texture_bind_group_layout_handle = query_resource!(world, TextureBindGroupLayout);
            texture_bind_group_layout_handle.unwrap()
        });

        let render_pipeline_layout = render_engine.data().device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline2D Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout.data().bind_group_layout,
                &texture_bind_group_layout.data().bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let pipeline_2d_shader = render_engine.data().device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Pipeline2D Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str_from_project_path!("/res/shaders/model_shader2d.wgsl").into())
        });
        
        let pipeline = render_engine.data().device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Pipeline2D"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &pipeline_2d_shader,
                entry_point: "vs_main",
                buffers: &[
                    ModelVertex::desc(),
                    InstanceTransform::desc()
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &pipeline_2d_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: render_engine.data().config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            pipeline
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct InstanceAtlas {
    pub model: Mat4,
    pub tex_coords: Vec2,
    pub padding: Vec2
}

impl InstanceAtlas {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceAtlas>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x2,
                }
            ]
        }
    }
}

#[derive(Resource)]
pub struct AtlasPipeline2D {
    pub pipeline: wgpu::RenderPipeline
}

impl AtlasPipeline2D {
    pub fn new(world: &mut World) -> Self {
        let render_engine = query_resource!(world, RenderEngine).unwrap();
        let camera_bind_group_layout = query_resource!(world, CameraBindGroupLayout).unwrap_or_else(|| {
            let camera_bind_group_layout_rs = CameraBindGroupLayout::new(&render_engine);
            world.add_resource(camera_bind_group_layout_rs);
            let camera_bind_group_layout_handle = query_resource!(world, CameraBindGroupLayout);
            camera_bind_group_layout_handle.unwrap()
        });
        let texture_bind_group_layout = query_resource!(world, TextureBindGroupLayout).unwrap_or_else(|| {
            let texture_bind_group_layout_rs = TextureBindGroupLayout::new(&render_engine);
            world.add_resource(texture_bind_group_layout_rs);
            let texture_bind_group_layout_handle = query_resource!(world, TextureBindGroupLayout);
            texture_bind_group_layout_handle.unwrap()
        });

        let render_pipeline_layout = render_engine.data().device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("AtlasPipeline2D Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout.data().bind_group_layout,
                &texture_bind_group_layout.data().bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let pipeline_2d_shader = render_engine.data().device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("AtlasPipeline2D Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str_from_project_path!("/res/shaders/atlas_shader2d.wgsl").into())
        });

        let pipeline = render_engine.data().device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("AtlasPipeline2D"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &pipeline_2d_shader,
                entry_point: "vs_main",
                buffers: &[
                    AtlasVertex::desc(),
                    InstanceAtlas::desc()
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &pipeline_2d_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: render_engine.data().config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            pipeline
        }
    }
}