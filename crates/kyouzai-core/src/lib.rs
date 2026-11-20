pub mod animation;
pub mod bounds;
pub mod companion;
pub mod config;
pub mod events;
pub mod physics;
pub mod sprite;
pub mod state_machine;
pub mod transform;

pub use animation::{AnimationClip, AnimationSet, Frame, Timeline};
pub use bounds::{MonitorArea, MonitorAreas, ScreenBounds};
pub use companion::{Companion, UpdateOutcome};
pub use config::{AnimationDef, ManifestError, PetManifest};
pub use events::CompanionEvent;
pub use sprite::{Sprite, SpriteRegion, TextureId};
pub use state_machine::{CompanionState, CompanionStateMachine, FALLING_STATE};
pub use transform::Transform;

// Unit tests live under `src/tests/`, one file per module, rather than inline
// `#[cfg(test)] mod tests { ... }` blocks — keeps implementation files reading as pure
// implementation. `#[path]` wires each file in as a real module of this crate (not a
// separate integration-test crate under `tests/` at the crate root, which would only see
// the crate's public API), so tests can still exercise anything at least `pub(crate)`.
#[cfg(test)]
#[path = "tests/animation_tests.rs"]
mod animation_tests;
#[cfg(test)]
#[path = "tests/bounds_tests.rs"]
mod bounds_tests;
#[cfg(test)]
#[path = "tests/companion_tests.rs"]
mod companion_tests;
#[cfg(test)]
#[path = "tests/config_tests.rs"]
mod config_tests;
#[cfg(test)]
#[path = "tests/physics_tests.rs"]
mod physics_tests;
#[cfg(test)]
#[path = "tests/sprite_tests.rs"]
mod sprite_tests;
#[cfg(test)]
#[path = "tests/state_machine_tests.rs"]
mod state_machine_tests;
#[cfg(test)]
#[path = "tests/transform_tests.rs"]
mod transform_tests;
