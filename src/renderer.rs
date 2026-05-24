// src/renderer.rs
use wgpu::{Device, Queue, TextureFormat, TextureView, CommandEncoder, 
    RenderPass, RenderPipeline, BindGroup};
use wgpu::util::DeviceExt;

use crate::common::types::{FontLoader, GlyphInfo, GpuBitmapFont, Size, TextureLoader};
use crate::common::vertex::{Vertex, DrawCommand};
use crate::common::primitives::Primitives;
use crate::texture_manager::TextureManager;
use crate::ui_manager::UiManager;

// Алиас для сложного типа ключа группировки
type DrawGroupKey = (u64, Option<(u32, u32, u32, u32)>);

pub struct UiRenderer {
    device: Device,
    queue: Queue,
    pipeline: RenderPipeline,
    uniform_bind_group: BindGroup,
    uniform_buffer: wgpu::Buffer,
    ui_manager: UiManager,
    commands: Vec<DrawCommand>,
    vertex_buffer: wgpu::Buffer,
    surface_width: u32,
    surface_height: u32,
    index_buffer: wgpu::Buffer,
}

impl UiRenderer {
    const MAX_VERTICES_PER_CHUNK: usize = 100_000;

    pub fn new(device: &Device, queue: &Queue, surface_format: TextureFormat, width: u32, height: u32, primitives: Box<dyn Primitives + Send + Sync>) -> Self {
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ui_pipeline_layout"),
            bind_group_layouts: &[Some(&uniform_layout), Some(&texture_bind_group_layout)],
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

            //primitive: wgpu::PrimitiveState::default(),
            primitive: wgpu::PrimitiveState {
                cull_mode: None, // Отключаем back-face culling для 2D UI
                ..wgpu::PrimitiveState::default()
            },            
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ui_vertex_buffer"),
            size: (Self::MAX_VERTICES_PER_CHUNK * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ui_index_buffer"),
            size: (Self::MAX_VERTICES_PER_CHUNK * 2 * std::mem::size_of::<u32>()) as u64, // ~2 индекса на вершину
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let texture_manager = TextureManager::new(device, queue, &texture_bind_group_layout);
        let ui_manager = UiManager::new(
            Size::new(width as f32, height as f32),
            texture_manager,
            primitives,
            1.0,
        );

        Self {
            device: device.clone(),
            queue: queue.clone(),
            pipeline,
            uniform_bind_group,
            uniform_buffer,
            ui_manager,
            commands: Vec::new(),
            vertex_buffer,
            surface_width: width,
            surface_height: height,
            index_buffer: index_buffer,
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
        self.ui_manager.layout(Size::new(width as f32, height as f32));
    }

    pub fn load_texture(&mut self, name: &str, loader: &dyn TextureLoader) -> Option<u64> {
        let (rgba, w, h) = loader.load_texture_rgba(name)?;
        Some(self.ui_manager.texture_manager_mut().load_from_rgba(&self.device, &self.queue, &rgba, w, h, name))
    }

    pub fn load_font(&mut self, name: &str, loader: &dyn FontLoader) -> bool {
        let (rgba, atlas_w, atlas_h, raw_glyphs) = match loader.load_font_data(name) {
            Some(data) => data,
            None => return false,
        };
        let texture_id = self.ui_manager.texture_manager_mut().load_from_rgba(
            &self.device, &self.queue, &rgba, atlas_w, atlas_h,
            &format!("font_{}", name)
        );

        let line_height = raw_glyphs.iter()
            .map(|g| g.height as f32)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        let mut chars = std::collections::HashMap::new();
        for raw in raw_glyphs {
            let info = GlyphInfo {
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

        let gpu_font = GpuBitmapFont::new(texture_id, line_height, chars);
        self.ui_manager.add_font(name.to_string(), Box::new(gpu_font));
        true
    }

    pub fn ui_manager(&mut self) -> &mut UiManager {
        &mut self.ui_manager
    }

    pub fn render(&mut self, encoder: &mut CommandEncoder, view: &TextureView) {
        //self.ui_manager.layout(Size::new(self.surface_width as f32, self.surface_height as f32));
        self.ui_manager.render(&mut self.commands);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("UI Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
//                    load: wgpu::LoadOp::Load,
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                                        r: 0.1,
                                        g: 0.2,
                                        b: 0.3,
                                        a: 1.0,
                                    }),                    
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

        //self.commands.clear();
    }

    /// Выполняет сгруппированные draw call'ы с учётом scissor-прямоугольников.
    /// 
fn execute_commands(&mut self, render_pass: &mut RenderPass) {
    type DrawGroupKey = (u64, Option<(u32, u32, u32, u32)>);

    // ✅ Adjacent batching: группы только для СОСЕДНИХ команд
    let mut groups: Vec<(DrawGroupKey, (Vec<Vertex>, Vec<u32>))> = Vec::new();

    for cmd in self.commands.drain(..) {
        let key_scissor = cmd.scissor_rect.map(|rect| {
            (
                rect.x.max(0.0) as u32,
                rect.y.max(0.0) as u32,
                rect.w.max(0.0) as u32,
                rect.h.max(0.0) as u32,
            )
        });
        let key = (cmd.texture_id, key_scissor);
        
        let should_merge = groups.last()
            .map(|(k, _)| *k == key)
            .unwrap_or(false);
        
        if should_merge {
            let group = &mut groups.last_mut().unwrap().1;
            let base_vertex = group.0.len() as u32;
            group.0.extend(cmd.vertices);
            group.1.extend(cmd.indices.into_iter().map(|i| i + base_vertex));
        } else {
            let verts = cmd.vertices;
            let indices = cmd.indices;
            groups.push((key, (verts, indices)));
        }
    }

    if groups.is_empty() {
        return;
    }

    // 🔧 Хелпер для отправки чанка на GPU
    let flush_chunk = |r_pass: &mut RenderPass,
                       v_buf: &wgpu::Buffer,
                       i_buf: &wgpu::Buffer,
                       q: &wgpu::Queue,
                       draws: &[(u64, Option<(u32, u32, u32, u32)>, u32, u32)],
                       verts: &[Vertex],
                       idxs: &[u32],
                       tex_mgr: &crate::texture_manager::TextureManager,
                       surf_w: u32,
                       surf_h: u32| {
        if verts.is_empty() { return; }

        q.write_buffer(v_buf, 0, bytemuck::cast_slice(verts));
        q.write_buffer(i_buf, 0, bytemuck::cast_slice(idxs));
        
        r_pass.set_vertex_buffer(0, v_buf.slice(..));
        r_pass.set_index_buffer(i_buf.slice(..), wgpu::IndexFormat::Uint32);

        for (tid, scissor_key, idx_start, idx_count) in draws {
            let bg = if *tid == 0 {
                tex_mgr.get_fallback_bind_group()
            } else {
                tex_mgr.get_bind_group(*tid)
                    .unwrap_or_else(|| tex_mgr.get_fallback_bind_group())
            };
            r_pass.set_bind_group(1, bg, &[]);

            // 🔧 FIX: Дереференс скиссора через &Some((x, y, w, h))
            if let &Some((x, y, sw, sh)) = scissor_key {
                // Clamp к размеру поверхности (защита от panic wgpu)
                let x_clamped = x.min(surf_w);
                let y_clamped = y.min(surf_h);
                let w_clamped = sw.min(surf_w.saturating_sub(x));
                let h_clamped = sh.min(surf_h.saturating_sub(y));
                
                if w_clamped > 0 && h_clamped > 0 {
                    r_pass.set_scissor_rect(x_clamped, y_clamped, w_clamped, h_clamped);
                } else {
                    continue;
                }
            } else {
                r_pass.set_scissor_rect(0, 0, surf_w, surf_h);
            }

            r_pass.draw_indexed(*idx_start..(*idx_start + *idx_count), 0, 0..1);
        }
    };

    // 🔧 Чанковая сборка геометрии
    let mut chunk_verts: Vec<Vertex> = Vec::with_capacity(Self::MAX_VERTICES_PER_CHUNK);
    let mut chunk_idxs: Vec<u32> = Vec::with_capacity(Self::MAX_VERTICES_PER_CHUNK * 3 / 2);
    let mut chunk_draws: Vec<(u64, Option<(u32, u32, u32, u32)>, u32, u32)> = Vec::new();

    for ((tid, scissor_key), (g_v, g_i)) in groups {
        // Если чанк + новая группа превышают лимит → сброс на GPU
        if chunk_verts.len() + g_v.len() > Self::MAX_VERTICES_PER_CHUNK ||
           chunk_idxs.len() + g_i.len() > chunk_idxs.capacity() {
            
            flush_chunk(
                render_pass, 
                &self.vertex_buffer, 
                &self.index_buffer, 
                &self.queue,
                &chunk_draws, 
                &chunk_verts, 
                &chunk_idxs,
                self.ui_manager.texture_manager(), 
                self.surface_width, 
                self.surface_height,
            );
            chunk_verts.clear();
            chunk_idxs.clear();
            chunk_draws.clear();
        }

        let base_v = chunk_verts.len() as u32;
        chunk_verts.extend(g_v);
        chunk_idxs.extend(g_i.iter().map(|idx| idx + base_v));
        chunk_draws.push((tid, scissor_key, chunk_idxs.len() as u32 - g_i.len() as u32, g_i.len() as u32));
    }

    // 🔧 Сброс остатка
    if !chunk_verts.is_empty() {
        flush_chunk(
            render_pass, 
            &self.vertex_buffer, 
            &self.index_buffer, 
            &self.queue,
            &chunk_draws, 
            &chunk_verts, 
            &chunk_idxs,
            self.ui_manager.texture_manager(), 
            self.surface_width, 
            self.surface_height,
        );
    }
}

    fn execute_commands2(&mut self, render_pass: &mut RenderPass) {
        type DrawGroupKey = (u64, Option<(u32, u32, u32, u32)>);
        
        // ✅ Adjacent batching: группы только для СОСЕДНИХ команд
        // Используем Vec<(Key, Group)> вместо HashMap/IndexMap
        let mut groups: Vec<(DrawGroupKey, (Vec<Vertex>, Vec<u32>))> = Vec::new();

        for cmd in self.commands.drain(..) {
            let key_scissor = cmd.scissor_rect.map(|rect| {
                (rect.x.max(0.0) as u32, rect.y.max(0.0) as u32,
                rect.w.max(0.0) as u32, rect.h.max(0.0) as u32)
            });
            let key = (cmd.texture_id, key_scissor);
            
            // Проверяем последнюю группу — если ключ совпадает, добавляем в неё
            let should_merge = groups.last()
                .map(|(k, _)| *k == key)
                .unwrap_or(false);
            
            if should_merge {
                let group = &mut groups.last_mut().unwrap().1;
                let base_vertex = group.0.len() as u32;
                group.0.extend(cmd.vertices);
                group.1.extend(cmd.indices.into_iter().map(|i| i + base_vertex));
            } else {
                // Создаём новую группу (даже если такой ключ уже был раньше!)
                let verts = cmd.vertices;
                let indices = cmd.indices;
                groups.push((key, (verts, indices)));
            }
        }

        // Второй цикл: слияние групп в общий буфер (порядок сохранён!)
        let mut all_vertices: Vec<Vertex> = Vec::new();
        let mut all_indices: Vec<u32> = Vec::new();
        let mut draw_calls: Vec<(u64, Option<(u32, u32, u32, u32)>, u32, u32)> = Vec::new();

        for ((tid, scissor_key), (verts, idxs)) in groups {
            let base_vertex = all_vertices.len() as u32;
            let idx_start = all_indices.len() as u32;
            let idx_count = idxs.len() as u32;

            all_vertices.extend(verts);
            let adjusted_indices: Vec<u32> = idxs.into_iter().map(|i| i + base_vertex).collect();
            all_indices.extend(adjusted_indices);
            draw_calls.push((tid, scissor_key, idx_start, idx_count));
        }

        if all_vertices.is_empty() { return; }

        self.queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&all_vertices));
        self.queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&all_indices));
        
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        let texture_manager = self.ui_manager.texture_manager();
        for (tid, scissor_key, idx_start, idx_count) in draw_calls {
            let bind_group = if tid == 0 {
                texture_manager.get_fallback_bind_group()
            } else {
                texture_manager.get_bind_group(tid)
                    .unwrap_or_else(|| texture_manager.get_fallback_bind_group())
            };
            render_pass.set_bind_group(1, bind_group, &[]);

            if let Some((x, y, w, h)) = scissor_key {
                if w > 0 && h > 0 {
                    render_pass.set_scissor_rect(x, y, w, h);
                } else { continue; }
            } else {
                render_pass.set_scissor_rect(0, 0, self.surface_width, self.surface_height);
            }

            render_pass.draw_indexed(idx_start..(idx_start + idx_count), 0, 0..1);
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
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_diffuse: sampler;
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coord) * in.color;
}
"#;