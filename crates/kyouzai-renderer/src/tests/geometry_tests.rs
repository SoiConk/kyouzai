use glam::Vec2;

use crate::{RenderTransform, TextureRegion, geometry::sprite_vertices};

fn full_region() -> TextureRegion {
    TextureRegion {
        uv_min: [0.0, 0.0],
        uv_max: [1.0, 1.0],
    }
}

#[test]
fn top_left_vertex_samples_the_top_of_the_source_image() {
    let transform = RenderTransform {
        position: Vec2::new(50.0, 50.0),
        size: Vec2::new(100.0, 100.0),
        rotation: 0.0,
    };

    let vertices = sprite_vertices(&transform, full_region(), [1.0, 1.0, 1.0, 1.0], 100, 100);

    let [ndc_x, ndc_y] = vertices[0].position;

    assert!((ndc_x - -1.0).abs() < 1e-4);
    assert!((ndc_y - 1.0).abs() < 1e-4);

    assert!((vertices[0].uv[1] - 0.0).abs() < 1e-4);
}

#[test]
fn bottom_left_vertex_samples_the_bottom_of_the_source_image() {
    let transform = RenderTransform {
        position: Vec2::new(50.0, 50.0),
        size: Vec2::new(100.0, 100.0),
        rotation: 0.0,
    };

    let vertices = sprite_vertices(&transform, full_region(), [1.0, 1.0, 1.0, 1.0], 100, 100);

    assert!((vertices[5].uv[1] - 1.0).abs() < 1e-4);
}

#[test]
fn quad_is_positioned_at_render_transform_position() {
    let transform = RenderTransform {
        position: Vec2::new(128.0, 128.0),
        size: Vec2::new(50.0, 50.0),
        rotation: 0.0,
    };

    let vertices = sprite_vertices(&transform, full_region(), [1.0, 1.0, 1.0, 1.0], 256, 256);

    // Top-left = position - half size = (103, 103).
    // NDC: x = -0.1953125, y = 0.1953125.
    let [ndc_x, ndc_y] = vertices[0].position;

    assert!((ndc_x - -0.1953125).abs() < 1e-4);
    assert!((ndc_y - 0.1953125).abs() < 1e-4);
}

#[test]
fn rotates_vertices_around_object_center() {
    use std::f32::consts::FRAC_PI_2;

    let transform = RenderTransform {
        position: Vec2::new(100.0, 50.0),
        size: Vec2::new(100.0, 80.0),
        rotation: FRAC_PI_2,
    };

    let vertices = sprite_vertices(&transform, full_region(), [1.0, 1.0, 1.0, 1.0], 200, 100);

    let expected = [[0.4, 1.0], [0.4, -1.0], [-0.4, -1.0]];

    for (vertex, expected_position) in vertices.iter().zip(expected.iter()) {
        assert!((vertex.position[0] - expected_position[0]).abs() < 1e-4);
        assert!((vertex.position[1] - expected_position[1]).abs() < 1e-4);
    }
}

#[test]
fn preserves_vertex_tint() {
    let transform = RenderTransform {
        position: Vec2::new(50.0, 50.0),
        size: Vec2::new(100.0, 100.0),
        rotation: 0.0,
    };

    let tint = [1.0, 0.5, 0.25, 0.75];

    let vertices = sprite_vertices(&transform, full_region(), tint, 100, 100);

    for vertex in &vertices {
        assert_eq!(vertex.color, tint);
    }
}

#[test]
fn maps_texture_region_uv_coordinates_correctly() {
    let transform = RenderTransform {
        position: Vec2::new(50.0, 50.0),
        size: Vec2::new(100.0, 100.0),
        rotation: 0.0,
    };

    let region = TextureRegion {
        uv_min: [0.25, 0.5],
        uv_max: [0.75, 1.0],
    };

    let vertices = sprite_vertices(&transform, region, [1.0, 1.0, 1.0, 1.0], 100, 100);

    let expected_uvs = [
        [0.25, 0.5],
        [0.75, 0.5],
        [0.75, 1.0],
        [0.25, 0.5],
        [0.75, 1.0],
        [0.25, 1.0],
    ];

    for (vertex, expected_uv) in vertices.iter().zip(expected_uvs.iter()) {
        assert!((vertex.uv[0] - expected_uv[0]).abs() < 1e-4);
        assert!((vertex.uv[1] - expected_uv[1]).abs() < 1e-4);
    }
}
