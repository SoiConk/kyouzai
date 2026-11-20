use std::time::Duration;

use glam::Vec2;

use crate::animation::{AnimationClip, AnimationSet, Frame};
use crate::bounds::{MonitorArea, MonitorAreas, ScreenBounds};
use crate::companion::{Companion, Facing};
use crate::events::CompanionEvent;
use crate::sprite::TextureId;
use crate::state_machine::{CompanionState, FALLING_STATE};
use crate::transform::Transform;

fn one_frame_clip() -> AnimationClip {
    AnimationClip {
        frames: vec![Frame {
            uv_min: [0.0, 0.0],
            uv_max: [1.0, 1.0],
            duration_seconds: 1.0,
        }],
        looped: true,
    }
}

fn test_animations() -> AnimationSet {
    let mut set = AnimationSet::default();
    set.insert(CompanionState::Idle, one_frame_clip());
    set.insert(CompanionState::Walk, one_frame_clip());
    set.insert(CompanionState::Sleep, one_frame_clip());
    set.insert(CompanionState::Grabbed, one_frame_clip());
    set
}

fn test_companion() -> Companion {
    test_companion_with_seed(42)
}

fn test_companion_with_seed(seed: u64) -> Companion {
    let transform = Transform::new(Vec2::new(500.0, 500.0), Vec2::new(100.0, 100.0));

    Companion::new_with_seed(transform, TextureId::new(1), test_animations(), 100.0, seed)
}

fn test_monitors(bounds: ScreenBounds) -> MonitorAreas {
    MonitorAreas::new(vec![MonitorArea {
        bounds,
        work_area: bounds,
    }])
}

fn test_monitor(min: Vec2, max: Vec2, work_min: Vec2, work_max: Vec2) -> MonitorArea {
    MonitorArea {
        bounds: ScreenBounds::new(min, max),
        work_area: ScreenBounds::new(work_min, work_max),
    }
}

/// Enters Walk, initializes its autonomous phase, then overrides its direction
/// so movement tests do not depend on random output.
fn start_walking(companion: &mut Companion, monitors: &MonitorAreas, direction: f32) {
    companion.request_state(CompanionState::Walk);

    // The first update initializes the autonomous Walk phase.
    companion.update(Duration::ZERO, monitors);

    companion.set_movement_direction(direction);
    companion.set_facing(if direction < 0.0 {
        Facing::Left
    } else {
        Facing::Right
    });
}

#[test]
fn new_companion_starts_idle() {
    let companion = test_companion();

    assert_eq!(companion.state_machine().current(), CompanionState::Idle);
}

#[test]
fn new_companion_with_same_seed_chooses_same_autonomous_direction() {
    let mut first = test_companion_with_seed(1234);
    let mut second = test_companion_with_seed(1234);

    let monitors = test_monitors(ScreenBounds::new(Vec2::ZERO, Vec2::new(1000.0, 600.0)));

    first.request_state(CompanionState::Walk);
    second.request_state(CompanionState::Walk);

    // Direction is not randomized by request_state().
    assert_eq!(first.movement_direction(), 1.0);
    assert_eq!(second.movement_direction(), 1.0);

    // Random direction is selected when autonomous Walk is initialized.
    first.update(Duration::ZERO, &monitors);
    second.update(Duration::ZERO, &monitors);

    assert_eq!(first.movement_direction(), second.movement_direction());
    assert_eq!(first.facing(), second.facing());

    assert!(first.movement_direction() == -1.0 || first.movement_direction() == 1.0);

    assert_eq!(
        first.facing(),
        if first.movement_direction() < 0.0 {
            Facing::Left
        } else {
            Facing::Right
        }
    );
}

#[test]
fn cannot_skip_the_falling_state_when_ending_a_drag_prematurely() {
    let mut companion = test_companion();

    // Attempting Idle directly (without ever grabbing) should simply be rejected.
    assert!(companion.request_state(CompanionState::Idle).is_empty());
    assert_eq!(companion.state_machine().current(), CompanionState::Idle);
}

#[test]
fn drag_lifecycle_goes_through_falling_before_idle() {
    let mut companion = test_companion();

    let events = companion.begin_drag();
    assert_eq!(companion.state_machine().current(), CompanionState::Grabbed);
    assert!(events.contains(&CompanionEvent::DragStarted));

    let events = companion.end_drag();
    assert_eq!(
        companion.state_machine().current(),
        CompanionState::Custom(FALLING_STATE)
    );
    assert!(events.contains(&CompanionEvent::DragEnded));

    // Physics has to actually land it before Idle becomes reachable again.
    let monitors = test_monitors(ScreenBounds::new(Vec2::ZERO, Vec2::new(1000.0, 600.0)));

    for _ in 0..600 {
        companion.update(Duration::from_millis(16), &monitors);
    }

    assert_eq!(companion.state_machine().current(), CompanionState::Idle);
}

#[test]
fn walking_bounces_off_the_right_edge_of_the_screen() {
    let mut companion = test_companion();

    let monitors = test_monitors(ScreenBounds::new(Vec2::ZERO, Vec2::new(550.0, 600.0)));

    start_walking(&mut companion, &monitors, 1.0);

    for _ in 0..200 {
        companion.update(Duration::from_millis(16), &monitors);
    }

    let half_w = companion.transform().size().x * 0.5;

    assert!(
        companion.transform().position().x <= monitors.areas()[0].work_area.max.x - half_w + 0.01
    );
}

#[test]
fn set_dragged_position_is_ignored_outside_grabbed() {
    let mut companion = test_companion();
    let before = companion.transform().position();

    companion.set_dragged_position(Vec2::new(999.0, 999.0));

    assert_eq!(companion.transform().position(), before);
}

#[test]
fn falling_lands_on_a_lower_adjacent_monitor() {
    let mut companion = test_companion();

    companion
        .transform_mut()
        .set_position(Vec2::new(2500.0, 500.0));

    companion.end_drag();

    let monitors = MonitorAreas::new(vec![
        test_monitor(
            Vec2::ZERO,
            Vec2::new(1920.0, 1080.0),
            Vec2::ZERO,
            Vec2::new(1920.0, 1040.0),
        ),
        test_monitor(
            Vec2::new(1920.0, 200.0),
            Vec2::new(3840.0, 1280.0),
            Vec2::new(1920.0, 200.0),
            Vec2::new(3840.0, 1240.0),
        ),
    ]);

    for _ in 0..600 {
        companion.update(Duration::from_millis(16), &monitors);

        if companion.state_machine().current() == CompanionState::Idle {
            break;
        }
    }

    assert_eq!(companion.state_machine().current(), CompanionState::Idle);
    assert!((companion.transform().position().y - 1190.0).abs() < 0.01);
}

#[test]
fn falling_does_not_land_in_a_gap_between_monitors() {
    let mut companion = test_companion();

    companion
        .transform_mut()
        .set_position(Vec2::new(2010.0, 500.0));

    companion.end_drag();

    let monitors = MonitorAreas::new(vec![
        test_monitor(
            Vec2::ZERO,
            Vec2::new(1920.0, 1080.0),
            Vec2::ZERO,
            Vec2::new(1920.0, 1040.0),
        ),
        test_monitor(
            Vec2::new(2100.0, 0.0),
            Vec2::new(4020.0, 1080.0),
            Vec2::new(2100.0, 0.0),
            Vec2::new(4020.0, 1040.0),
        ),
    ]);

    for _ in 0..120 {
        companion.update(Duration::from_millis(16), &monitors);
    }

    assert_eq!(
        companion.state_machine().current(),
        CompanionState::Custom(FALLING_STATE)
    );
}

#[test]
fn walking_moves_to_adjacent_monitor_on_the_right() {
    let mut companion = test_companion();

    companion
        .transform_mut()
        .set_position(Vec2::new(1800.0, 500.0));

    let monitors = MonitorAreas::new(vec![
        test_monitor(
            Vec2::ZERO,
            Vec2::new(1920.0, 1080.0),
            Vec2::ZERO,
            Vec2::new(1920.0, 1040.0),
        ),
        test_monitor(
            Vec2::new(1920.0, 0.0),
            Vec2::new(3840.0, 1080.0),
            Vec2::new(1920.0, 0.0),
            Vec2::new(3840.0, 1040.0),
        ),
    ]);

    start_walking(&mut companion, &monitors, 1.0);

    for _ in 0..100 {
        companion.update(Duration::from_millis(16), &monitors);
    }

    assert!(
        companion.transform().position().x > 1920.0,
        "Companion should have crossed onto the right monitor"
    );
}

#[test]
fn walking_moves_to_adjacent_monitor_on_the_left() {
    let mut companion = test_companion();

    let monitors = MonitorAreas::new(vec![
        test_monitor(
            Vec2::ZERO,
            Vec2::new(1920.0, 1080.0),
            Vec2::ZERO,
            Vec2::new(1920.0, 1040.0),
        ),
        test_monitor(
            Vec2::new(1920.0, 0.0),
            Vec2::new(3840.0, 1080.0),
            Vec2::new(1920.0, 0.0),
            Vec2::new(3840.0, 1040.0),
        ),
    ]);

    start_walking(&mut companion, &monitors, -1.0);

    companion
        .transform_mut()
        .set_position(Vec2::new(2020.0, 500.0));

    for _ in 0..100 {
        companion.update(Duration::from_millis(16), &monitors);
    }

    assert!(
        companion.transform().position().x < 1920.0,
        "Companion should have crossed onto the left monitor"
    );
}

#[test]
fn walking_falls_after_leaving_a_higher_monitor() {
    let monitors = MonitorAreas::new(vec![
        test_monitor(
            Vec2::new(0.0, 0.0),
            Vec2::new(1920.0, 1080.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(1920.0, 1080.0),
        ),
        test_monitor(
            Vec2::new(1920.0, 300.0),
            Vec2::new(3840.0, 1380.0),
            Vec2::new(1920.0, 300.0),
            Vec2::new(3840.0, 1380.0),
        ),
    ]);

    let mut companion = test_companion();

    start_walking(&mut companion, &monitors, 1.0);

    companion
        .transform_mut()
        .set_position(Vec2::new(1800.0, 1030.0));

    for _ in 0..120 {
        companion.update(Duration::from_millis(16), &monitors);
    }

    assert_eq!(
        companion.state_machine().current(),
        CompanionState::Custom(FALLING_STATE)
    );
}

#[test]
fn falling_lands_on_lower_ground_without_snapping_to_higher_ground() {
    let monitors = MonitorAreas::new(vec![
        test_monitor(
            Vec2::ZERO,
            Vec2::new(1920.0, 1080.0),
            Vec2::ZERO,
            Vec2::new(1920.0, 1040.0),
        ),
        test_monitor(
            Vec2::new(1920.0, 200.0),
            Vec2::new(3840.0, 1280.0),
            Vec2::new(1920.0, 200.0),
            Vec2::new(3840.0, 1240.0),
        ),
    ]);

    let mut companion = test_companion();

    // Sprite still overlaps the higher monitor horizontally.
    companion
        .transform_mut()
        .set_position(Vec2::new(1900.0, 1100.0));

    companion.end_drag();

    for _ in 0..600 {
        companion.update(Duration::from_millis(16), &monitors);
    }

    assert_eq!(companion.state_machine().current(), CompanionState::Idle);
    assert!(
        (companion.transform().position().y - 1190.0).abs() < 0.01,
        "Companion should land on the lower monitor, not snap to the higher one"
    );
}

#[test]
fn walking_and_falling_across_three_monitors() {
    let monitors = MonitorAreas::new(vec![
        test_monitor(
            Vec2::new(0.0, 0.0),
            Vec2::new(1920.0, 1080.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(1920.0, 1040.0),
        ),
        test_monitor(
            Vec2::new(1920.0, 200.0),
            Vec2::new(3840.0, 1280.0),
            Vec2::new(1920.0, 200.0),
            Vec2::new(3840.0, 1240.0),
        ),
        test_monitor(
            Vec2::new(3840.0, 400.0),
            Vec2::new(5760.0, 1480.0),
            Vec2::new(3840.0, 400.0),
            Vec2::new(5760.0, 1440.0),
        ),
    ]);

    let transform = Transform::new(Vec2::new(1800.0, 500.0), Vec2::new(100.0, 100.0));

    let mut companion =
        Companion::new_with_seed(transform, TextureId::new(1), test_animations(), 1000.0, 42);

    // A -> B
    start_walking(&mut companion, &monitors, 1.0);

    for _ in 0..20 {
        companion.update(Duration::from_millis(16), &monitors);
    }

    assert_eq!(
        companion.state_machine().current(),
        CompanionState::Custom(FALLING_STATE)
    );

    for _ in 0..120 {
        companion.update(Duration::from_millis(16), &monitors);
    }

    assert_eq!(companion.state_machine().current(), CompanionState::Idle);
    assert!((companion.transform().position().y - 1190.0).abs() < 0.01);

    // B -> C
    start_walking(&mut companion, &monitors, 1.0);

    for _ in 0..120 {
        companion.update(Duration::from_millis(16), &monitors);
    }

    assert_eq!(
        companion.state_machine().current(),
        CompanionState::Custom(FALLING_STATE)
    );

    for _ in 0..140 {
        companion.update(Duration::from_millis(16), &monitors);
    }

    assert_eq!(companion.state_machine().current(), CompanionState::Idle);
    assert!((companion.transform().position().y - 1390.0).abs() < 0.01);
}

#[test]
fn walking_crosses_partially_overlapping_monitors_and_falls() {
    let monitors = MonitorAreas::new(vec![
        test_monitor(
            Vec2::new(0.0, 0.0),
            Vec2::new(1920.0, 1080.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(1920.0, 1040.0),
        ),
        test_monitor(
            Vec2::new(1920.0, 700.0),
            Vec2::new(3840.0, 1780.0),
            Vec2::new(1920.0, 700.0),
            Vec2::new(3840.0, 1740.0),
        ),
    ]);

    let transform = Transform::new(Vec2::new(1800.0, 1000.0), Vec2::new(100.0, 100.0));

    let mut companion =
        Companion::new_with_seed(transform, TextureId::new(1), test_animations(), 1000.0, 42);

    start_walking(&mut companion, &monitors, 1.0);

    for _ in 0..20 {
        companion.update(Duration::from_millis(16), &monitors);
    }

    assert!(companion.transform().position().x > 1920.0);

    assert_eq!(
        companion.state_machine().current(),
        CompanionState::Custom(FALLING_STATE)
    );

    for _ in 0..300 {
        companion.update(Duration::from_millis(16), &monitors);
    }

    assert_eq!(companion.state_machine().current(), CompanionState::Idle);

    assert!((companion.transform().position().y - 1690.0).abs() < 0.01);
}

#[test]
fn walking_does_not_cross_a_gap_between_work_areas() {
    let monitors = MonitorAreas::new(vec![
        test_monitor(
            Vec2::ZERO,
            Vec2::new(1920.0, 1080.0),
            Vec2::ZERO,
            Vec2::new(1820.0, 1040.0),
        ),
        test_monitor(
            Vec2::new(1920.0, 0.0),
            Vec2::new(3840.0, 1080.0),
            Vec2::new(2020.0, 0.0),
            Vec2::new(3840.0, 1040.0),
        ),
    ]);

    let transform = Transform::new(Vec2::new(1750.0, 1000.0), Vec2::new(100.0, 100.0));

    let mut companion =
        Companion::new_with_seed(transform, TextureId::new(1), test_animations(), 1000.0, 42);

    start_walking(&mut companion, &monitors, 1.0);

    for _ in 0..100 {
        companion.update(Duration::from_millis(16), &monitors);
    }

    assert!(
        companion.transform().position().x <= 1870.0 + 0.01,
        "Companion should not walk beyond the right edge of its work area"
    );

    assert_eq!(companion.state_machine().current(), CompanionState::Walk);
}

#[test]
fn walking_falls_when_entering_a_higher_monitor() {
    let monitors = MonitorAreas::new(vec![
        test_monitor(
            Vec2::new(0.0, 0.0),
            Vec2::new(1920.0, 1080.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(1920.0, 1040.0),
        ),
        test_monitor(
            Vec2::new(1920.0, -100.0),
            Vec2::new(3840.0, 980.0),
            Vec2::new(1920.0, -100.0),
            Vec2::new(3840.0, 940.0),
        ),
    ]);

    let transform = Transform::new(Vec2::new(1800.0, 990.0), Vec2::new(100.0, 100.0));

    let mut companion =
        Companion::new_with_seed(transform, TextureId::new(1), test_animations(), 1000.0, 42);

    start_walking(&mut companion, &monitors, 1.0);

    for _ in 0..20 {
        companion.update(Duration::from_millis(16), &monitors);
    }

    assert!(
        companion.transform().position().x > 1920.0,
        "Companion should cross the monitor boundary"
    );

    assert_eq!(
        companion.state_machine().current(),
        CompanionState::Custom(FALLING_STATE),
        "Companion should fall instead of climbing onto the higher monitor"
    );
}

#[test]
fn next_update_uses_animation_deadline_before_autonomy_is_initialized() {
    let companion = test_companion();

    assert_eq!(
        companion.seconds_until_next_update(),
        companion.seconds_until_next_frame()
    );
}

#[test]
fn next_update_uses_earliest_idle_animation_or_phase_deadline() {
    let mut companion = test_companion();
    let monitors = test_monitors(ScreenBounds::new(Vec2::ZERO, Vec2::new(1000.0, 600.0)));

    companion.update(Duration::ZERO, &monitors);

    let animation_wait = companion.seconds_until_next_frame().unwrap();
    let update_wait = companion.seconds_until_next_update().unwrap();

    assert_eq!(update_wait, animation_wait);
}

#[test]
fn next_update_uses_earliest_walk_animation_or_phase_deadline() {
    let mut companion = test_companion();
    let monitors = test_monitors(ScreenBounds::new(Vec2::ZERO, Vec2::new(1000.0, 600.0)));

    start_walking(&mut companion, &monitors, 1.0);

    let animation_wait = companion.seconds_until_next_frame().unwrap();
    let update_wait = companion.seconds_until_next_update().unwrap();

    assert_eq!(update_wait, animation_wait);
}

#[test]
fn next_update_ignores_autonomy_deadline_while_sleeping() {
    let mut companion = test_companion();

    companion.request_state(CompanionState::Sleep);

    assert_eq!(
        companion.seconds_until_next_update(),
        companion.seconds_until_next_frame()
    );
}
