//! TOML config file loading and three-layer merge logic for settings.

use std::path::PathBuf;

use bevy::prelude::*;
use serde::Deserialize;

use crate::contracts::settings::{EditorSettings, SettingsRegistry};

// ---------------------------------------------------------------------------
// Partial types (for merge semantics — Option<T> fields)
// ---------------------------------------------------------------------------

/// Partial settings for one layer. `None` = inherit from lower layer.
#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct PartialSettings {
    #[serde(default)]
    pub(crate) editor: PartialEditorSettings,
    pub(crate) theme: Option<String>,
}

/// Partial editor preferences.
#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct PartialEditorSettings {
    pub(crate) font_size: Option<f32>,
    pub(crate) workspace_preset: Option<String>,
}

// ---------------------------------------------------------------------------
// Defaults
// ---------------------------------------------------------------------------

impl PartialSettings {
    /// Build the defaults layer with all values populated.
    pub(crate) fn defaults() -> Self {
        Self {
            editor: PartialEditorSettings {
                font_size: Some(15.0),
                workspace_preset: Some(String::new()),
            },
            theme: Some("brand".to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// Config directory resolution
// ---------------------------------------------------------------------------

/// Returns the config directory based on compile-time feature flags.
/// Same pattern as `shortcuts/config.rs`.
pub(crate) fn config_dir() -> PathBuf {
    #[cfg(feature = "macos-app")]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(home).join("Library/Application Support/hexorder")
    }

    #[cfg(not(feature = "macos-app"))]
    {
        PathBuf::from("config")
    }
}

/// Returns the full path to the settings config file.
pub(crate) fn config_path() -> PathBuf {
    config_dir().join("settings.toml")
}

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------

/// Load the user settings layer from disk. Returns default (empty) on missing
/// or unparsable file.
pub(crate) fn load_user_settings() -> PartialSettings {
    let path = config_path();

    let contents = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            info!("No settings config at {}, using defaults", path.display());
            return PartialSettings::default();
        }
        Err(e) => {
            warn!("Failed to read settings config {}: {e}", path.display());
            return PartialSettings::default();
        }
    };

    match toml::from_str(&contents) {
        Ok(s) => {
            info!("Loaded user settings from {}", path.display());
            s
        }
        Err(e) => {
            warn!("Failed to parse settings config {}: {e}", path.display());
            PartialSettings::default()
        }
    }
}

// ---------------------------------------------------------------------------
// Three-layer merge
// ---------------------------------------------------------------------------

/// Merge three layers into a resolved `SettingsRegistry`.
/// Priority: project > user > defaults (field-by-field).
pub(crate) fn merge(
    defaults: &PartialSettings,
    user: &PartialSettings,
    project: &PartialSettings,
) -> SettingsRegistry {
    SettingsRegistry {
        editor: EditorSettings {
            font_size: project
                .editor
                .font_size
                .or(user.editor.font_size)
                .or(defaults.editor.font_size)
                .unwrap_or(15.0),
            workspace_preset: project
                .editor
                .workspace_preset
                .clone()
                .or_else(|| user.editor.workspace_preset.clone())
                .or_else(|| defaults.editor.workspace_preset.clone())
                .unwrap_or_default(),
        },
        active_theme: project
            .theme
            .clone()
            .or_else(|| user.theme.clone())
            .or_else(|| defaults.theme.clone())
            .unwrap_or_else(|| "brand".to_string()),
    }
}
