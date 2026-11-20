#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(u64);

impl TextureId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpriteRegion {
    uv_min: [f32; 2],
    uv_max: [f32; 2],
}

impl SpriteRegion {
    pub const FULL: Self = Self {
        uv_min: [0.0, 0.0],
        uv_max: [1.0, 1.0],
    };

    pub const fn new(uv_min: [f32; 2], uv_max: [f32; 2]) -> Self {
        Self { uv_min, uv_max }
    }

    /// Computes the UV region of one cell in a `columns x rows` grid spritesheet
    /// (0-indexed, `col`/`row` growing right/down). Used to turn a `pet.json` manifest's
    /// `{row, frame_count}` animation definitions into concrete per-frame regions without
    /// every call site re-deriving the same division-by-grid-size math.
    pub fn from_grid(col: u32, row: u32, columns: u32, rows: u32) -> Option<Self> {
        if columns == 0 || rows == 0 || col >= columns || row >= rows {
            return None;
        }

        let u0 = col as f32 / columns as f32;
        let u1 = (col + 1) as f32 / columns as f32;
        let v0 = row as f32 / rows as f32;
        let v1 = (row + 1) as f32 / rows as f32;

        Some(Self::new([u0, v0], [u1, v1]))
    }

    pub fn uv_min(&self) -> [f32; 2] {
        self.uv_min
    }

    pub fn uv_max(&self) -> [f32; 2] {
        self.uv_max
    }
}

impl Default for SpriteRegion {
    fn default() -> Self {
        Self::FULL
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sprite {
    texture: TextureId,
    region: SpriteRegion,
    tint: [f32; 4],
}

impl Sprite {
    pub const fn new(texture: TextureId) -> Self {
        Self {
            texture,
            region: SpriteRegion::FULL,
            tint: [1.0, 1.0, 1.0, 1.0],
        }
    }

    pub const fn with_region(texture: TextureId, region: SpriteRegion) -> Self {
        Self {
            texture,
            region,
            tint: [1.0, 1.0, 1.0, 1.0],
        }
    }

    pub fn texture(&self) -> TextureId {
        self.texture
    }

    pub fn region(&self) -> SpriteRegion {
        self.region
    }

    pub fn tint(&self) -> [f32; 4] {
        self.tint
    }

    pub fn set_texture(&mut self, texture: TextureId) {
        self.texture = texture;
    }

    pub fn set_region(&mut self, region: SpriteRegion) {
        self.region = region;
    }

    pub fn set_tint(&mut self, tint: [f32; 4]) {
        self.tint = tint.map(|value| value.clamp(0.0, 1.0));
    }
}
