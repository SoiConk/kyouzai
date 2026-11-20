use glam::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    position: Vec2,
    size: Vec2,
    scale: Vec2,
    rotation: f32,
}

impl Transform {
    pub const fn new(position: Vec2, size: Vec2) -> Self {
        Self {
            position,
            size,
            scale: Vec2::ONE,
            rotation: 0.0,
        }
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn size(&self) -> Vec2 {
        self.size
    }

    pub fn scale(&self) -> Vec2 {
        self.scale
    }

    pub fn rotation(&self) -> f32 {
        self.rotation
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    pub fn set_size(&mut self, size: Vec2) {
        self.size = size;
    }

    pub fn set_scale(&mut self, scale: Vec2) {
        self.scale = scale;
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
    }

    pub fn translated(mut self, delta: Vec2) -> Self {
        self.position += delta;
        self
    }

    pub fn scaled(mut self, factor: Vec2) -> Self {
        self.scale *= factor;
        self
    }

    pub fn rotated(mut self, angle: f32) -> Self {
        self.rotation += angle;
        self
    }

    pub fn effective_size(&self) -> Vec2 {
        self.size * self.scale
    }

    /// Maps a point from local window-pixel space (the same space the renderer draws
    /// into, and the same space winit reports `CursorMoved` positions in) into
    /// normalized sprite-local coordinates in `[0, 1] x [0, 1]`, where `(0, 0)` is the
    /// top-left of this sprite's (rotated) bounding box and `(1, 1)` is the bottom-right.
    ///
    /// Returns `None` when the point falls outside the bounding box entirely.
    ///
    /// This is deliberately kept as the exact mathematical inverse of the vertex
    /// transform `kyouzai-renderer` applies when placing the sprite quad (see
    /// `sprite_vertices` in that crate), so that mouse hit-testing lines up
    /// pixel-for-pixel with what is actually drawn, rotation included. If the renderer's
    /// transform ever changes, this must change with it.
    ///
    /// `window_center` is the point the renderer currently anchors the sprite to (see the
    /// architecture note on `Renderer::render`: it is always the render target's own
    /// center, not `self.position()`, because each companion owns a window sized exactly
    /// to its sprite).
    pub fn hit_test_normalized(&self, window_point: Vec2, window_center: Vec2) -> Option<Vec2> {
        let half = self.effective_size() * 0.5;
        if half.x <= f32::EPSILON || half.y <= f32::EPSILON {
            return None;
        }

        let relative = window_point - window_center;
        let (sin, cos) = self.rotation.sin_cos();

        // Inverse rotation: undo the +rotation the renderer applies by rotating by
        // -rotation here instead.
        let local = Vec2::new(
            relative.x * cos + relative.y * sin,
            -relative.x * sin + relative.y * cos,
        );

        if local.x.abs() > half.x || local.y.abs() > half.y {
            return None;
        }

        Some(Vec2::new(
            (local.x / half.x + 1.0) * 0.5,
            (local.y / half.y + 1.0) * 0.5,
        ))
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new(Vec2::ZERO, Vec2::ZERO)
    }
}
