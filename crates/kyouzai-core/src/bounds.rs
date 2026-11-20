use glam::Vec2;

/// A rectangular region in desktop physical-pixel coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenBounds {
    pub min: Vec2,
    pub max: Vec2,
}

impl ScreenBounds {
    pub const fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn width(&self) -> f32 {
        (self.max.x - self.min.x).max(0.0)
    }

    pub fn height(&self) -> f32 {
        (self.max.y - self.min.y).max(0.0)
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x < self.max.x
            && point.y >= self.min.y
            && point.y < self.max.y
    }

    pub fn distance_squared_to(&self, point: Vec2) -> f32 {
        let closest = point.clamp(self.min, self.max);
        point.distance_squared(closest)
    }

    /// Returns whether two rectangles overlap with a non-zero area.
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }
}

/// The physical bounds and usable work area of one monitor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MonitorArea {
    pub bounds: ScreenBounds,
    pub work_area: ScreenBounds,
}

/// All monitors available on the desktop.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MonitorAreas {
    areas: Vec<MonitorArea>,
}

impl MonitorAreas {
    pub fn new(areas: Vec<MonitorArea>) -> Self {
        Self { areas }
    }

    pub fn areas(&self) -> &[MonitorArea] {
        &self.areas
    }

    pub fn is_empty(&self) -> bool {
        self.areas.is_empty()
    }

    /// Returns the monitor containing the given desktop position.
    pub fn monitor_at(&self, position: Vec2) -> Option<&MonitorArea> {
        self.areas
            .iter()
            .find(|monitor| monitor.bounds.contains(position))
    }

    /// Returns the nearest monitor to the given desktop position.
    pub fn nearest_monitor(&self, position: Vec2) -> Option<&MonitorArea> {
        self.areas.iter().min_by(|a, b| {
            a.bounds
                .distance_squared_to(position)
                .total_cmp(&b.bounds.distance_squared_to(position))
        })
    }

    /// Finds a monitor adjacent to the current monitor at the crossing position.
    pub fn adjacent_monitor(
        &self,
        current: &MonitorArea,
        crossing_bounds: ScreenBounds,
        direction: Vec2,
    ) -> Option<&MonitorArea> {
        if direction == Vec2::ZERO {
            return None;
        }

        let bounds = current.bounds;

        self.areas.iter().find(|monitor| {
            if std::ptr::eq(*monitor, current) {
                return false;
            }

            let other = monitor.bounds;

            if direction.x > 0.0 && direction.y == 0.0 {
                other.min.x == bounds.max.x
                    && crossing_bounds.min.y < other.max.y
                    && crossing_bounds.max.y > other.min.y
            } else if direction.x < 0.0 && direction.y == 0.0 {
                other.max.x == bounds.min.x
                    && crossing_bounds.min.y < other.max.y
                    && crossing_bounds.max.y > other.min.y
            } else if direction.y > 0.0 && direction.x == 0.0 {
                other.min.y == bounds.max.y
                    && crossing_bounds.min.x < other.max.x
                    && crossing_bounds.max.x > other.min.x
            } else if direction.y < 0.0 && direction.x == 0.0 {
                other.max.y == bounds.min.y
                    && crossing_bounds.min.x < other.max.x
                    && crossing_bounds.max.x > other.min.x
            } else {
                false
            }
        })
    }
}
