use crate::config::PetManifest;

const SAMPLE: &str = r#"{
    "id": "kyouzai",
    "name": "Kyouzai",
    "spritesheet": "spritesheet.png",
    "grid_columns": 4,
    "grid_rows": 2,
    "sprite_size": [128, 128],
    "walk_speed": 80.0,
    "states": {
        "idle": {
            "range": { "row": 0, "start": 0, "count": 4 },
            "fps": 6.0
        },
        "walk": {
            "range": { "row": 1, "start": 0, "count": 4 },
            "fps": 8.0
        }
    }
}"#;

#[test]
fn parses_a_well_formed_manifest() {
    let manifest = PetManifest::from_json(SAMPLE).expect("valid manifest");

    assert_eq!(manifest.id, "kyouzai");
    assert_eq!(manifest.states.len(), 2);
}

#[test]
fn builds_an_animation_set_with_correct_frame_count() {
    let manifest = PetManifest::from_json(SAMPLE).expect("valid manifest");
    let set = manifest.build_animation_set().expect("valid animation set");

    let idle = set
        .get(crate::state_machine::CompanionState::Idle)
        .expect("idle clip present");

    assert_eq!(idle.frames.len(), 4);
}

#[test]
fn rejects_a_manifest_missing_the_idle_state() {
    let manifest = PetManifest::from_json(
        r#"{
            "id": "x",
            "name": "x",
            "spritesheet": "s.png",
            "grid_columns": 1,
            "grid_rows": 1,
            "sprite_size": [64, 64],
            "states": {
                "walk": {
                    "range": { "row": 0, "start": 0, "count": 1 },
                    "fps": 1.0
                }
            }
        }"#,
    )
    .expect("valid JSON");

    assert!(manifest.build_animation_set().is_err());
}
