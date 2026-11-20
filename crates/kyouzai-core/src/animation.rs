use std::collections::BTreeMap;

use crate::state_machine::CompanionState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
    pub duration_seconds: f32,
}

#[derive(Debug, Clone)]
pub struct AnimationClip {
    pub frames: Vec<Frame>,
    pub looped: bool,
}

#[derive(Debug, Clone)]
pub struct Timeline {
    frame_index: usize,
    elapsed_seconds: f32,
}

impl Default for Timeline {
    fn default() -> Self {
        Self {
            frame_index: 0,
            elapsed_seconds: 0.0,
        }
    }
}

impl Timeline {
    pub fn frame_index(&self) -> usize {
        self.frame_index
    }

    pub fn elapsed_seconds(&self) -> f32 {
        self.elapsed_seconds
    }

    pub fn reset(&mut self) {
        self.frame_index = 0;
        self.elapsed_seconds = 0.0;
    }

    pub fn advance(&mut self, clip: &AnimationClip, dt_seconds: f32) -> bool {
        if clip.frames.is_empty() {
            return false;
        }

        if self.frame_index >= clip.frames.len() {
            self.frame_index = 0;
            self.elapsed_seconds = 0.0;
        }

        self.elapsed_seconds += dt_seconds.max(0.0);
        let mut changed = false;

        loop {
            let duration = clip.frames[self.frame_index]
                .duration_seconds
                .max(f32::EPSILON);

            if self.elapsed_seconds < duration {
                break;
            }

            self.elapsed_seconds -= duration;
            let next = self.frame_index + 1;

            if next >= clip.frames.len() {
                if clip.looped {
                    self.frame_index = 0;
                    changed = true;
                } else {
                    self.frame_index = clip.frames.len() - 1;
                    self.elapsed_seconds = 0.0;
                    break;
                }
            } else {
                self.frame_index = next;
                changed = true;
            }
        }

        changed
    }
}

#[derive(Debug, Clone, Default)]
pub struct AnimationSet {
    clips: BTreeMap<String, AnimationClip>,
}

impl AnimationSet {
    pub fn insert(&mut self, state: CompanionState, clip: AnimationClip) {
        self.insert_by_key(state.animation_key(), clip);
    }

    pub fn insert_by_key(&mut self, key: impl Into<String>, clip: AnimationClip) {
        self.clips.insert(key.into(), clip);
    }

    pub fn get(&self, state: CompanionState) -> Option<&AnimationClip> {
        let key = state.animation_key();
        self.clips
            .get(key.as_ref())
            .or_else(|| self.clips.get("idle"))
    }
}
