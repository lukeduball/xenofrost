use glam::{Mat4, Vec2};

use crate::core::{render_engine::mesh::{Mesh, PositionVertex}, utilities::include_str_from_project_path, world::{query_resource, resource::{Resource, ResourceHandle}, World}};

use super::{camera::CameraBindGroupLayout, mesh::{ModelVertex, Vertex}, texture::TextureBindGroupLayout, RenderEngine};

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
pub struct Pipeline2d {
    pub pipeline: wgpu::RenderPipeline
}

impl Pipeline2d {
    pub fn new(world: &mut World) -> Self {
        let pipeline = create_default_pipeline2d(world, "Pipeline2d", include_str_from_project_path!("/res/shaders/model_shader2d.wgsl"));

        Self {
            pipeline
        }
    }
}

fn create_default_pipeline2d(world: &mut World, label: &str, shader: &str) -> wgpu::RenderPipeline {
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
        label: Some(format!("{} Layout", label).as_str()),
        bind_group_layouts: &[
            &camera_bind_group_layout.data().bind_group_layout,
            &texture_bind_group_layout.data().bind_group_layout,
        ],
        push_constant_ranges: &[],
    });

    let pipeline_2d_shader = render_engine.data().device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(format!("{} Shader", label).as_str()),
        source: wgpu::ShaderSource::Wgsl(shader.into())
    });
    
    let pipeline = render_engine.data().device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(format!("{}", label).as_str()),
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

    pipeline
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct InstanceDebugShape {
    pub model: Mat4,
    pub shape_size: Vec2,
    pub line_thickness: f32,
    _padding: f32,
}

impl InstanceDebugShape {
    pub fn new(model: Mat4, shape_size: Vec2, line_thickness: f32) -> Self {
        Self {
            model,
            shape_size,
            line_thickness,
            _padding: 0.0,
        }
    }

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
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 18]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32
                }
            ]
        }
    }
}


#[derive(Resource)]
pub struct DebugBordersPipeline2d {
    pub pipeline: wgpu::RenderPipeline
}

impl DebugBordersPipeline2d {
    pub fn new(world: &mut World) -> Self {
        let pipeline = create_debug_shape_pipeline2d(world, "DebugBordersPipeline2d", include_str_from_project_path!("/res/shaders/debug_shapes_shader2d.wgsl"));
    
        Self {
            pipeline
        }
    }
}

fn create_debug_shape_pipeline2d(world: &mut World, label: &str, shader: &str) -> wgpu::RenderPipeline {
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
        label: Some(format!("{} Layout", label).as_str()),
        bind_group_layouts: &[
            &camera_bind_group_layout.data().bind_group_layout,
            &texture_bind_group_layout.data().bind_group_layout,
        ],
        push_constant_ranges: &[],
    });

    let pipeline_2d_shader = render_engine.data().device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(format!("{} Shader", label).as_str()),
        source: wgpu::ShaderSource::Wgsl(shader.into())
    });
    
    let pipeline = render_engine.data().device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(format!("{}", label).as_str()),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &pipeline_2d_shader,
            entry_point: "vs_main",
            buffers: &[
                ModelVertex::desc(),
                InstanceDebugShape::desc()
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

    pipeline
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct InstanceAtlas {
    pub model: Mat4,
    pub tex_coords: Vec2,
    pub sprite_size: Vec2
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
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 18]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x2
                }
            ]
        }
    }
}

#[derive(Resource)]
pub struct AtlasPipeline2d {
    pub pipeline: wgpu::RenderPipeline
}

impl AtlasPipeline2d {
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
            label: Some("AtlasPipeline2d Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout.data().bind_group_layout,
                &texture_bind_group_layout.data().bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let pipeline_2d_shader = render_engine.data().device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("AtlasPipeline2d Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str_from_project_path!("/res/shaders/atlas_shader2d.wgsl").into())
        });

        let pipeline = render_engine.data().device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("AtlasPipeline2d"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &pipeline_2d_shader,
                entry_point: "vs_main",
                buffers: &[
                    PositionVertex::desc(),
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

#[derive(Resource)]
pub struct ColorBindGroupLayout {
    pub bind_group_layout: wgpu::BindGroupLayout
}

impl ColorBindGroupLayout {
    pub fn new(render_engine: &ResourceHandle<RenderEngine>) -> Self {
        Self {
            bind_group_layout: render_engine.data().device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ColorBindGroupLayout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer { 
                            ty: wgpu::BufferBindingType::Uniform, 
                            has_dynamic_offset: false, 
                            min_binding_size: None, 
                        },
                        count: None
                    }
                ],
            })
        }
    }
}

pub struct DebugLineInstance {
    pub mesh: Mesh,
    pub color_uniform: wgpu::Buffer,
    pub color_bind_group: wgpu::BindGroup
}

#[derive(Resource)]
pub struct DebugLinesPipeline2d {
    pub pipeline: wgpu::RenderPipeline
}

impl DebugLinesPipeline2d {
    pub fn new(world: &mut World) -> Self {
        let pipeline = create_debug_lines_pipeline2d(world, "Debug Lines", include_str_from_project_path!("/res/shaders/debug_lines_shader2d.wgsl"));

        Self {
            pipeline
        }
    }
}

fn create_debug_lines_pipeline2d(world: &mut World, label: &str, shader: &str) -> wgpu::RenderPipeline {
    let render_engine = query_resource!(world, RenderEngine).unwrap();
    let camera_bind_group_layout = query_resource!(world, CameraBindGroupLayout).unwrap_or_else(|| {
        let camera_bind_group_layout_rs = CameraBindGroupLayout::new(&render_engine);
        world.add_resource(camera_bind_group_layout_rs);
        let camera_bind_group_layout_handle = query_resource!(world, CameraBindGroupLayout);
        camera_bind_group_layout_handle.unwrap()
    });
    let color_bind_group_layout = query_resource!(world, ColorBindGroupLayout).unwrap_or_else(|| {
        let color_bind_group_layout_rs = ColorBindGroupLayout::new(&render_engine);
        world.add_resource(color_bind_group_layout_rs);
        let color_bind_group_layout_handle = query_resource!(world, ColorBindGroupLayout);
        color_bind_group_layout_handle.unwrap()
    });

    let render_pipeline_layout = render_engine.data().device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(format!("{} Layout", label).as_str()),
        bind_group_layouts: &[
            &camera_bind_group_layout.data().bind_group_layout,
            &color_bind_group_layout.data().bind_group_layout
        ],
        push_constant_ranges: &[],
    });

    let pipeline_2d_shader = render_engine.data().device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(format!("{} Shader", label).as_str()),
        source: wgpu::ShaderSource::Wgsl(shader.into())
    });
    
    let pipeline = render_engine.data().device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(format!("{}", label).as_str()),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &pipeline_2d_shader,
            entry_point: "vs_main",
            buffers: &[
                PositionVertex::desc(),
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
            topology: wgpu::PrimitiveTopology::LineStrip,
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

    pipeline
}