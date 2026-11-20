mod pipeline;
mod surface;
mod texture;
mod vertex_buffer;

use self::pipeline::Pipeline;
use self::surface::SurfaceState;
use self::texture::TextureManager;
use self::vertex_buffer::VertexBuffer;

use crate::error::{RenderError, RendererInitError, TextureError};
use crate::geometry::sprite_vertices;
use crate::vertex::Vertex;
use crate::{RenderObject, TextureHandle};

use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use std::sync::Arc;

struct DrawCommand {
    texture: TextureHandle,
    vertex_range: std::ops::Range<u32>,
}

pub struct Renderer {
    surface_state: SurfaceState,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: Pipeline,
    texture_manager: TextureManager,
    vertex_buffer: VertexBuffer,
    vertices: Vec<Vertex>,
    draw_commands: Vec<DrawCommand>,
}

impl Renderer {
    pub async fn new(
        window_handle: RawWindowHandle,
        display_handle: RawDisplayHandle,
        size: (u32, u32),
    ) -> Result<Self, RendererInitError> {
        let instance = wgpu::Instance::default();

        // SAFETY: The caller guarantees that the window and display handles
        // remain valid for the entire lifetime of this Renderer.
        let surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: Some(display_handle),
                raw_window_handle: window_handle,
            })
        }
        .map_err(|error| RendererInitError::SurfaceCreation(error.to_string()))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                ..Default::default()
            })
            .await
            .map_err(|error| RendererInitError::AdapterUnavailable(error.to_string()))?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Kyouzai Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
                ..Default::default()
            })
            .await
            .map_err(|error| RendererInitError::DeviceRequestFailed(error.to_string()))?;

        let surface_state = SurfaceState::new(surface, &adapter, &device, size)?;

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Kyouzai Texture Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let pipeline = Pipeline::new(&device, surface_state.config.format);

        let texture_manager =
            TextureManager::new(Arc::clone(&pipeline.texture_bind_group_layout), sampler);

        let vertex_buffer = VertexBuffer::new(&device)
            .map_err(|error| RendererInitError::VertexBufferCreationFailed(error.to_string()))?;

        Ok(Self {
            surface_state,
            device,
            queue,
            pipeline,
            texture_manager,
            vertex_buffer,
            vertices: Vec::new(),
            draw_commands: Vec::new(),
        })
    }

    pub fn create_texture(
        &mut self,
        width: u32,
        height: u32,
        rgba8_data: &[u8],
    ) -> Result<TextureHandle, TextureError> {
        self.texture_manager
            .create(&self.device, &self.queue, width, height, rgba8_data)
    }

    pub fn remove_texture(&mut self, handle: TextureHandle) -> bool {
        self.texture_manager.remove(handle)
    }

    pub fn has_texture(&self, handle: TextureHandle) -> bool {
        self.texture_manager.contains(&handle)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_state.resize(&self.device, width, height);
    }

    pub fn render(&mut self, objects: &[RenderObject]) -> Result<(), RenderError> {
        if self.surface_state.is_zero_sized() {
            return Ok(());
        }

        self.vertices.clear();
        self.draw_commands.clear();

        let width = self.surface_state.config.width;
        let height = self.surface_state.config.height;

        for object in objects {
            if !self.texture_manager.contains(&object.texture) {
                continue;
            }

            let object_vertices =
                sprite_vertices(&object.transform, object.region, object.tint, width, height);

            let start = u32::try_from(self.vertices.len())
                .map_err(|_| RenderError::VertexBufferCapacityOverflow)?;

            self.vertices.extend_from_slice(&object_vertices);

            let end = u32::try_from(self.vertices.len())
                .map_err(|_| RenderError::VertexBufferCapacityOverflow)?;

            self.draw_commands.push(DrawCommand {
                texture: object.texture,
                vertex_range: start..end,
            });
        }

        let (output, suboptimal) = match self.surface_state.acquire_frame() {
            Ok(frame) => frame,

            Err(error @ (RenderError::SurfaceLost | RenderError::SurfaceOutdated)) => {
                self.surface_state.reconfigure(&self.device);
                return Err(error);
            }

            Err(error) => return Err(error),
        };

        self.vertex_buffer
            .upload(&self.device, &self.queue, &self.vertices)?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Kyouzai Render Encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Kyouzai Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            pass.set_pipeline(&self.pipeline.render_pipeline);

            if !self.vertices.is_empty() {
                pass.set_vertex_buffer(0, self.vertex_buffer.buffer().slice(..));

                for command in &self.draw_commands {
                    let Some(texture) = self.texture_manager.get(&command.texture) else {
                        continue;
                    };

                    pass.set_bind_group(0, texture.bind_group(), &[]);
                    pass.draw(command.vertex_range.clone(), 0..1);
                }
            }
        }

        self.queue.submit(Some(encoder.finish()));
        self.queue.present(output);

        if suboptimal {
            self.surface_state.reconfigure(&self.device);
        }

        Ok(())
    }
}
