struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec3<f32>,
    @location(2) @interpolate(flat) options: u32,
    @location(3) outline_color: vec3<f32>,
    @location(4) outline_thickness: f32,
    @location(5) glow_color: vec3<f32>,
    @location(6) glow_thickness: f32,
    @location(7) glow_offset: vec2<f32>
};

struct InstanceInput {
    @location(1) size: vec2<f32>,
    @location(2) position: vec2<f32>,
    @location(3) relative_position: vec2<f32>,
    @location(4) texcoords_x: vec2<f32>,
    @location(5) texcoords_y: vec2<f32>,
    @location(6) color: vec3<f32>,
    @location(7) options: u32,
    @location(8) outline_color: vec3<f32>,
    @location(9) outline_thickness: f32,
    @location(10) glow_color: vec3<f32>,
    @location(11) glow_thickness: f32,
    @location(12) glow_offset: vec2<f32>
};

struct AspectRatioUniform {
    aspect_ratio: f32
};

@group(1) @binding(0)
var<uniform> aspect_ratio_uniform: AspectRatioUniform;

struct FontRatioUniform {
    font_ratio: f32
}

@group(2) @binding(0)
var<uniform> font_ratio_uniform: FontRatioUniform;

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
    @builtin(vertex_index) vertex_index : u32
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords.x = instance.texcoords_x[vertex_index / 2];
    out.tex_coords.y = instance.texcoords_y[vertex_index % 2];
    var font_ratio = font_ratio_uniform.font_ratio;
    if (instance.options & 0x1) == 1 {
        font_ratio = 1.0;
    }
    var relative_position = vec2(vertex.position.xy * instance.size * font_ratio + instance.relative_position * font_ratio);
    relative_position.y *= aspect_ratio_uniform.aspect_ratio;
    out.clip_position = vec4(instance.position + relative_position, 0.0, 1.0);
    out.color = instance.color;

    out.outline_color = instance.outline_color;
    out.outline_thickness = instance.outline_thickness;

    out.glow_color = instance.glow_color;
    out.glow_thickness = instance.glow_thickness;
    out.glow_offset = instance.glow_offset;

    return out;
}

@group(0) @binding(0)
var font_atlas_texture: texture_2d<f32>;
@group(0) @binding(1)
var font_atlas_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let outline_enabled = (in.options & 0x2) == 1;
    let outline_color = vec4(in.outline_color, 1.0);
    let min_outline_values = vec2(0.5 - in.outline_thickness, 0.5 - in.outline_thickness + (in.outline_thickness * 0.25));
    let max_outline_values = vec2(0.5 - (in.outline_thickness * 0.25), 0.5);

    let sample = textureSample(font_atlas_texture, font_atlas_sampler, in.tex_coords);
    var base_color = vec4(in.color, 0.0);

    if sample.r < min_outline_values.x && sample.r >= min_outline_values.x - in.glow_thickness {
        base_color = vec4(in.glow_color, sample.r);
    }

    if sample.r < 0.5 && sample.r >= min_outline_values.x {
        base_color = outline_color;
    }

    if sample.r >= 0.5 {
        base_color = vec4(in.color, 1.0);
    }

    return base_color;
}