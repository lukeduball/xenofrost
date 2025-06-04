struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) line_thickness: f32,
    @location(2) shape_size: vec2<f32>,
};

struct InstanceInput {
    @location(5)  model_matrix_0: vec4<f32>,
    @location(6)  model_matrix_1: vec4<f32>,
    @location(7)  model_matrix_2: vec4<f32>,
    @location(8)  model_matrix_3: vec4<f32>,
    @location(9)  size: vec2<f32>,
    @location(10) line_thickness: f32,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );


    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.tex_coords *= instance.size;
    out.shape_size = instance.size;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    out.line_thickness = instance.line_thickness;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var alpha = 0.0;
    if in.tex_coords.x <= in.line_thickness || in.tex_coords.x >= in.shape_size.x - in.line_thickness {
        alpha = 1.0;
    }
    else if in.tex_coords.y <= in.line_thickness || in.tex_coords.y >= in.shape_size.y - in.line_thickness {
        alpha = 1.0;
    }

    return vec4<f32>(1.0, 0.0, 0.0, alpha);
}