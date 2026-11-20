use thiserror::Error;

#[derive(Debug, Error)]
pub enum RendererInitError {
    #[error("failed to create surface: {0}")]
    SurfaceCreation(String),

    #[error("no suitable GPU adapter found: {0}")]
    AdapterUnavailable(String),

    #[error("failed to create GPU device: {0}")]
    DeviceRequestFailed(String),

    #[error("surface has no supported format")]
    SurfaceFormatUnavailable,

    #[error("vertex buffer creation failed: {0}")]
    VertexBufferCreationFailed(String),
}

#[derive(Debug, Error)]
pub enum TextureError {
    #[error("texture dimensions must be greater than zero")]
    InvalidSize,

    #[error("texture data length does not match RGBA8 dimensions")]
    InvalidDataLength,

    #[error("texture dimensions exceed the GPU limit")]
    DimensionsTooLarge,

    #[error("texture handle allocation exhausted")]
    HandleExhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RenderError {
    #[error("surface lost")]
    SurfaceLost,

    #[error("surface outdated")]
    SurfaceOutdated,

    #[error("surface acquisition timed out")]
    Timeout,

    #[error("surface is occluded")]
    Occluded,

    #[error("surface validation error")]
    Validation,

    #[error("vertex buffer capacity calculation overflowed")]
    VertexBufferCapacityOverflow,

    #[error("requested vertex buffer size exceeds the GPU limit")]
    VertexBufferTooLarge,
}
