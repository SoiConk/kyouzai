use crate::transform::Transform;
use glam::Vec2;

#[test]
fn new_transform_uses_default_scale_and_rotation() {
    let transform = Transform::new(Vec2::new(10.0, 20.0), Vec2::new(100.0, 50.0));

    assert_eq!(transform.position(), Vec2::new(10.0, 20.0));
    assert_eq!(transform.size(), Vec2::new(100.0, 50.0));
    assert_eq!(transform.scale(), Vec2::ONE);
    assert_eq!(transform.rotation(), 0.0);
}

#[test]
fn effective_size_applies_scale() {
    let mut transform = Transform::new(Vec2::ZERO, Vec2::new(100.0, 50.0));

    transform.set_scale(Vec2::new(2.0, 3.0));

    assert_eq!(transform.effective_size(), Vec2::new(200.0, 150.0));
}

#[test]
fn transformations_update_expected_values() {
    let transform = Transform::new(Vec2::new(10.0, 20.0), Vec2::new(100.0, 50.0))
        .translated(Vec2::new(5.0, -5.0))
        .scaled(Vec2::new(2.0, 2.0))
        .rotated(1.5);

    assert_eq!(transform.position(), Vec2::new(15.0, 15.0));
    assert_eq!(transform.scale(), Vec2::new(2.0, 2.0));
    assert_eq!(transform.rotation(), 1.5);
}

#[test]
fn hit_test_normalized_finds_top_left_without_rotation() {
    let transform = Transform::new(Vec2::ZERO, Vec2::new(100.0, 50.0));
    let window_center = Vec2::new(200.0, 200.0);

    // 25px right, 10px up from center -> inside the 100x50 box.
    let point = window_center + Vec2::new(25.0, -10.0);
    let normalized = transform
        .hit_test_normalized(point, window_center)
        .expect("point is inside the bounding box");

    assert!((normalized.x - 0.75).abs() < 1e-5);
    assert!((normalized.y - 0.3).abs() < 1e-5);
}

#[test]
fn hit_test_normalized_rejects_points_outside_bounding_box() {
    let transform = Transform::new(Vec2::ZERO, Vec2::new(100.0, 50.0));
    let window_center = Vec2::new(200.0, 200.0);

    let far_away = window_center + Vec2::new(1000.0, 0.0);
    assert!(
        transform
            .hit_test_normalized(far_away, window_center)
            .is_none()
    );
}
