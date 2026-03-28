struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec3<f32>,
};

struct InstanceInput {
    @location(1) position: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) color: vec3<f32>,
};

struct AspectRatioUniform {
    aspect_ratio: f32
};

@group(1) @binding(0)
var<uniform> aspect_ratio_uniform: AspectRatioUniform;

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
    @builtin(vertex_index) vertex_index : u32
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords.x = 0.0;
    out.tex_coords.y = 0.0;
    var relative_position = vec2(vertex.position.xy * instance.size);
    relative_position.y *= aspect_ratio_uniform.aspect_ratio;
    out.clip_position = vec4(instance.position + relative_position, 0.0, 1.0);
    out.color = instance.color;

    return out;
}

@group(0) @binding(0)
var font_atlas_texture: texture_2d<f32>;
@group(0) @binding(1)
var font_atlas_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(in.color, 1.0);
}