use crate::{MonitorArea, MonitorAreas, ScreenBounds};

use glam::Vec2;

#[test]
fn width_and_height_are_derived_from_min_max() {
    let bounds = ScreenBounds::new(Vec2::new(10.0, 20.0), Vec2::new(110.0, 220.0));
    assert_eq!(bounds.width(), 100.0);
    assert_eq!(bounds.height(), 200.0);
}

fn monitor(min: Vec2, max: Vec2, work_min: Vec2, work_max: Vec2) -> MonitorArea {
    MonitorArea {
        bounds: ScreenBounds::new(min, max),
        work_area: ScreenBounds::new(work_min, work_max),
    }
}

#[test]
fn screen_bounds_contains_uses_half_open_edges() {
    let bounds = ScreenBounds::new(Vec2::ZERO, Vec2::new(1920.0, 1080.0));

    assert!(bounds.contains(Vec2::new(0.0, 0.0)));
    assert!(bounds.contains(Vec2::new(1919.0, 1079.0)));
    assert!(!bounds.contains(Vec2::new(1920.0, 500.0)));
    assert!(!bounds.contains(Vec2::new(500.0, 1080.0)));
}

#[test]
fn screen_bounds_calculates_distance_to_nearest_point() {
    let bounds = ScreenBounds::new(Vec2::ZERO, Vec2::new(1920.0, 1080.0));

    assert_eq!(bounds.distance_squared_to(Vec2::new(2000.0, 500.0)), 6400.0);
    assert_eq!(bounds.distance_squared_to(Vec2::new(500.0, 500.0)), 0.0);
}

#[test]
fn monitor_at_finds_monitor_containing_position() {
    let left = monitor(
        Vec2::ZERO,
        Vec2::new(1920.0, 1080.0),
        Vec2::ZERO,
        Vec2::new(1920.0, 1040.0),
    );
    let right = monitor(
        Vec2::new(1920.0, 0.0),
        Vec2::new(3840.0, 1080.0),
        Vec2::new(1920.0, 0.0),
        Vec2::new(3840.0, 1040.0),
    );

    let monitors = MonitorAreas::new(vec![left, right]);

    assert!(std::ptr::eq(
        monitors.monitor_at(Vec2::new(2500.0, 500.0)).unwrap(),
        &monitors.areas()[1]
    ));
}

#[test]
fn nearest_monitor_finds_closest_monitor_in_gap() {
    let left = monitor(
        Vec2::ZERO,
        Vec2::new(1920.0, 1080.0),
        Vec2::ZERO,
        Vec2::new(1920.0, 1040.0),
    );
    let right = monitor(
        Vec2::new(2000.0, 0.0),
        Vec2::new(3920.0, 1080.0),
        Vec2::new(2000.0, 0.0),
        Vec2::new(3920.0, 1040.0),
    );

    let monitors = MonitorAreas::new(vec![left, right]);

    assert!(std::ptr::eq(
        monitors.nearest_monitor(Vec2::new(1950.0, 500.0)).unwrap(),
        &monitors.areas()[0]
    ));
}

#[test]
fn adjacent_monitor_finds_monitor_in_movement_direction() {
    let left = monitor(
        Vec2::ZERO,
        Vec2::new(1920.0, 1080.0),
        Vec2::ZERO,
        Vec2::new(1920.0, 1040.0),
    );
    let right = monitor(
        Vec2::new(1920.0, 200.0),
        Vec2::new(3840.0, 1280.0),
        Vec2::new(1920.0, 200.0),
        Vec2::new(3840.0, 1240.0),
    );

    let monitors = MonitorAreas::new(vec![left, right]);
    let current = &monitors.areas()[0];

    let crossing_bounds = ScreenBounds::new(Vec2::new(1870.0, 450.0), Vec2::new(1970.0, 550.0));

    let adjacent = monitors.adjacent_monitor(current, crossing_bounds, Vec2::X);

    assert!(std::ptr::eq(adjacent.unwrap(), &monitors.areas()[1]));
}

#[test]
fn adjacent_monitor_does_not_cross_non_overlapping_edges() {
    let left = monitor(
        Vec2::ZERO,
        Vec2::new(1920.0, 1080.0),
        Vec2::ZERO,
        Vec2::new(1920.0, 1040.0),
    );
    let right = monitor(
        Vec2::new(1920.0, 1200.0),
        Vec2::new(3840.0, 2280.0),
        Vec2::new(1920.0, 1200.0),
        Vec2::new(3840.0, 2240.0),
    );

    let monitors = MonitorAreas::new(vec![left, right]);
    let current = &monitors.areas()[0];

    let crossing_bounds = ScreenBounds::new(Vec2::new(1870.0, 450.0), Vec2::new(1970.0, 550.0));

    assert!(
        monitors
            .adjacent_monitor(current, crossing_bounds, Vec2::X)
            .is_none()
    );
}

#[test]
fn screen_bounds_intersects_when_rectangles_overlap() {
    let a = ScreenBounds::new(Vec2::ZERO, Vec2::new(1920.0, 1080.0));
    let b = ScreenBounds::new(Vec2::new(1800.0, 0.0), Vec2::new(3000.0, 1080.0));

    assert!(a.intersects(&b));
}

#[test]
fn screen_bounds_does_not_intersect_when_rectangles_only_touch() {
    let a = ScreenBounds::new(Vec2::ZERO, Vec2::new(1920.0, 1080.0));
    let b = ScreenBounds::new(Vec2::new(1920.0, 0.0), Vec2::new(3840.0, 1080.0));

    assert!(!a.intersects(&b));
}

#[test]
fn screen_bounds_does_not_intersect_when_rectangles_are_separated() {
    let a = ScreenBounds::new(Vec2::ZERO, Vec2::new(1920.0, 1080.0));
    let b = ScreenBounds::new(Vec2::new(2000.0, 0.0), Vec2::new(3920.0, 1080.0));

    assert!(!a.intersects(&b));
}

#[test]
fn adjacent_monitor_detects_partial_vertical_overlap() {
    let left = monitor(
        Vec2::ZERO,
        Vec2::new(1920.0, 1080.0),
        Vec2::ZERO,
        Vec2::new(1920.0, 1040.0),
    );
    let right = monitor(
        Vec2::new(1920.0, 200.0),
        Vec2::new(3840.0, 1280.0),
        Vec2::new(1920.0, 200.0),
        Vec2::new(3840.0, 1240.0),
    );

    let monitors = MonitorAreas::new(vec![left, right]);
    let current = &monitors.areas()[0];

    // Sprite's center in the left monitor
    // Sprite bottom still stand on right monitor
    let crossing_bounds = ScreenBounds::new(Vec2::new(1870.0, 150.0), Vec2::new(1970.0, 250.0));

    let adjacent = monitors.adjacent_monitor(current, crossing_bounds, Vec2::X);

    assert!(std::ptr::eq(adjacent.unwrap(), &monitors.areas()[1]));
}

#[test]
fn monitor_at_returns_matching_monitor_regardless_of_unrelated_monitor_order() {
    let left = monitor(
        Vec2::ZERO,
        Vec2::new(1920.0, 1080.0),
        Vec2::ZERO,
        Vec2::new(1920.0, 1040.0),
    );

    let right = monitor(
        Vec2::new(1920.0, 0.0),
        Vec2::new(3840.0, 1080.0),
        Vec2::new(1920.0, 0.0),
        Vec2::new(3840.0, 1040.0),
    );

    let unrelated = monitor(
        Vec2::new(4000.0, 0.0),
        Vec2::new(5920.0, 1080.0),
        Vec2::new(4000.0, 0.0),
        Vec2::new(5920.0, 1040.0),
    );

    let monitors = MonitorAreas::new(vec![unrelated, right, left]);

    let found = monitors.monitor_at(Vec2::new(2500.0, 500.0));

    assert!(std::ptr::eq(found.unwrap(), &monitors.areas()[1]));
}
