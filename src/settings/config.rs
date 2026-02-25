//! TOML config file loading and three-layer merge logic for settings.

use std::path::PathBuf;

use bevy::prelude::*;
use serde::Deserialize;

use hexorder_contracts::settings::{
    EditorSettings, SettingsRegistry, ThemeDefinition, ThemeLibrary,
};

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

// ---------------------------------------------------------------------------
// Theme loading
// ---------------------------------------------------------------------------

/// Returns the compiled brand theme as a `ThemeDefinition`.
/// Values match the `BrandTheme` constants in `editor_ui/components.rs`.
pub(crate) fn brand_theme_definition() -> ThemeDefinition {
    ThemeDefinition {
        name: "Brand".to_string(),
        bg_deep: [10, 10, 10],
        bg_panel: [25, 25, 25],
        bg_surface: [35, 35, 35],
        widget_inactive: [40, 40, 40],
        widget_hovered: [55, 55, 55],
        widget_active: [70, 70, 70],
        accent_primary: [0, 92, 128],
        accent_secondary: [200, 150, 64],
        text_primary: [224, 224, 224],
        text_secondary: [128, 128, 128],
        border: [60, 60, 60],
        danger: [200, 80, 80],
        success: [80, 152, 80],
    }
}

/// Load custom themes from the themes directory. Returns the brand theme
/// first, followed by any custom themes found on disk.
pub(crate) fn load_themes() -> ThemeLibrary {
    let mut themes = vec![brand_theme_definition()];

    let themes_dir = config_dir().join("themes");
    let entries = match std::fs::read_dir(&themes_dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            info!(
                "No themes directory at {}, using brand theme only",
                themes_dir.display()
            );
            return ThemeLibrary { themes };
        }
        Err(e) => {
            warn!(
                "Failed to read themes directory {}: {e}",
                themes_dir.display()
            );
            return ThemeLibrary { themes };
        }
    };

    for entry in entries {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }

        let contents = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to read theme file {}: {e}", path.display());
                continue;
            }
        };

        match toml::from_str::<ThemeDefinition>(&contents) {
            Ok(theme) => {
                info!("Loaded theme '{}' from {}", theme.name, path.display());
                themes.push(theme);
            }
            Err(e) => {
                warn!("Failed to parse theme file {}: {e}", path.display());
            }
        }
    }

    ThemeLibrary { themes }
}
