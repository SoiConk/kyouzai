use std::collections::HashMap;
use std::sync::Arc;

use crate::{error::TextureError, render_object::TextureHandle};

/// GPU resources required to sample a texture.
pub(super) struct GpuTexture {
    _texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
}

/// Manages GPU textures and their bind groups.
pub(super) struct TextureManager {
    textures: HashMap<TextureHandle, GpuTexture>,
    bind_group_layout: Arc<wgpu::BindGroupLayout>,
    sampler: wgpu::Sampler,
    next_handle: u64,
}

impl TextureManager {
    pub(super) fn new(
        bind_group_layout: Arc<wgpu::BindGroupLayout>,
        sampler: wgpu::Sampler,
    ) -> Self {
        Self {
            textures: HashMap::new(),
            bind_group_layout,
            sampler,
            next_handle: 1,
        }
    }

    pub(super) fn create(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        rgba8_data: &[u8],
    ) -> Result<TextureHandle, TextureError> {
        if width == 0 || height == 0 {
            return Err(TextureError::InvalidSize);
        }

        let max_dimension = device.limits().max_texture_dimension_2d;

        if width > max_dimension || height > max_dimension {
            return Err(TextureError::DimensionsTooLarge);
        }

        let expected_len = (width as usize)
            .checked_mul(height as usize)
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or(TextureError::InvalidDataLength)?;

        if rgba8_data.len() != expected_len {
            return Err(TextureError::InvalidDataLength);
        }

        let next_handle = self
            .next_handle
            .checked_add(1)
            .ok_or(TextureError::HandleExhausted)?;

        let handle = TextureHandle::new(self.next_handle);

        let bytes_per_row = width
            .checked_mul(4)
            .ok_or(TextureError::InvalidDataLength)?;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Kyouzai Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
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
            rgba8_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Kyouzai Texture Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        self.textures.insert(
            handle,
            GpuTexture {
                _texture: texture,
                bind_group,
            },
        );

        self.next_handle = next_handle;

        Ok(handle)
    }

    pub(super) fn remove(&mut self, handle: TextureHandle) -> bool {
        self.textures.remove(&handle).is_some()
    }

    pub(super) fn get(&self, handle: &TextureHandle) -> Option<&GpuTexture> {
        self.textures.get(handle)
    }

    pub(super) fn contains(&self, handle: &TextureHandle) -> bool {
        self.textures.contains_key(handle)
    }
}

impl GpuTexture {
    pub(super) fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
