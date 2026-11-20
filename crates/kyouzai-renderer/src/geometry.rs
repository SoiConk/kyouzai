use glam::Vec2;

use crate::{
    render_object::{RenderTransform, TextureRegion},
    vertex::Vertex,
};

/// Builds the 6 vertices (two triangles) for one sprite quad.
///
/// `transform.position` is the center of the sprite in
/// render-target-local physical pixels.
///
/// The coordinate origin is at the top-left corner,
/// with positive Y pointing down.
pub(crate) fn sprite_vertices(
    transform: &RenderTransform,
    region: TextureRegion,
    tint: [f32; 4],
    width: u32,
    height: u32,
) -> [Vertex; 6] {
    let half = transform.size * 0.5;
    let center = transform.position;

    let local = [
        Vec2::new(-half.x, -half.y),
        Vec2::new(half.x, -half.y),
        Vec2::new(half.x, half.y),
        Vec2::new(-half.x, -half.y),
        Vec2::new(half.x, half.y),
        Vec2::new(-half.x, half.y),
    ];

    let (sin, cos) = transform.rotation.sin_cos();

    let rotate = |point: Vec2| -> Vec2 {
        Vec2::new(point.x * cos - point.y * sin, point.x * sin + point.y * cos) + center
    };

    let to_ndc = |point: Vec2| -> [f32; 2] {
        let x = point.x / width.max(1) as f32 * 2.0 - 1.0;
        let y = 1.0 - point.y / height.max(1) as f32 * 2.0;
        [x, y]
    };

    let uvs = [
        [region.uv_min[0], region.uv_min[1]],
        [region.uv_max[0], region.uv_min[1]],
        [region.uv_max[0], region.uv_max[1]],
        [region.uv_min[0], region.uv_min[1]],
        [region.uv_max[0], region.uv_max[1]],
        [region.uv_min[0], region.uv_max[1]],
    ];

    std::array::from_fn(|index| Vertex {
        position: to_ndc(rotate(local[index])),
        uv: uvs[index],
        color: tint,
    })
}
