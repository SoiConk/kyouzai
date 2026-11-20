use crate::animation::{AnimationClip, AnimationSet, Frame, Timeline};
use crate::state_machine::CompanionState;

fn frame(duration_seconds: f32) -> Frame {
    Frame {
        uv_min: [0.0, 0.0],
        uv_max: [1.0, 1.0],
        duration_seconds,
    }
}

#[test]
fn advance_switches_frame_after_its_duration_elapses() {
    let clip = AnimationClip {
        frames: vec![frame(0.1), frame(0.1)],
        looped: true,
    };
    let mut timeline = Timeline::default();

    assert!(!timeline.advance(&clip, 0.05));
    assert_eq!(timeline.frame_index(), 0);

    assert!(timeline.advance(&clip, 0.06));
    assert_eq!(timeline.frame_index(), 1);
}

#[test]
fn advance_recovers_when_frame_index_is_out_of_range_for_a_shorter_clip() {
    // Simulates forgetting to reset() after switching from a longer clip to a
    // shorter one: frame_index=5 is out of bounds for a 2-frame clip.
    let mut timeline = Timeline::default();
    timeline.advance(
        &AnimationClip {
            frames: vec![frame(0.01); 6],
            looped: true,
        },
        0.2, // force frame_index up to somewhere > 1
    );

    let short_clip = AnimationClip {
        frames: vec![frame(0.1), frame(0.1)],
        looped: true,
    };

    // Must not panic.
    timeline.advance(&short_clip, 0.0);
    assert!(timeline.frame_index() < short_clip.frames.len());
}

#[test]
fn reset_returns_to_frame_zero() {
    let clip = AnimationClip {
        frames: vec![frame(0.01), frame(0.01)],
        looped: true,
    };
    let mut timeline = Timeline::default();
    // 0.015s into a 2×0.01s looped clip lands mid-way through frame 1, not back at an
    // exact frame boundary — see architecture doc §6.2(a): the original 0.02s advance
    // was exactly 2 full frame-cycles, so the timeline legitimately (and correctly)
    // landed back on frame 0 on its own, making the `assert_ne!` below fail for
    // reasons that had nothing to do with `reset()` actually working.
    timeline.advance(&clip, 0.015);
    assert_ne!(timeline.frame_index(), 0);

    timeline.reset();
    assert_eq!(timeline.frame_index(), 0);
    assert_eq!(timeline.elapsed_seconds(), 0.0);
}

#[test]
fn animation_set_falls_back_to_idle_for_unauthored_states() {
    let mut set = AnimationSet::default();
    set.insert(
        CompanionState::Idle,
        AnimationClip {
            frames: vec![frame(0.1)],
            looped: true,
        },
    );

    assert!(set.get(CompanionState::Walk).is_some());
    assert!(set.get(CompanionState::Custom(42)).is_some());
}
