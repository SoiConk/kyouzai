use crate::sprite::{Sprite, SpriteRegion, TextureId};

#[test]
fn sprite_defaults_to_full_region_and_white_tint() {
    let sprite = Sprite::new(TextureId::new(7));

    assert_eq!(sprite.texture(), TextureId::new(7));
    assert_eq!(sprite.region(), SpriteRegion::FULL);
    assert_eq!(sprite.tint(), [1.0; 4]);
}

#[test]
fn sprite_region_can_select_atlas_area() {
    let region = SpriteRegion::new([0.25, 0.5], [0.5, 1.0]);
    let sprite = Sprite::with_region(TextureId::new(2), region);

    assert_eq!(sprite.region(), region);
}

#[test]
fn from_grid_computes_expected_cell() {
    // 4 columns x 2 rows grid; cell (col=1, row=1) is the second column of the
    // second (bottom) row.
    let region = SpriteRegion::from_grid(1, 1, 4, 2).expect("valid grid cell");

    assert_eq!(region.uv_min(), [0.25, 0.5]);
    assert_eq!(region.uv_max(), [0.5, 1.0]);
}

#[test]
fn from_grid_returns_none_for_invalid_cell() {
    assert!(SpriteRegion::from_grid(4, 0, 4, 2).is_none());
    assert!(SpriteRegion::from_grid(0, 2, 4, 2).is_none());
    assert!(SpriteRegion::from_grid(0, 0, 0, 2).is_none());
}
