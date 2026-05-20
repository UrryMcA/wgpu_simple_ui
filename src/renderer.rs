use std::collections::HashMap;
use wgpu::{Device, Queue, TextureFormat, TextureView, CommandEncoder, RenderPass, RenderPipeline, BindGroup, BindGroupLayout};
use wgpu::util::DeviceExt;

use crate::common::vertex::{Vertex, DrawCommand};
use crate::common::primitives::{Primitives};
use crate::texture_manager::TextureManager;
use crate::loader::{TextureLoader, FontLoader};
use crate::gpu_bitmap_font::{RawGlyph, GpuBitmapFont};
use crate::ui::UiManager;

pub struct UiRenderer {
    device: Device,
    queue: Queue,
    pipeline: RenderPipeline,
    uniform_bind_group: BindGroup,
    uniform_buffer: wgpu::Buffer,
    texture_bind_group_layout: BindGroupLayout,
    texture_manager: TextureManager,
    ui_manager: UiManager,
    commands: Vec<DrawCommand>,
    vertex_buffer: wgpu::Buffer,
    vertex_buffer_capacity: usize,
    surface_width: u32,
    surface_height: u32,
}

impl UiRenderer {
    const MAX_VERTICES: usize = 100_000;

    pub fn new(device: &Device, queue: &Queue, surface_format: TextureFormat, width: u32, height: u32, primitives: Box<dyn Primitives + Send + Sync>) -> Self {
        // Texture bind group layout
        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ui_texture_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Uniforms
        let proj = Self::ortho_projection(width as f32, height as f32);
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ui_uniforms"),
            contents: bytemuck::cast_slice(&[Uniforms { proj }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ui_uniform_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ui_uniform_bg"),
            layout: &uniform_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ui_pipeline_layout"),
            bind_group_layouts: &[&uniform_layout, &texture_bind_group_layout],
            immediate_size: 0,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ui_vert"),
            source: wgpu::ShaderSource::Wgsl(VERTEX_SHADER.into()),
        });
        let fragment = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ui_frag"),
            source: wgpu::ShaderSource::Wgsl(FRAGMENT_SHADER.into()),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ui_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ui_vertex_buffer"),
            size: (Self::MAX_VERTICES * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let texture_manager = TextureManager::new(device, &texture_bind_group_layout);
        let ui_manager = UiManager::new(primitives);

        Self {
            device: device.clone(),
            queue: queue.clone(),
            pipeline,
            uniform_bind_group,
            uniform_buffer,
            texture_bind_group_layout,
            texture_manager,
            ui_manager,
            commands: Vec::new(),
            vertex_buffer,
            vertex_buffer_capacity: Self::MAX_VERTICES,
            surface_width: width,
            surface_height: height,
        }
    }

    fn ortho_projection(width: f32, height: f32) -> [[f32; 4]; 4] {
        let left = 0.0;
        let right = width;
        let bottom = height;
        let top = 0.0;
        let near = -1.0;
        let far = 1.0;
        [
            [2.0 / (right - left), 0.0, 0.0, 0.0],
            [0.0, 2.0 / (top - bottom), 0.0, 0.0],
            [0.0, 0.0, -2.0 / (far - near), 0.0],
            [
                -(right + left) / (right - left),
                -(top + bottom) / (top - bottom),
                -(far + near) / (far - near),
                1.0,
            ],
        ]
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_width = width;
        self.surface_height = height;
        let proj = Self::ortho_projection(width as f32, height as f32);
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[Uniforms { proj }]));
        self.ui_manager.layout(crate::ui::Size::new(width as f32, height as f32));
    }

    pub fn load_texture(&mut self, name: &str, loader: &dyn TextureLoader) -> Option<u64> {
        let (rgba, w, h) = loader.load_texture_rgba(name)?;
        Some(self.texture_manager.load_from_rgba(&self.device, &self.queue, &rgba, w, h, name))
    }

    pub fn load_font(&mut self, name: &str, loader: &dyn FontLoader) -> bool {
        let (rgba, atlas_w, atlas_h, raw_glyphs) = match loader.load_font_data(name) {
            Some(data) => data,
            None => return false,
        };
        let texture_id = self.texture_manager.load_from_rgba(
            &self.device, &self.queue, &rgba, atlas_w, atlas_h,
            &format!("font_{}", name)
        );
        let mut chars = std::collections::HashMap::new();
        for raw in raw_glyphs {
            let info = crate::common::GlyphInfo {
                width: raw.width as f32,
                height: raw.height as f32,
                u0: raw.x as f32 / atlas_w as f32,
                v0: raw.y as f32 / atlas_h as f32,
                u1: (raw.x + raw.width) as f32 / atlas_w as f32,
                v1: (raw.y + raw.height) as f32 / atlas_h as f32,
                xoffset: raw.xoffset as f32,
                yoffset: raw.yoffset as f32,
                xadvance: raw.xadvance as f32,
            };
            chars.insert(raw.id, info);
        }
        let line_height = raw_glyphs.iter().map(|g| g.height as f32).max().unwrap_or(0.0);
        let gpu_font = GpuBitmapFont::new(texture_id, line_height, chars);
        self.ui_manager.add_font(name.to_string(), Box::new(gpu_font));
        true
    }

    pub fn ui_manager(&mut self) -> &mut UiManager {
        &mut self.ui_manager
    }

    pub fn render(&mut self, encoder: &mut CommandEncoder, view: &TextureView) {
        // Layout UI
        self.ui_manager.layout(crate::ui::Size::new(self.surface_width as f32, self.surface_height as f32));
        // Сбор команд
        self.ui_manager.render(&mut self.commands, &self.texture_manager);
        // Рендер
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("UI Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        if !self.commands.is_empty() {
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            self.execute_commands(&mut render_pass);
        }
        self.commands.clear();
    }

    fn execute_commands(&mut self, render_pass: &mut RenderPass) {
        let mut groups: HashMap<u64, Vec<Vec<Vertex>>> = HashMap::new();
        for cmd in self.commands.drain(..) {
            groups.entry(cmd.texture_id).or_default().push(cmd.vertices);
        }
        let mut all_vertices = Vec::new();
        let mut draw_calls = Vec::new();
        for (tid, verts_list) in groups.iter() {
            let start = all_vertices.len() as u32;
            let count = verts_list.iter().map(|v| v.len()).sum::<usize>() as u32;
            for verts in verts_list {
                all_vertices.extend(verts);
            }
            draw_calls.push((*tid, start, count));
        }
        if all_vertices.is_empty() {
            return;
        }
        self.queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&all_vertices));
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        for (tid, start, count) in draw_calls {
            let bind_group = if tid == 0 {
                self.texture_manager.get_fallback_bind_group()
            } else {
                self.texture_manager.get_bind_group(tid).unwrap_or_else(|| self.texture_manager.get_fallback_bind_group())
            };
            render_pass.set_bind_group(1, bind_group, &[]);
            render_pass.draw(start..start + count, 0..1);
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    proj: [[f32; 4]; 4],
}

const VERTEX_SHADER: &str = r#"
struct Uniforms { proj: mat4x4<f32>, }
@group(0) @binding(0) var<uniform> uniforms: Uniforms;
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) color: vec4<f32>,
};
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) color: vec4<f32>,
};
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = uniforms.proj * vec4(in.position, 0.0, 1.0);
    out.tex_coord = in.tex_coord;
    out.color = in.color;
    return out;
}
"#;

const FRAGMENT_SHADER: &str = r#"
@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_diffuse: sampler;
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coord) * in.color;
}
"#;