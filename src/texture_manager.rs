use std::collections::HashMap;
use std::sync::Arc;
use wgpu::{BindGroup, BindGroupLayout, Device, Queue, Sampler, Texture, TextureView};
use crate::common::types::Size;

/// Тип сэмплера (способ семплирования текстуры)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SamplerKind {
    Clamp,
    Repeat,
    ClampNearest,
    RepeatNearest,
}

impl SamplerKind {
    pub fn create_sampler(&self, device: &Device) -> Sampler {
        let (address_mode, mag_filter, min_filter) = match self {
            SamplerKind::Clamp => (
                wgpu::AddressMode::ClampToEdge,
                wgpu::FilterMode::Linear,
                wgpu::FilterMode::Linear,
            ),
            SamplerKind::Repeat => (
                wgpu::AddressMode::Repeat,
                wgpu::FilterMode::Linear,
                wgpu::FilterMode::Linear,
            ),
            SamplerKind::ClampNearest => (
                wgpu::AddressMode::ClampToEdge,
                wgpu::FilterMode::Nearest,
                wgpu::FilterMode::Nearest,
            ),
            SamplerKind::RepeatNearest => (
                wgpu::AddressMode::Repeat,
                wgpu::FilterMode::Nearest,
                wgpu::FilterMode::Nearest,
            ),
        };
        device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: address_mode,
            address_mode_v: address_mode,
            address_mode_w: address_mode,
            mag_filter,
            min_filter,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        })
    }
}

pub struct TextureManager {
    textures: HashMap<u64, TextureEntry>,
    samplers: HashMap<SamplerKind, Sampler>,
    bind_groups: HashMap<(u64, SamplerKind), BindGroup>,
    next_id: u64,
    bind_group_layout: Arc<BindGroupLayout>,
    sizes: HashMap<String, Size>,
    sizes_by_id: HashMap<u64, Size>,
}

struct TextureEntry {
    _texture: Texture,
    view: TextureView,
    size: Size,
}

impl TextureManager {
    pub fn new(device: &Device, queue: &Queue, bind_group_layout: &Arc<BindGroupLayout>) -> Self {
        let mut samplers = HashMap::new();
        for kind in [
            SamplerKind::Clamp,
            SamplerKind::Repeat,
            SamplerKind::ClampNearest,
            SamplerKind::RepeatNearest,
        ] {
            samplers.insert(kind, kind.create_sampler(device));
        }

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
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &fallback_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[255, 255, 255, 255],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
        );
        let fallback_view = fallback_texture.create_view(&Default::default());

        let mut bind_groups = HashMap::new();
        for (&kind, sampler) in &samplers {
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("fallback_bg_{:?}", kind)),
                layout: bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&fallback_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                ],
            });
            bind_groups.insert((0, kind), bind_group);
        }

        Self {
            textures: HashMap::new(),
            samplers,
            bind_groups,
            next_id: 1,
            bind_group_layout: bind_group_layout.clone(),
            sizes: HashMap::new(),
            sizes_by_id: HashMap::new(),
        }
    }

    pub fn load_from_rgba(
        &mut self,
        device: &Device,
        queue: &Queue,
        rgba: &[u8],
        width: u32,
        height: u32,
        label: &str,
    ) -> u64 {
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
        let size = Size::new(width as f32, height as f32);
        let id = self.next_id;
        self.textures.insert(
            id,
            TextureEntry {
                _texture: texture,
                view,
                size,
            },
        );
        self.sizes.insert(label.to_string(), size);
        self.sizes_by_id.insert(id, size);
        self.next_id += 1;
        id
    }

    pub fn get_size(&self, name: &str) -> Option<Size> {
        self.sizes.get(name).copied()
    }

    pub fn get_size_by_id(&self, id: u64) -> Option<Size> {
        self.sizes_by_id.get(&id).copied()
    }

    pub fn get_bind_group(
        &mut self,
        device: &Device,
        texture_id: u64,
        sampler_kind: SamplerKind,
    ) -> &BindGroup {
        let key = (texture_id, sampler_kind);
        if !self.bind_groups.contains_key(&key) {
            let entry = match self.textures.get(&texture_id) {
                Some(entry) => entry,
                None => return self.bind_groups.get(&(0, sampler_kind)).unwrap(),
            };
            let sampler = self.samplers.get(&sampler_kind).unwrap();
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("texture_bg_{}_{:?}", texture_id, sampler_kind)),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&entry.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                ],
            });
            self.bind_groups.insert(key, bind_group);
        }
        self.bind_groups.get(&key).unwrap()
    }

    pub fn get_fallback_bind_group(&mut self, sampler_kind: SamplerKind) -> &BindGroup {
        self.bind_groups
            .get(&(0, sampler_kind))
            .expect("Fallback bind group missing")
    }
}