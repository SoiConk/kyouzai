use std::path::{Path, PathBuf};

use glam::Vec2;
use kyouzai_asset::{AssetError, AssetLoader, LoadedImage};
use kyouzai_core::{Companion, ManifestError, PetManifest, TextureId, Transform};
use kyouzai_renderer::{Renderer, TextureError, TextureHandle};

use crate::alpha_mask::AlphaMask;

const DEFAULT_PET_ID: &str = "kyouzai";

const DEFAULT_TEXTURE_ID: TextureId = TextureId::new(1);

#[derive(Debug)]
struct LoadedPet {
    manifest: PetManifest,
    spritesheet: LoadedImage,
}

#[derive(Debug)]
pub enum PetLoadError {
    ReadManifest {
        path: PathBuf,
        source: std::io::Error,
    },
    ParseManifest(ManifestError),
    LoadSpritesheet(AssetError),
}

impl std::fmt::Display for PetLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadManifest { path, source } => {
                write!(
                    f,
                    "failed to read pet manifest '{}': {source}",
                    path.display()
                )
            }
            Self::ParseManifest(error) => {
                write!(f, "failed to parse pet manifest: {error}")
            }
            Self::LoadSpritesheet(error) => {
                write!(f, "failed to load spritesheet: {error}")
            }
        }
    }
}

impl std::error::Error for PetLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadManifest { source, .. } => Some(source),
            Self::ParseManifest(error) => Some(error),
            Self::LoadSpritesheet(error) => Some(error),
        }
    }
}

#[derive(Debug)]
pub enum PetError {
    Load(PetLoadError),
    Manifest(ManifestError),
    Texture(TextureError),
    SpritesheetDimensionsOverflow,
    SpritesheetSizeMismatch {
        expected: (u32, u32),
        actual: (u32, u32),
    },
}

impl std::fmt::Display for PetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load(error) => write!(f, "failed to load pet: {error}"),
            Self::Manifest(error) => write!(f, "invalid pet manifest: {error}"),
            Self::Texture(error) => write!(f, "failed to upload pet texture: {error}"),
            Self::SpritesheetDimensionsOverflow => {
                write!(f, "spritesheet dimensions overflow")
            }
            Self::SpritesheetSizeMismatch { expected, actual } => {
                write!(
                    f,
                    "spritesheet size mismatch: expected {}x{}, got {}x{}",
                    expected.0, expected.1, actual.0, actual.1
                )
            }
        }
    }
}

impl std::error::Error for PetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Load(error) => Some(error),
            Self::Manifest(error) => Some(error),
            Self::Texture(error) => Some(error),
            Self::SpritesheetDimensionsOverflow | Self::SpritesheetSizeMismatch { .. } => None,
        }
    }
}

pub struct PetAssets {
    pub companion: Companion,
    pub alpha_mask: AlphaMask,
    pub texture_handle: TextureHandle,
    pub sprite_size: (u32, u32),
}

fn load_manifest(path: &Path) -> Result<PetManifest, PetLoadError> {
    let raw = std::fs::read_to_string(path).map_err(|source| PetLoadError::ReadManifest {
        path: path.to_path_buf(),
        source,
    })?;

    PetManifest::from_json(&raw).map_err(PetLoadError::ParseManifest)
}

fn load_pet(manifest_path: &Path) -> Result<LoadedPet, PetLoadError> {
    let manifest = load_manifest(manifest_path)?;

    let spritesheet_path = manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(&manifest.spritesheet);

    let spritesheet = AssetLoader::new()
        .load(spritesheet_path)
        .map_err(PetLoadError::LoadSpritesheet)?;

    Ok(LoadedPet {
        manifest,
        spritesheet,
    })
}

pub fn load_default_pet(
    renderer: &mut Renderer,
    initial_position: Vec2,
) -> Result<PetAssets, PetError> {
    let manifest_path = default_manifest_path();
    let loaded = load_pet(&manifest_path).map_err(PetError::Load)?;

    let animations = loaded
        .manifest
        .build_animation_set()
        .map_err(PetError::Manifest)?;

    let [sprite_width, sprite_height] = loaded.manifest.sprite_size;

    let expected_width = loaded
        .manifest
        .grid_columns
        .checked_mul(sprite_width)
        .ok_or(PetError::SpritesheetDimensionsOverflow)?;

    let expected_height = loaded
        .manifest
        .grid_rows
        .checked_mul(sprite_height)
        .ok_or(PetError::SpritesheetDimensionsOverflow)?;

    let expected = (expected_width, expected_height);
    let actual = (loaded.spritesheet.width(), loaded.spritesheet.height());

    if expected != actual {
        return Err(PetError::SpritesheetSizeMismatch { expected, actual });
    }

    let texture_handle = renderer
        .create_texture(
            loaded.spritesheet.width(),
            loaded.spritesheet.height(),
            loaded.spritesheet.rgba8_data(),
        )
        .map_err(PetError::Texture)?;

    let alpha_mask = AlphaMask::from_image(&loaded.spritesheet);

    let [width, height] = loaded.manifest.sprite_size;

    let transform = Transform::new(initial_position, Vec2::new(width as f32, height as f32));

    let companion = Companion::new(
        transform,
        DEFAULT_TEXTURE_ID,
        animations,
        loaded.manifest.walk_speed,
    );

    Ok(PetAssets {
        companion,
        alpha_mask,
        texture_handle,
        sprite_size: (width, height),
    })
}

pub fn default_sprite_size() -> Result<(u32, u32), PetError> {
    let manifest_path = default_manifest_path();

    let manifest = load_manifest(&manifest_path).map_err(PetError::Load)?;
    manifest.validate().map_err(PetError::Manifest)?;
    let [width, height] = manifest.sprite_size;

    Ok((width, height))
}

fn default_manifest_path() -> PathBuf {
    let relative = Path::new("assets")
        .join("default-pets")
        .join(DEFAULT_PET_ID)
        .join("pet.json");

    if let Ok(exe_path) = std::env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
    {
        let candidate = exe_dir.join(&relative);

        if candidate.exists() {
            return candidate;
        }
    }

    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}
