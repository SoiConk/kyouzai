//! Handles mouse interaction with the companion:
//! - Hit testing against the sprite's alpha mask.
//! - Tracking click-through state.
//! - Distinguishing clicks from drag gestures.
//!
//! Platform-specific input and click-through operations are handled by
//! `Application`, not by this module.

use glam::Vec2;
use kyouzai_core::{Companion, CompanionEvent, CompanionState};
use kyouzai_platform::MouseButton;

use crate::alpha_mask::AlphaMask;

// -----------------------------------------------------------------------------
// Configuration
// -----------------------------------------------------------------------------

/// Minimum cursor movement in physical pixels required to start a drag.
pub(crate) const DRAG_THRESHOLD_PX: f32 = 4.0;

/// Alpha values at or below this threshold are treated as transparent.
const ALPHA_HIT_THRESHOLD: u8 = 10;

// -----------------------------------------------------------------------------
// Input state and outcome
// -----------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct InputOutcome {
    pub redraw: bool,
    pub wake_requested: bool,
    pub events: Vec<CompanionEvent>,
}

pub(crate) struct InputContext<'a> {
    pub companion: &'a mut Companion,
    pub alpha_mask: Option<&'a AlphaMask>,
    pub window_center: Vec2,
    pub cursor: Option<Vec2>,
    pub global_cursor: Option<Vec2>,
}

pub struct InputController {
    cursor: Vec2,
    hovering_opaque: bool,
    press_origin: Option<Vec2>,
    grab_offset: Option<Vec2>,
}

impl InputController {
    pub fn new() -> Self {
        Self {
            cursor: Vec2::ZERO,
            hovering_opaque: false,
            press_origin: None,
            grab_offset: None,
        }
    }

    /// Returns whether the window should currently allow mouse events to pass through.
    pub fn should_be_click_through(&self) -> bool {
        !self.hovering_opaque
    }

    /// Clears hover and pending click state when the cursor leaves the window.
    pub fn handle_cursor_left(&mut self) {
        self.hovering_opaque = false;
        self.press_origin = None;
    }

    /// Handles a window-local cursor movement.
    ///
    /// A pending press becomes a drag once the cursor moves beyond the threshold.
    pub fn handle_cursor_moved(
        &mut self,
        position: Vec2,
        companion: &mut Companion,
        alpha_mask: Option<&AlphaMask>,
        window_center: Vec2,
        global_cursor: Option<Vec2>,
    ) -> InputOutcome {
        self.cursor = position;
        self.hovering_opaque = hit_test(companion, alpha_mask, position, window_center);

        let mut outcome = InputOutcome::default();

        if companion.is_dragging() {
            let cursor_screen = to_screen_space(companion, window_center, position, global_cursor);

            if let Some(offset) = self.grab_offset {
                companion.set_dragged_position(cursor_screen + offset);
            }

            outcome.redraw = true;
            return outcome;
        }

        if let Some(origin) = self.press_origin
            && origin.distance(position) >= DRAG_THRESHOLD_PX
        {
            outcome.events = companion.begin_drag();

            if outcome.events.contains(&CompanionEvent::DragStarted) {
                let cursor_screen =
                    to_screen_space(companion, window_center, position, global_cursor);

                if let Some(offset) = self.grab_offset {
                    companion.set_dragged_position(cursor_screen + offset);
                }

                outcome.redraw = true;
            } else {
                self.grab_offset = None;
            }

            self.press_origin = None;
        }

        outcome
    }

    /// Rechecks hover state using an independently polled screen-space cursor position.
    ///
    /// This is necessary because the operating system may stop sending cursor-move
    /// events while click-through is enabled. Polling allows the application to detect
    /// when the cursor moves back over an opaque pixel.
    ///
    /// Unlike `handle_cursor_moved`, this method never starts or updates a drag.
    pub(crate) fn poll_global_cursor(
        &mut self,
        position: Vec2,
        companion: &Companion,
        alpha_mask: Option<&AlphaMask>,
        window_center: Vec2,
    ) {
        self.hovering_opaque = hit_test(companion, alpha_mask, position, window_center);
    }

    /// Handles mouse-button transitions.
    ///
    /// Only the left button participates in click and drag interactions.
    pub fn handle_mouse_button(
        &mut self,
        button: MouseButton,
        pressed: bool,
        context: InputContext<'_>,
    ) -> InputOutcome {
        let InputContext {
            companion,
            alpha_mask,
            window_center,
            cursor,
            global_cursor,
        } = context;

        let mut outcome = InputOutcome::default();

        if button != MouseButton::Left {
            return outcome;
        }

        if pressed {
            let cursor_valid = if let Some(cursor) = cursor {
                self.cursor = cursor;
                self.hovering_opaque = hit_test(companion, alpha_mask, cursor, window_center);
                true
            } else {
                false
            };

            if companion.state_machine().current() == CompanionState::Sleep {
                outcome.wake_requested = cursor_valid && self.hovering_opaque;
                self.press_origin = None;
                self.grab_offset = None;
                return outcome;
            }

            if cursor_valid && self.hovering_opaque {
                self.press_origin = Some(self.cursor);

                let cursor_screen =
                    to_screen_space(companion, window_center, self.cursor, global_cursor);

                self.grab_offset = Some(companion.transform().position() - cursor_screen);
            } else {
                self.press_origin = None;
                self.grab_offset = None;
            }
        } else if companion.is_dragging() {
            self.press_origin = None;
            self.grab_offset = None;
            outcome.events = companion.end_drag();
            outcome.redraw = true;
        } else if self.press_origin.take().is_some() {
            self.grab_offset = None;
            outcome.events.push(CompanionEvent::Clicked);
            outcome.redraw = true;
        }

        outcome
    }
}

// -----------------------------------------------------------------------------
// Hit testing
// -----------------------------------------------------------------------------

/// Checks whether the cursor is over an opaque pixel of the companion's sprite.
fn hit_test(
    companion: &Companion,
    alpha_mask: Option<&AlphaMask>,
    cursor: Vec2,
    window_center: Vec2,
) -> bool {
    let Some(mask) = alpha_mask else {
        return false;
    };

    let Some(sprite) = companion.sprite() else {
        return false;
    };

    let Some(normalized) = companion
        .transform()
        .hit_test_normalized(cursor, window_center)
    else {
        return false;
    };

    let region = sprite.region();
    let u = lerp(region.uv_min()[0], region.uv_max()[0], normalized.x);
    let v = lerp(region.uv_min()[1], region.uv_max()[1], normalized.y);

    mask.sample(u, v) > ALPHA_HIT_THRESHOLD
}

// -----------------------------------------------------------------------------
// Coordinate conversion
// -----------------------------------------------------------------------------

/// Converts a window-local cursor position into desktop screen coordinates.
///
/// Prefers the current global cursor position when available. This avoids using
/// potentially stale window-local events during fast window movement.
///
/// The fallback reconstructs screen coordinates from the window's current position.
/// It is best-effort and may drift when processing delayed cursor events.
fn to_screen_space(
    companion: &Companion,
    window_center: Vec2,
    window_local: Vec2,
    global_cursor: Option<Vec2>,
) -> Vec2 {
    global_cursor
        .unwrap_or_else(|| (companion.transform().position() - window_center) + window_local)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
