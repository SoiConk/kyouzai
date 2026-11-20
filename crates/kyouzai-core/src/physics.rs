use glam::Vec2;

pub const GRAVITY: f32 = 1200.0;

pub fn apply_gravity(velocity: &mut Vec2, dt_seconds: f32) {
    velocity.y += GRAVITY * dt_seconds;
}

pub fn resolve_ground(position: &mut Vec2, velocity: &mut Vec2, ground_y: f32) -> bool {
    if position.y < ground_y {
        return false;
    }

    position.y = ground_y;
    velocity.y = 0.0;
    true
}
