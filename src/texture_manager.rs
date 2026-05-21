use std::collections::HashMap;
use wgpu::{Device, Queue, Texture, TextureView, Sampler, BindGroup, BindGroupLayout};

use crate::common::types::Size;

pub struct TextureManager {
    textures: HashMap<u64, TextureEntry>,
    next_id: u64,
    bind_group_layout: BindGroupLayout,
    fallback_bind_group: BindGroup,
    sizes: HashMap<String, Size>, 
}

struct TextureEntry {
    _texture: Texture,
    _view: TextureView,
    _sampler: Sampler,
    bind_group: BindGroup,
}

impl TextureManager {
    pub fn new(device: &Device, bind_group_layout: &BindGroupLayout) -> Self {
        let fallback_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fallback_texture"),
            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let fallback_view = fallback_texture.create_view(&Default::default());
        let fallback_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let fallback_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fallback_bg"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&fallback_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&fallback_sampler),
                },
            ],
        });

        Self {
            textures: HashMap::new(),
            next_id: 1,
            bind_group_layout: bind_group_layout.clone(),
            fallback_bind_group,
            sizes: HashMap::new(),
        }
    }

    pub fn load_from_rgba(&mut self, device: &Device, queue: &Queue, rgba: &[u8], width: u32, height: u32, label: &str) -> u64 {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        );
        let view = texture.create_view(&Default::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{}_bg", label)),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        let id = self.next_id;
        self.textures.insert(id, TextureEntry { _texture: texture, _view: view, _sampler: sampler, bind_group });
        self.sizes.insert(label.to_string(), Size::new(width as f32, height as f32));
        self.next_id += 1;
        id
    }

    pub fn get_size(&self, name: &str) -> Option<Size> {
        self.sizes.get(name).copied()
    }

    pub fn get_bind_group(&self, id: u64) -> Option<&BindGroup> {
        self.textures.get(&id).map(|e| &e.bind_group)
    }

    pub fn get_fallback_bind_group(&self) -> &BindGroup {
        &self.fallback_bind_group
    }
}