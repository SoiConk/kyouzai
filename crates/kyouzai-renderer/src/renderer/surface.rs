use crate::error::{RenderError, RendererInitError};

pub(super) struct SurfaceState {
    pub(super) surface: wgpu::Surface<'static>,
    pub(super) config: wgpu::SurfaceConfiguration,
    width: u32,
    height: u32,
}

impl SurfaceState {
    pub(super) fn new(
        surface: wgpu::Surface<'static>,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        size: (u32, u32),
    ) -> Result<Self, RendererInitError> {
        let capabilities = surface.get_capabilities(adapter);

        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|format| format.is_srgb())
            .or_else(|| capabilities.formats.first().copied())
            .ok_or(RendererInitError::SurfaceFormatUnavailable)?;

        let (width, height) = size;

        let alpha_mode = [
            wgpu::CompositeAlphaMode::PreMultiplied,
            wgpu::CompositeAlphaMode::PostMultiplied,
            wgpu::CompositeAlphaMode::Auto,
        ]
        .into_iter()
        .find(|mode| capabilities.alpha_modes.contains(mode))
        .unwrap_or(wgpu::CompositeAlphaMode::Opaque);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: width.max(1),
            height: height.max(1),
            present_mode: capabilities
                .present_modes
                .iter()
                .copied()
                .find(|mode| *mode == wgpu::PresentMode::Fifo)
                .unwrap_or(wgpu::PresentMode::AutoVsync),
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(device, &config);

        Ok(Self {
            surface,
            config,
            width,
            height,
        })
    }

    pub(super) fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        if width == 0 || height == 0 {
            return;
        }

        self.config.width = width;
        self.config.height = height;

        self.reconfigure(device);
    }

    pub(super) fn reconfigure(&self, device: &wgpu::Device) {
        self.surface.configure(device, &self.config);
    }

    pub(super) fn acquire_frame(&self) -> Result<(wgpu::SurfaceTexture, bool), RenderError> {
        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => Ok((frame, false)),

            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => Ok((frame, true)),

            wgpu::CurrentSurfaceTexture::Timeout => Err(RenderError::Timeout),

            wgpu::CurrentSurfaceTexture::Occluded => Err(RenderError::Occluded),

            wgpu::CurrentSurfaceTexture::Outdated => Err(RenderError::SurfaceOutdated),

            wgpu::CurrentSurfaceTexture::Lost => Err(RenderError::SurfaceLost),

            wgpu::CurrentSurfaceTexture::Validation => Err(RenderError::Validation),
        }
    }

    pub(super) fn is_zero_sized(&self) -> bool {
        self.width == 0 || self.height == 0
    }
}
