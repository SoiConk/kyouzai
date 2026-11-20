use std::time::Duration;

use glam::Vec2;

use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

use crate::animation::{AnimationSet, Timeline};
use crate::bounds::{MonitorAreas, ScreenBounds};
use crate::events::CompanionEvent;
use crate::physics;
use crate::sprite::{Sprite, SpriteRegion, TextureId};
use crate::state_machine::{CompanionState, CompanionStateMachine, FALLING_STATE};
use crate::transform::Transform;

/// How long the built-in placeholder Idle/Walk toggle stays in each state.
const IDLE_DURATION_RANGE: (f32, f32) = (4.0, 8.0);
const WALK_DURATION_RANGE: (f32, f32) = (2.0, 6.0);

/// Direction the companion faces visually.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Facing {
    Left,
    Right,
}

/// Result of one `Companion::update` tick.
#[derive(Debug, Clone, Default)]
pub struct UpdateOutcome {
    /// Whether the visible sprite frame changed and a redraw should be requested.
    pub redraw_needed: bool,
    /// Notable things that happened this tick.
    pub events: Vec<CompanionEvent>,
}

#[derive(Debug)]
pub struct Companion {
    transform: Transform,
    state_machine: CompanionStateMachine,
    sprite: Option<Sprite>,
    animations: AnimationSet,
    timeline: Timeline,
    /// Only meaningful while falling (`Custom(FALLING_STATE)`); pixels/second.
    velocity: Vec2,
    ground_y: Option<f32>,
    /// Horizontal movement direction: -1.0 (left), 1.0 (right).
    movement_direction: f32,
    /// Visual orientation, independent of movement direction.
    facing: Facing,
    walk_speed: f32,
    autonomy_timer: f32,
    /// Random number generator used for autonomous behavior.
    rng: StdRng,
    /// Duration of the current autonomous phase, in seconds.
    /// `None` means the autonomous behavior has not initialized this phase yet.
    phase_duration: Option<f32>,
}

impl Companion {
    pub fn new(
        transform: Transform,
        texture: TextureId,
        animations: AnimationSet,
        walk_speed: f32,
    ) -> Self {
        let rng: StdRng = rand::make_rng();

        Self::with_rng(transform, texture, animations, walk_speed, rng)
    }

    /// Creates a companion with a deterministic random seed.
    pub fn new_with_seed(
        transform: Transform,
        texture: TextureId,
        animations: AnimationSet,
        walk_speed: f32,
        seed: u64,
    ) -> Self {
        Self::with_rng(
            transform,
            texture,
            animations,
            walk_speed,
            StdRng::seed_from_u64(seed),
        )
    }

    fn with_rng(
        transform: Transform,
        texture: TextureId,
        animations: AnimationSet,
        walk_speed: f32,
        rng: StdRng,
    ) -> Self {
        let mut companion = Self {
            transform,
            state_machine: CompanionStateMachine::default(),
            sprite: Some(Sprite::new(texture)),
            animations,
            timeline: Timeline::default(),
            velocity: Vec2::ZERO,
            ground_y: None,
            movement_direction: 1.0,
            facing: Facing::Right,
            walk_speed,
            autonomy_timer: 0.0,
            rng,
            phase_duration: None,
        };

        companion.sync_sprite_region();
        companion
    }

    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    pub fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    pub fn state_machine(&self) -> &CompanionStateMachine {
        &self.state_machine
    }

    pub fn sprite(&self) -> Option<&Sprite> {
        self.sprite.as_ref()
    }

    pub fn is_dragging(&self) -> bool {
        self.state_machine.current() == CompanionState::Grabbed
    }

    pub fn movement_direction(&self) -> f32 {
        self.movement_direction
    }

    pub fn facing(&self) -> Facing {
        self.facing
    }

    /// Sets the horizontal movement direction without changing visual orientation.
    ///
    /// Negative values mean left; positive values mean right.
    /// Zero and non-finite values are ignored.
    pub fn set_movement_direction(&mut self, direction: f32) {
        if !direction.is_finite() || direction == 0.0 {
            return;
        }

        self.movement_direction = direction.signum();
    }

    /// Sets the visual orientation independently of movement direction.
    pub fn set_facing(&mut self, facing: Facing) {
        if self.facing == facing {
            return;
        }

        self.facing = facing;
        self.sync_sprite_region();
    }

    /// Attempts to move to `next`, enforcing `CompanionStateMachine::can_transition`.
    /// Returns the events produced (empty if the transition was illegal or a no-op).
    pub fn request_state(&mut self, next: CompanionState) -> Vec<CompanionEvent> {
        if !self.state_machine.can_transition(next) {
            return Vec::new();
        }

        let previous_key = self.state_machine.current().animation_key();
        self.state_machine.transition(next);

        if previous_key != next.animation_key() {
            self.timeline.reset();
        }

        // Invalidate the previous autonomous phase.
        // The autonomous behavior initializes a new phase when it is next used.
        self.autonomy_timer = 0.0;
        self.phase_duration = None;

        match next {
            CompanionState::Grabbed => {
                self.velocity = Vec2::ZERO;
                self.ground_y = None;
            }
            CompanionState::Custom(id) if id == FALLING_STATE => {
                self.ground_y = None;
            }
            _ => {}
        }

        vec![CompanionEvent::StateChanged]
    }

    /// Starts a drag after the mouse-down has been confirmed on an opaque sprite pixel.
    pub fn begin_drag(&mut self) -> Vec<CompanionEvent> {
        let mut events = self.request_state(CompanionState::Grabbed);

        if !events.is_empty() {
            events.push(CompanionEvent::DragStarted);
        }

        events
    }

    /// Ends a drag and starts falling/landing physics.
    pub fn end_drag(&mut self) -> Vec<CompanionEvent> {
        let mut events = self.request_state(CompanionState::Custom(FALLING_STATE));

        if !events.is_empty() {
            events.push(CompanionEvent::DragEnded);
        }

        events
    }

    pub fn set_dragged_position(&mut self, position: Vec2) {
        if self.state_machine.current() == CompanionState::Grabbed {
            self.transform.set_position(position);
        }
    }

    pub fn seconds_until_next_frame(&self) -> Option<f32> {
        let clip = self.animations.get(self.state_machine.current())?;
        let frame = clip.frames.get(self.timeline.frame_index())?;

        Some((frame.duration_seconds - self.timeline.elapsed_seconds()).max(0.0))
    }

    /// Returns the time until the next animation frame or autonomous phase transition.
    ///
    /// Returns `None` when neither has a scheduled update.
    pub fn seconds_until_next_update(&self) -> Option<f32> {
        let animation_wait = self.seconds_until_next_frame();

        let autonomy_wait = match self.state_machine.current() {
            CompanionState::Idle | CompanionState::Walk => self
                .phase_duration
                .map(|duration| (duration - self.autonomy_timer).max(0.0)),
            _ => None,
        };

        match (animation_wait, autonomy_wait) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }

    pub fn update(&mut self, dt: Duration, monitors: &MonitorAreas) -> UpdateOutcome {
        let dt_secs = dt.as_secs_f32();
        let mut events = Vec::new();

        match self.state_machine.current() {
            CompanionState::Idle => {
                self.advance_autonomy(dt_secs, CompanionState::Walk, &mut events);
            }
            CompanionState::Walk => {
                self.advance_autonomy(dt_secs, CompanionState::Idle, &mut events);

                if self.state_machine.current() == CompanionState::Walk {
                    self.step_walk(dt_secs, monitors, &mut events);
                }
            }
            CompanionState::Sleep => {}
            CompanionState::Grabbed => {}
            CompanionState::Custom(id) if id == FALLING_STATE => {
                self.step_falling(dt_secs, monitors, &mut events);
            }
            CompanionState::Custom(_) => {}
        }

        let frame_changed = self.advance_timeline(dt_secs);
        self.sync_sprite_region();

        UpdateOutcome {
            redraw_needed: frame_changed,
            events,
        }
    }

    /// Initializes the autonomous phase corresponding to the current state.
    ///
    /// Random values are generated only when autonomous behavior actually
    /// starts using an Idle or Walk phase.
    fn begin_autonomous_phase(&mut self) {
        self.autonomy_timer = 0.0;

        match self.state_machine.current() {
            CompanionState::Idle => {
                self.phase_duration = Some(
                    self.rng
                        .random_range(IDLE_DURATION_RANGE.0..=IDLE_DURATION_RANGE.1),
                );
            }
            CompanionState::Walk => {
                self.phase_duration = Some(
                    self.rng
                        .random_range(WALK_DURATION_RANGE.0..=WALK_DURATION_RANGE.1),
                );

                let direction = if self.rng.random_bool(0.5) { -1.0 } else { 1.0 };

                self.set_movement_direction(direction);
                self.set_facing(if direction < 0.0 {
                    Facing::Left
                } else {
                    Facing::Right
                });
            }
            _ => {
                self.phase_duration = None;
            }
        }
    }

    fn advance_autonomy(
        &mut self,
        dt_secs: f32,
        next: CompanionState,
        events: &mut Vec<CompanionEvent>,
    ) {
        if self.phase_duration.is_none() {
            self.begin_autonomous_phase();
        }

        let Some(phase_duration) = self.phase_duration else {
            return;
        };

        self.autonomy_timer += dt_secs;

        if self.autonomy_timer < phase_duration {
            return;
        }

        let transition_events = self.request_state(next);

        if transition_events.is_empty() {
            return;
        }

        events.extend(transition_events);

        // The transition succeeded, so autonomous behavior can begin
        // the next phase and make its random decisions.
        self.begin_autonomous_phase();
    }

    fn step_walk(
        &mut self,
        dt_secs: f32,
        monitors: &MonitorAreas,
        events: &mut Vec<CompanionEvent>,
    ) {
        let mut position = self.transform.position();
        let half_size = self.transform.size() * 0.5;

        // Acquire the initial ground when walking starts.
        if self.ground_y.is_none() {
            let Some(monitor) = monitors.monitor_at(position) else {
                events.extend(self.request_state(CompanionState::Custom(FALLING_STATE)));
                return;
            };

            let ground_y = monitor.work_area.max.y;
            self.ground_y = Some(ground_y);
            position.y = ground_y - half_size.y;
        }

        let ground_y = self.ground_y.unwrap();

        // Find a monitor that supports the Companion at the current ground height.
        let support_monitor = monitors.areas().iter().find(|monitor| {
            (monitor.work_area.max.y - ground_y).abs() <= 1.0
                && position.x + half_size.x > monitor.work_area.min.x
                && position.x - half_size.x < monitor.work_area.max.x
        });

        let Some(monitor) = support_monitor else {
            self.ground_y = None;
            events.extend(self.request_state(CompanionState::Custom(FALLING_STATE)));
            return;
        };

        let direction = self.movement_direction;

        let mut next_position = position;
        next_position.x += self.walk_speed * dt_secs * direction;

        let min_x = monitor.work_area.min.x + half_size.x;
        let max_x = monitor.work_area.max.x - half_size.x;

        if min_x > max_x {
            return;
        }

        if direction < 0.0 && next_position.x <= min_x {
            let can_cross = monitor.work_area.min.x <= monitor.bounds.min.x;

            let crossing_position = next_position + Vec2::NEG_X;
            let crossing_bounds =
                ScreenBounds::new(crossing_position - half_size, crossing_position + half_size);

            let adjacent = can_cross
                .then(|| monitors.adjacent_monitor(monitor, crossing_bounds, Vec2::NEG_X))
                .flatten();

            if adjacent.is_none() {
                next_position.x = min_x;
                self.set_movement_direction(1.0);
                self.set_facing(Facing::Right);
            }
        } else if direction > 0.0 && next_position.x >= max_x {
            let can_cross = monitor.work_area.max.x >= monitor.bounds.max.x;

            let crossing_position = next_position + Vec2::X;
            let crossing_bounds =
                ScreenBounds::new(crossing_position - half_size, crossing_position + half_size);

            let adjacent = can_cross
                .then(|| monitors.adjacent_monitor(monitor, crossing_bounds, Vec2::X))
                .flatten();

            if adjacent.is_none() {
                next_position.x = max_x;
                self.set_movement_direction(-1.0);
                self.set_facing(Facing::Left);
            }
        }

        next_position.y = ground_y - half_size.y;
        self.transform.set_position(next_position);

        // Check whether any monitor still supports the Companion.
        let left = next_position.x - half_size.x;
        let right = next_position.x + half_size.x;

        let has_support = monitors.areas().iter().any(|monitor| {
            (monitor.work_area.max.y - ground_y).abs() <= 1.0
                && right > monitor.work_area.min.x
                && left < monitor.work_area.max.x
        });

        if !has_support {
            self.ground_y = None;
            events.extend(self.request_state(CompanionState::Custom(FALLING_STATE)));
        }
    }

    fn step_falling(
        &mut self,
        dt_secs: f32,
        monitors: &MonitorAreas,
        events: &mut Vec<CompanionEvent>,
    ) {
        let previous_position = self.transform.position();
        let half_size = self.transform.size() * 0.5;

        physics::apply_gravity(&mut self.velocity, dt_secs);

        let mut position = previous_position + self.velocity * dt_secs;

        let previous_bottom = previous_position.y + half_size.y;
        let next_bottom = position.y + half_size.y;

        let landing = monitors
            .areas()
            .iter()
            .filter_map(|monitor| {
                let ground_y = monitor.work_area.max.y;

                let crossed_ground = previous_bottom <= ground_y && next_bottom >= ground_y;

                let overlaps_horizontally = position.x + half_size.x > monitor.work_area.min.x
                    && position.x - half_size.x < monitor.work_area.max.x;

                (crossed_ground && overlaps_horizontally).then_some(ground_y)
            })
            .min_by(f32::total_cmp);

        let landing = landing.or_else(|| {
            let monitor = monitors.monitor_at(position)?;

            let ground_y = monitor.work_area.max.y;
            let overlaps_horizontally = position.x + half_size.x > monitor.work_area.min.x
                && position.x - half_size.x < monitor.work_area.max.x;

            (overlaps_horizontally && previous_bottom > ground_y).then_some(ground_y)
        });

        if let Some(ground_y) = landing {
            position.y = ground_y - half_size.y;
            self.velocity.y = 0.0;
            events.extend(self.request_state(CompanionState::Idle));
        }

        self.transform.set_position(position);
    }

    fn advance_timeline(&mut self, dt_secs: f32) -> bool {
        let Some(clip) = self.animations.get(self.state_machine.current()) else {
            return false;
        };

        self.timeline.advance(clip, dt_secs)
    }

    fn sync_sprite_region(&mut self) {
        let Some(clip) = self.animations.get(self.state_machine.current()) else {
            return;
        };

        let Some(frame) = clip.frames.get(self.timeline.frame_index()) else {
            return;
        };

        let mut uv_min = frame.uv_min;
        let mut uv_max = frame.uv_max;

        if self.facing == Facing::Left {
            std::mem::swap(&mut uv_min[0], &mut uv_max[0]);
        }

        if let Some(sprite) = self.sprite.as_mut() {
            sprite.set_region(SpriteRegion::new(uv_min, uv_max));
        }
    }
}
