use bytemuck::cast_slice;

use crate::{error::RenderError, vertex::Vertex};

const INITIAL_CAPACITY: usize = 6;

pub(super) struct VertexBuffer {
    buffer: wgpu::Buffer,
    capacity: usize,
}

impl VertexBuffer {
    pub(super) fn new(device: &wgpu::Device) -> Result<Self, RenderError> {
        let capacity = INITIAL_CAPACITY;
        let size = Self::buffer_size(capacity, device)?;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Kyouzai Sprite Vertex Buffer"),
            size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(Self { buffer, capacity })
    }

    pub(super) fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertices: &[Vertex],
    ) -> Result<(), RenderError> {
        if vertices.is_empty() {
            return Ok(());
        }

        self.ensure_capacity(device, vertices.len())?;

        queue.write_buffer(&self.buffer, 0, cast_slice(vertices));

        Ok(())
    }

    pub(super) fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    fn ensure_capacity(
        &mut self,
        device: &wgpu::Device,
        required: usize,
    ) -> Result<(), RenderError> {
        if required <= self.capacity {
            return Ok(());
        }

        let capacity = required
            .checked_next_power_of_two()
            .ok_or(RenderError::VertexBufferCapacityOverflow)?;

        let size = Self::buffer_size(capacity, device)?;

        self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Kyouzai Sprite Vertex Buffer"),
            size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        self.capacity = capacity;

        Ok(())
    }

    fn buffer_size(capacity: usize, device: &wgpu::Device) -> Result<u64, RenderError> {
        let byte_size = capacity
            .checked_mul(std::mem::size_of::<Vertex>())
            .ok_or(RenderError::VertexBufferCapacityOverflow)?;

        let size =
            u64::try_from(byte_size).map_err(|_| RenderError::VertexBufferCapacityOverflow)?;

        if size > device.limits().max_buffer_size {
            return Err(RenderError::VertexBufferTooLarge);
        }

        Ok(size)
    }
}
