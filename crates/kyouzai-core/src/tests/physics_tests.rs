use crate::physics::*;
use glam::Vec2;

#[test]
fn gravity_increases_downward_speed_over_time() {
    let mut velocity = Vec2::ZERO;

    apply_gravity(&mut velocity, 0.5);

    assert_eq!(velocity.y, GRAVITY * 0.5);
}

#[test]
fn resolve_ground_does_nothing_while_above_ground() {
    let mut position = Vec2::new(0.0, 10.0);
    let mut velocity = Vec2::new(0.0, 500.0);

    let landed = resolve_ground(&mut position, &mut velocity, 100.0);

    assert!(!landed);
    assert_eq!(position.y, 10.0);
    assert_eq!(velocity.y, 500.0);
}

#[test]
fn resolve_ground_stops_velocity_on_impact() {
    let mut position = Vec2::new(0.0, 150.0);
    let mut velocity = Vec2::new(0.0, 1000.0);

    let landed = resolve_ground(&mut position, &mut velocity, 100.0);

    assert!(landed);
    assert_eq!(position.y, 100.0);
    assert_eq!(velocity, Vec2::ZERO);
}

#[test]
fn resolve_ground_stops_velocity_on_slow_impact() {
    let mut position = Vec2::new(0.0, 100.0);
    let mut velocity = Vec2::new(0.0, 10.0);

    let landed = resolve_ground(&mut position, &mut velocity, 100.0);

    assert!(landed);
    assert_eq!(position.y, 100.0);
    assert_eq!(velocity, Vec2::ZERO);
}
