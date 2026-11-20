//! Parsing for `pet.json` pet manifests (architecture doc §3:
//! `assets/default-pets/<id>/pet.json`).
//!
//! Deliberately contains ONLY parsing and pure data transformation, no
//! filesystem access. Reading `pet.json` and loading the referenced
//! spritesheet are handled by `kyouzai-app`, which orchestrates Core and Asset.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::animation::{AnimationClip, AnimationSet, Frame};
use crate::sprite::SpriteRegion;

fn default_walk_speed() -> f32 {
    60.0
}

fn default_looped() -> bool {
    true
}

// -----------------------------------------------------------------------------
// Manifest data
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetManifest {
    pub id: String,
    pub name: String,

    /// Spritesheet path relative to the manifest's directory.
    pub spritesheet: String,

    pub grid_columns: u32,
    pub grid_rows: u32,

    /// Logical sprite size in pixels.
    pub sprite_size: [u32; 2],

    #[serde(default = "default_walk_speed")]
    pub walk_speed: f32,

    /// Keyed by `CompanionState::animation_key()`.
    /// The "idle" state is mandatory.
    pub states: BTreeMap<String, AnimationDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationRange {
    /// Zero-indexed row, growing from top to bottom.
    pub row: u32,

    /// First column of the range.
    pub start: u32,

    /// Number of consecutive frames.
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationDef {
    /// Consecutive frames on a single row.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<AnimationRange>,

    /// Arbitrary frame coordinates in playback order: [column, row].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frames: Option<Vec<[u32; 2]>>,

    pub fps: f32,

    #[serde(default = "default_looped")]
    pub looped: bool,
}

// -----------------------------------------------------------------------------
// Errors
// -----------------------------------------------------------------------------

#[derive(Debug)]
pub enum ManifestError {
    Json(serde_json::Error),

    MissingRequiredState(&'static str),

    InvalidGridSize,
    InvalidSpriteSize,
    InvalidWalkSpeed,

    InvalidAnimationDefinition {
        state: String,
    },
    EmptyAnimation {
        state: String,
    },
    InvalidFrameRange {
        state: String,
    },
    InvalidFrameCoordinate {
        state: String,
        column: u32,
        row: u32,
    },
    InvalidFps {
        state: String,
    },
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(error) => {
                write!(f, "invalid pet manifest JSON: {error}")
            }
            Self::MissingRequiredState(state) => {
                write!(f, "pet manifest is missing the required '{state}' state")
            }
            Self::InvalidGridSize => {
                write!(f, "grid_columns and grid_rows must both be greater than 0")
            }
            Self::InvalidSpriteSize => {
                write!(f, "sprite_size dimensions must both be greater than 0")
            }
            Self::InvalidWalkSpeed => {
                write!(f, "walk_speed must be finite and non-negative")
            }
            Self::InvalidAnimationDefinition { state } => {
                write!(
                    f,
                    "state '{state}' must define exactly one of 'range' or 'frames'"
                )
            }
            Self::EmptyAnimation { state } => {
                write!(f, "state '{state}' must contain at least one frame")
            }
            Self::InvalidFrameRange { state } => {
                write!(
                    f,
                    "state '{state}' has a frame range outside the spritesheet grid"
                )
            }
            Self::InvalidFrameCoordinate { state, column, row } => {
                write!(
                    f,
                    "state '{state}' has an invalid frame coordinate [{column}, {row}]"
                )
            }
            Self::InvalidFps { state } => {
                write!(f, "state '{state}' fps must be finite and greater than 0")
            }
        }
    }
}

impl std::error::Error for ManifestError {}

// -----------------------------------------------------------------------------
// Parsing, validation and transformation
// -----------------------------------------------------------------------------

impl PetManifest {
    pub fn from_json(data: &str) -> Result<Self, ManifestError> {
        serde_json::from_str(data).map_err(ManifestError::Json)
    }

    /// Validates manifest fields and animation definitions.
    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.grid_columns == 0 || self.grid_rows == 0 {
            return Err(ManifestError::InvalidGridSize);
        }

        if self.sprite_size[0] == 0 || self.sprite_size[1] == 0 {
            return Err(ManifestError::InvalidSpriteSize);
        }

        if !self.walk_speed.is_finite() || self.walk_speed < 0.0 {
            return Err(ManifestError::InvalidWalkSpeed);
        }

        if !self.states.contains_key("idle") {
            return Err(ManifestError::MissingRequiredState("idle"));
        }

        for (key, def) in &self.states {
            match (&def.range, &def.frames) {
                (Some(range), None) => {
                    if range.count == 0 {
                        return Err(ManifestError::EmptyAnimation { state: key.clone() });
                    }

                    let end = range.start.checked_add(range.count);

                    if range.row >= self.grid_rows || end.is_none_or(|end| end > self.grid_columns)
                    {
                        return Err(ManifestError::InvalidFrameRange { state: key.clone() });
                    }
                }

                (None, Some(frames)) => {
                    if frames.is_empty() {
                        return Err(ManifestError::EmptyAnimation { state: key.clone() });
                    }

                    for &[column, row] in frames {
                        if column >= self.grid_columns || row >= self.grid_rows {
                            return Err(ManifestError::InvalidFrameCoordinate {
                                state: key.clone(),
                                column,
                                row,
                            });
                        }
                    }
                }

                _ => {
                    return Err(ManifestError::InvalidAnimationDefinition { state: key.clone() });
                }
            }

            if !def.fps.is_finite() || def.fps <= 0.0 {
                return Err(ManifestError::InvalidFps { state: key.clone() });
            }
        }

        Ok(())
    }

    /// Converts animation definitions into concrete UV regions.
    pub fn build_animation_set(&self) -> Result<AnimationSet, ManifestError> {
        self.validate()?;

        let mut set = AnimationSet::default();

        for (key, def) in &self.states {
            let frame_duration = 1.0 / def.fps;

            let frames = match (&def.range, &def.frames) {
                (Some(range), None) => {
                    let end = range.start + range.count;

                    (range.start..end)
                        .map(|column| self.build_frame([column, range.row], frame_duration))
                        .collect()
                }

                (None, Some(coordinates)) => coordinates
                    .iter()
                    .copied()
                    .map(|coordinate| self.build_frame(coordinate, frame_duration))
                    .collect(),

                _ => unreachable!("animation definitions were already validated"),
            };

            set.insert_by_key(
                key.clone(),
                AnimationClip {
                    frames,
                    looped: def.looped,
                },
            );
        }

        Ok(set)
    }

    fn build_frame(&self, [column, row]: [u32; 2], duration_seconds: f32) -> Frame {
        let region = SpriteRegion::from_grid(column, row, self.grid_columns, self.grid_rows)
            .expect("frame coordinates were already validated");

        Frame {
            uv_min: region.uv_min(),
            uv_max: region.uv_max(),
            duration_seconds,
        }
    }
}
