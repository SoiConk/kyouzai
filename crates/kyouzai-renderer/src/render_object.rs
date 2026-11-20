use glam::Vec2;

/// Opaque handle identifying a texture managed by the Renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureHandle(u64);

impl TextureHandle {
    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }
}

/// Transform of an object in render-target-local coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderTransform {
    /// Position relative to the top-left corner of the render target.
    pub position: Vec2,
    /// Final dimensions of the object in render-target-local physical pixels,
    /// with scale already applied.
    pub size: Vec2,
    /// Rotation in radians.
    pub rotation: f32,
}

/// Texture region expressed in normalized UV coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextureRegion {
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
}

/// Describes one object that the Renderer should draw.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderObject {
    pub texture: TextureHandle,
    pub transform: RenderTransform,
    pub region: TextureRegion,
    pub tint: [f32; 4],
}
