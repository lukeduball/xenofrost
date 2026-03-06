use std::{collections::HashMap, error::Error, fs::File, io::BufReader};

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};
use wgpu::vertex_attr_array;

use crate::{core::render_engine::{mesh::{PositionVertex, Vertex}, pipeline::{PipelineLayoutDescriptor, VertexState, create_default_pipeline2d_descriptor, create_render_pipeline_from_descriptor, create_shader}, texture::Texture}, include_bytes_from_project_path, include_str_from_project_path};

#[derive(Serialize, Deserialize)]
struct AtlasData {
    #[serde(rename = "type")]
    font_type: String,
    size: f32,
    width: u32,
    height: u32,
    #[serde(rename = "yOrigin")]
    y_origin: String
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MetricData {
    em_size: u32,
    line_height: f32,
    ascender: f32,
    descender: f32,
    underline_y: f32,
    underline_thickness: f32
}

#[derive(Serialize, Deserialize)]
struct Bounds {
    left: f32,
    bottom: f32,
    right: f32,
    top: f32
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CharacterData {
    unicode: u32,
    advance: f32,
    plane_bounds: Option<Bounds>,
    atlas_bounds: Option<Bounds>
}

#[derive(Serialize, Deserialize)]
struct FontInformation {
    atlas: AtlasData,
    metrics: MetricData,
    glyphs: Vec<CharacterData>,
    kerning: Vec<String>
}

struct CharacterSpecification {
    advance: f32,
    size: Option<Vec2>,
    position: Vec2,
    texcoords_x: Vec2,
    texcoords_y: Vec2
}

impl CharacterSpecification {
    fn create_char_spec_from_char_info(char_info: CharacterData, atlas_width: u32, atlas_height: u32, factor: f32) -> Self {
        let mut size = None;
        let mut position = Vec2::new(0.0, 0.0);
        if let Some(plane_bounds) = char_info.plane_bounds {
            let width = (plane_bounds.right - plane_bounds.left) * factor;
            let height = (plane_bounds.top - plane_bounds.bottom) * factor;
            size = Some(Vec2::new(width, height));

            let x = width / 2.0 + plane_bounds.left * factor;
            let y = height / 2.0 + plane_bounds.bottom * factor;
            position = Vec2::new(x, y);
        }

        let mut texcoords_x = Vec2::splat(0.0);
        let mut texcoords_y = Vec2::splat(0.0);
        if let Some(atlas_bounds) = char_info.atlas_bounds {
            texcoords_x = Vec2::new(atlas_bounds.left, atlas_bounds.right) / atlas_width as f32;
            texcoords_y = Vec2::new(atlas_height as f32 - atlas_bounds.top, atlas_height as f32 - atlas_bounds.bottom) / atlas_height as f32;
        } 

        Self {
            advance: char_info.advance * factor,
            size,
            position,
            texcoords_x,
            texcoords_y
        }
    }
}

pub struct FontSpecification {
    _atlas: AtlasData,
    _metrics: MetricData,
    glyphs: HashMap<char, CharacterSpecification>,
    _kerning: Vec<String>
}

impl FontSpecification {
    fn create_font_spec_from_font_info(font_info: FontInformation) -> Self {
        let mut glyphs_hash_map = HashMap::new();
        for character_data in font_info.glyphs {
            let character = char::from_u32(character_data.unicode).unwrap();
            //Resize so the base font size of 1 refers to 4px per em
            let factor = 2.0 / font_info.atlas.size;
            let character_spec = CharacterSpecification::create_char_spec_from_char_info(character_data, font_info.atlas.width, font_info.atlas.height, factor);
            glyphs_hash_map.insert(character, character_spec);
        }

        Self {
            _atlas: font_info.atlas,
            _metrics: font_info.metrics,
            glyphs: glyphs_hash_map,
            _kerning: font_info.kerning
        }
    }
}

pub fn load_font_from_file(font_name: &str) -> Result<FontSpecification, Box<dyn Error>> {
    let font_path = format!("res/fonts/{}", font_name);
    let file = File::open(font_path)?;
    let reader = BufReader::new(file);
    let font_info: FontInformation = serde_json::from_reader(reader)?;
    let font_spec = FontSpecification::create_font_spec_from_font_info(font_info);
    Ok(font_spec)
}

pub enum DefaultFonts {
    OpenSans,
    OpenSansSDF,
}

pub fn get_font_from_defaults(font: DefaultFonts, device: &wgpu::Device, queue: &wgpu::Queue) -> (FontSpecification, Texture) {
    match font {
        DefaultFonts::OpenSans => {
            let font_info: FontInformation = serde_json::from_str(include_str_from_project_path!("/res/fonts/opensans.json")).unwrap();
            let font_spec = FontSpecification::create_font_spec_from_font_info(font_info);
            let texture_atlas = Texture::from_bytes(device, queue, include_bytes_from_project_path!("/res/fonts/opensans.png"), "OpenSans Font Atlas Texture");
            (font_spec, texture_atlas)
        },
        DefaultFonts::OpenSansSDF => {
            let font_info: FontInformation = serde_json::from_str(include_str_from_project_path!("/res/fonts/opensans-sdf.json")).unwrap();
            let font_spec = FontSpecification::create_font_spec_from_font_info(font_info);
            let texture_atlas = Texture::from_bytes(device, queue, include_bytes_from_project_path!("/res/fonts/opensans-sdf.png"), "OpenSansSDF Font Atlas Texture");
            (font_spec, texture_atlas)
        }
    }
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct CharacterInstance {
    pub size: Vec2,
    pub position: Vec2,
    pub relative_position: Vec2,
    pub texcoords_x: Vec2,
    pub texcoords_y: Vec2,
    pub color: Vec3,
    pub options: u32
}

impl CharacterInstance {
    const ATTRIBUTES: [wgpu::VertexAttribute; 7] = vertex_attr_array![
        1 => Float32x2,
        2 => Float32x2,
        3 => Float32x2,
        4 => Float32x2,
        5 => Float32x2,
        6 => Float32x3,
        7 => Uint32
    ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES
        }
    }
}

pub fn construct_string_instance_data(text: &str, screen_position: Vec2, font_size: f32, color: Vec3, scale_with_screen: bool, font_specification: &FontSpecification) -> Vec<CharacterInstance> {
    let mut character_instance_list = Vec::new();

    let mut cursor_position = 0.0;
    for character in text.chars() {
        let char_specification_option = font_specification.glyphs.get(&character);
        let char_spec = match char_specification_option {
            Some(char_spec) => char_spec,
            None => font_specification.glyphs.get(&'*').unwrap(),
        };
        if let Some(size) = char_spec.size {
            let mut relative_position = char_spec.position * font_size;
            relative_position.x += cursor_position;
            let char_instance = CharacterInstance { 
                size: size * font_size, 
                position: screen_position,
                relative_position, 
                texcoords_x: char_spec.texcoords_x, 
                texcoords_y: char_spec.texcoords_y,
                color,
                options: scale_with_screen as u32
            };
            character_instance_list.push(char_instance);
        }
        cursor_position += char_spec.advance * font_size;
    }

    character_instance_list
}

pub fn get_font_ratio(width: f32) -> f32 {
    1920.0 / width
}

pub fn create_font_ratio_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let font_ratio_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Font Ratio Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None
            }
        ]
    });

    font_ratio_bind_group_layout
}

pub fn create_bitmap_font_pipeline(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
    aspect_ratio_bind_group_layout: &wgpu::BindGroupLayout,
    font_ratio_bind_group_layout: &wgpu::BindGroupLayout
) -> wgpu::RenderPipeline {
    let pipeline_layout_descriptor = PipelineLayoutDescriptor {
        label: "Bitmap Font Pipeline Layout Descriptor",
        bind_group_layouts: vec![texture_bind_group_layout, aspect_ratio_bind_group_layout, font_ratio_bind_group_layout]
    };
    let shader_module = create_shader(device, "Bitmap Font Shader", include_str_from_project_path!("/res/shaders/bitmap_font.wgsl"));
    let mut pipeline_descriptor = create_default_pipeline2d_descriptor(config, &pipeline_layout_descriptor, &shader_module);
    pipeline_descriptor.label = "Bitmap Font Pipeline";
    pipeline_descriptor.vertex = VertexState { module: &shader_module, entry_point: "vs_main", buffers: vec![PositionVertex::desc(), CharacterInstance::desc()] };

    let pipeline = create_render_pipeline_from_descriptor(device, pipeline_descriptor);
    pipeline
}