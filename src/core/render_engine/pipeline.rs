use std::num::NonZero;

use glam::{Mat4, Vec2};
use wgpu::{BindGroupLayout, ShaderModuleDescriptor};

use crate::core::{render_engine::{mesh::{Mesh, PositionVertex}}, utilities::include_str_from_project_path};

use super::mesh::{ModelVertex, Vertex};

pub struct PipelineLayoutDescriptor<'a> {
    label: &'a str,
    bind_group_layouts: Vec<&'a BindGroupLayout>,
}

impl<'a> PipelineLayoutDescriptor<'a> {
    fn create_pipeline_layout_descriptor_object(&self, device: &wgpu::Device) -> wgpu::PipelineLayout {
        let pipeline_layout_descriptor = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(self.label),
            bind_group_layouts: &self.bind_group_layouts,
            push_constant_ranges: &[]
        });

        pipeline_layout_descriptor
    } 
}

struct VertexState<'a> {
    module: &'a wgpu::ShaderModule,
    entry_point: &'a str,
    buffers: Vec<wgpu::VertexBufferLayout<'a>>
}

struct FragmentState<'a> {
    module: &'a wgpu::ShaderModule,
    entry_point: &'a str,
    targets: Vec<Option<wgpu::ColorTargetState>>,
}

pub struct RenderPipelineDescriptor<'a> {
    label: &'a str,
    layout: &'a PipelineLayoutDescriptor<'a>,
    vertex: VertexState<'a>,
    fragment: FragmentState<'a>,
    primitive: wgpu::PrimitiveState,
    depth_stencil: Option<wgpu::DepthStencilState>,
    multisample: wgpu::MultisampleState,
    multiview: Option<NonZero<u32>>,
}

pub fn create_shader(device: &wgpu::Device, label: &str, shader_path: &str) -> wgpu::ShaderModule {
    device.create_shader_module(ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(shader_path.into())
    })
}

pub fn create_default_pipeline2d_bind_group_layout<'a>(
    camera_bind_group_layout: &'a BindGroupLayout, 
    texture_bind_group_layout: &'a BindGroupLayout
) -> PipelineLayoutDescriptor<'a> {
    let pipeline_layout_descriptor = PipelineLayoutDescriptor {
        label: "Default Pipeline2d Layout",
        bind_group_layouts: vec![
                camera_bind_group_layout,
                texture_bind_group_layout
            ]
    };

    pipeline_layout_descriptor
}

pub fn create_default_pipeline2d_descriptor<'a>(
    config: &wgpu::SurfaceConfiguration, 
    pipeline_layout_descriptor: &'a PipelineLayoutDescriptor, 
    shader_module: &'a wgpu::ShaderModule
) -> RenderPipelineDescriptor<'a> {
    let descriptor = RenderPipelineDescriptor {
        label: "Default 2d Pipeline",
        layout: pipeline_layout_descriptor,
        vertex: VertexState { 
            module: shader_module, 
            entry_point: "vs_main", 
            buffers: vec![ModelVertex::desc(), InstanceTransform::desc()] 
        },
        fragment: FragmentState { 
            module: shader_module, 
            entry_point: "fs_main", 
            targets: vec![Some(wgpu::ColorTargetState { 
                format: config.format, 
                blend: Some(wgpu::BlendState::ALPHA_BLENDING), 
                write_mask: wgpu::ColorWrites::ALL 
            })] 
        },
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
    };

    descriptor
}

fn create_render_pipeline_from_descriptor(device: &wgpu::Device, descriptor: RenderPipelineDescriptor) -> wgpu::RenderPipeline {
    let render_pipeline_layout = descriptor.layout.create_pipeline_layout_descriptor_object(device);
    
    let render_pipeline_descriptor = wgpu::RenderPipelineDescriptor {
        label: Some(descriptor.label),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: descriptor.vertex.module,
            entry_point: descriptor.vertex.entry_point,
            buffers: &descriptor.vertex.buffers,
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: descriptor.fragment.module,
            entry_point: descriptor.fragment.entry_point,
            targets: &descriptor.fragment.targets,
            compilation_options: Default::default()
        }),
        primitive: descriptor.primitive,
        depth_stencil: descriptor.depth_stencil,
        multisample: descriptor.multisample,
        multiview: descriptor.multiview,
        cache: None,
    };

    let pipeline = device.create_render_pipeline(&render_pipeline_descriptor);

    pipeline
}   

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

pub fn create_default_pipeline2d(
    device: &wgpu::Device, 
    config: &wgpu::SurfaceConfiguration, 
    camera_bind_group_layout: &BindGroupLayout, 
    texture_bind_group_layout: &BindGroupLayout
) -> wgpu::RenderPipeline {
    let pipeline_2d_layout = create_default_pipeline2d_bind_group_layout(camera_bind_group_layout, texture_bind_group_layout);
    let shader_module = create_shader(device, "Default Pipeline 2d Shader", include_str_from_project_path!("/res/shaders/model_shader2d.wgsl"));
    let pipeline_2d_descriptor = create_default_pipeline2d_descriptor(config, &pipeline_2d_layout, &shader_module);

    create_render_pipeline_from_descriptor(device, pipeline_2d_descriptor)
}

pub fn create_debug_lines_pipeline2d(
    device: &wgpu::Device, 
    config: &wgpu::SurfaceConfiguration, 
    camera_bind_group_layout: &BindGroupLayout
) -> wgpu::RenderPipeline {
    let pipeline_layout_descriptor = PipelineLayoutDescriptor {
        label: "Debug Lines 2d Pipeline Layout",
        bind_group_layouts: vec![camera_bind_group_layout]
    };
    let shader_module = create_shader(device, "Debug Lines 2d Shader", include_str_from_project_path!("/res/shaders/debug_lines_shader2d.wgsl"));
    let mut pipeline_descriptor = create_default_pipeline2d_descriptor(config, &pipeline_layout_descriptor, &shader_module);
    pipeline_descriptor.label = "Debug Lines Pipeline2d";
    pipeline_descriptor.vertex = VertexState { module: &shader_module, entry_point: "vs_main", buffers: vec![PositionVertex::desc()] };
    pipeline_descriptor.primitive.topology = wgpu::PrimitiveTopology::LineStrip;

    let pipeline = create_render_pipeline_from_descriptor(device, pipeline_descriptor);

    pipeline   
}

pub fn create_atlas_pipeline2d(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    camera_bind_group_layout: &BindGroupLayout,
    texture_bind_group_layout: &BindGroupLayout
) -> wgpu::RenderPipeline {
    let pipeline_layout_descriptor = create_default_pipeline2d_bind_group_layout(camera_bind_group_layout, texture_bind_group_layout);
    let shader_module = create_shader(device, "Atlas 2d Shader", include_str_from_project_path!("/res/shaders/atlas_shader2d.wgsl"));
    let mut pipeline_descriptor = create_default_pipeline2d_descriptor(config, &pipeline_layout_descriptor, &shader_module);
    pipeline_descriptor.label = "Atlas Pipeline 2d";
    pipeline_descriptor.vertex = VertexState { module: &shader_module, entry_point: "vs_main", buffers: vec![PositionVertex::desc(), InstanceAtlas::desc()] };

    let pipeline = create_render_pipeline_from_descriptor(device, pipeline_descriptor);

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

pub fn create_color_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let color_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Color Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None
            }
        ]
    });

    color_bind_group_layout
}

pub struct DebugLineInstance {
    pub mesh: Mesh,
    pub color_uniform: wgpu::Buffer,
    pub color_bind_group: wgpu::BindGroup
}