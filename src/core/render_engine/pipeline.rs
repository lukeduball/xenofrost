use crate::core::world::{resource::Resource, World, query_resource};

use super::{camera::CameraBindGroupLayout, mesh::{ModelVertex, Vertex}, InstanceRaw, RenderEngine};


#[derive(Resource)]
pub struct Pipeline2D {
    pub pipeline: wgpu::RenderPipeline
}

impl Pipeline2D {
    pub fn new(world: &mut World) -> Self {
        let render_engine = query_resource!(world, RenderEngine).unwrap();
        let camera_bind_group_layout = query_resource!(world, CameraBindGroupLayout).unwrap();

        let render_pipeline_layout = render_engine.data().device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline2D Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout.data().bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let pipeline_2d_shader = render_engine.data().device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Pipeline2D Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/shaders/shader.wgsl")).into())
        });
        
        let pipeline = render_engine.data().device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Pipeline2D"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &pipeline_2d_shader,
                entry_point: "vs_main",
                buffers: &[
                    ModelVertex::desc(),
                    InstanceRaw::desc()
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