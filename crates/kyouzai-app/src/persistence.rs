use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// -----------------------------------------------------------------------------
// Persistence configuration
// -----------------------------------------------------------------------------

const QUALIFIER: &str = "dev";
const ORGANIZATION: &str = "kyouzai";
const APPLICATION: &str = "kyouzai-companion";
const FILE_NAME: &str = "state.json";

// -----------------------------------------------------------------------------
// Application state
// -----------------------------------------------------------------------------

/// Small pieces of application state persisted between runs.
///
/// This state is intentionally separate from the pet's runtime state:
/// it stores application-level information needed to restore the previous
/// session, not the companion's behavior or animation state.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppState {
    /// Last known screen-space position of the companion.
    ///
    /// Uses the same coordinate space as `kyouzai_core::Transform::position`.
    /// `None` means no previous position has been saved.
    pub last_position: Option<[f32; 2]>,
}

// -----------------------------------------------------------------------------
// Storage location
// -----------------------------------------------------------------------------

/// Resolves the application state file path.
///
/// Returns `None` if the platform does not provide a suitable configuration
/// directory.
fn state_file_path() -> Option<PathBuf> {
    let directories = directories::ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)?;

    Some(directories.config_dir().join(FILE_NAME))
}

// -----------------------------------------------------------------------------
// Loading
// -----------------------------------------------------------------------------

/// Loads the previously persisted application state.
///
/// Loading is best-effort: a missing file, an inaccessible configuration
/// directory, or invalid JSON results in the default state.
///
/// Losing the remembered position is not considered a fatal startup error.
pub fn load() -> AppState {
    let Some(path) = state_file_path() else {
        return AppState::default();
    };

    let Ok(contents) = std::fs::read_to_string(path) else {
        return AppState::default();
    };

    serde_json::from_str(&contents).unwrap_or_default()
}

// -----------------------------------------------------------------------------
// Saving
// -----------------------------------------------------------------------------

/// Persists the application state.
///
/// Saving is best-effort and never interrupts the application session.
/// Failures are logged for diagnostics instead of being propagated.
pub fn save(state: &AppState) {
    let Some(path) = state_file_path() else {
        return;
    };

    if let Some(parent) = path.parent()
        && let Err(error) = std::fs::create_dir_all(parent)
    {
        tracing::warn!(
            "failed to create config directory {}: {error}",
            parent.display()
        );
        return;
    }

    let contents = match serde_json::to_string_pretty(state) {
        Ok(contents) => contents,
        Err(error) => {
            tracing::warn!("failed to serialize app state: {error}");
            return;
        }
    };

    if let Err(error) = std::fs::write(&path, contents) {
        tracing::warn!("failed to save app state to {}: {error}", path.display());
    }
}
