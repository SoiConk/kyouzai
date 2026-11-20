//! Schedules checks for fullscreen applications.
//!
//! This module only controls polling frequency. `Application` remains responsible
//! for interpreting the result and requesting companion state transitions.

use std::time::{Duration, Instant};

use kyouzai_platform::PlatformWindowExt;

// -----------------------------------------------------------------------------
// Configuration
// -----------------------------------------------------------------------------

/// Polling interval while the companion is awake.
const AWAKE_CHECK_INTERVAL: Duration = Duration::from_secs(2);

/// Polling interval while the companion is sleeping.
const ASLEEP_CHECK_INTERVAL: Duration = Duration::from_secs(5);

// -----------------------------------------------------------------------------
// Sleep manager
// -----------------------------------------------------------------------------

pub struct SleepManager {
    last_check: Instant,
}

impl SleepManager {
    pub fn new() -> Self {
        Self {
            last_check: Instant::now(),
        }
    }

    /// Checks whether another application is fullscreen if the polling interval
    /// has elapsed.
    ///
    /// Returns `None` when no check is due, or `Some(is_fullscreen)` otherwise.
    ///
    /// The sleeping state is supplied by the caller on every invocation rather
    /// than cached here. This ensures polling always follows the companion's
    /// actual state, including when a requested state transition is rejected.
    pub fn poll<P: PlatformWindowExt>(
        &mut self,
        now: Instant,
        is_currently_sleeping: bool,
        platform: &P,
    ) -> Option<bool> {
        if now.saturating_duration_since(self.last_check) < self.interval(is_currently_sleeping) {
            return None;
        }

        self.last_check = now;
        Some(platform.is_fullscreen_app_foreground())
    }

    /// Returns the number of seconds until the next scheduled check.
    ///
    /// The event loop can use this value to avoid sleeping past the next
    /// fullscreen check.
    pub fn seconds_until_next_check(&self, now: Instant, is_currently_sleeping: bool) -> f32 {
        self.interval(is_currently_sleeping)
            .saturating_sub(now.saturating_duration_since(self.last_check))
            .as_secs_f32()
    }

    fn interval(&self, is_currently_sleeping: bool) -> Duration {
        if is_currently_sleeping {
            ASLEEP_CHECK_INTERVAL
        } else {
            AWAKE_CHECK_INTERVAL
        }
    }
}
