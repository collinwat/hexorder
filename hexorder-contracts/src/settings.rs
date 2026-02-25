//! Shared Settings types. See `docs/contracts/settings.md`.
//!
//! Defines the settings registry, editor preferences, theme definitions,
//! and a change notification event.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Editor Settings
// ---------------------------------------------------------------------------

/// Resolved editor preferences. All values are concrete (not Optional).
#[derive(Debug, Clone)]
pub struct EditorSettings {
    /// Base font size in points. Range 10.0–24.0, default 15.0.
    pub font_size: f32,
    /// Active workspace preset identifier (e.g. "map_editing"). Empty = default.
    pub workspace_preset: String,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            font_size: 15.0,
            workspace_preset: String::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Settings Registry
// ---------------------------------------------------------------------------

/// Central settings resource. Holds the resolved (merged) view of all settings
/// across three layers: compiled defaults, user config file, and project overrides.
///
/// Inserted by `SettingsPlugin` at startup. Systems read via `Res<SettingsRegistry>`.
#[derive(Resource, Debug, Clone)]
pub struct SettingsRegistry {
    /// Resolved editor preferences.
    pub editor: EditorSettings,
    /// Name of the active theme. `"brand"` = compiled default.
    pub active_theme: String,
}

impl Default for SettingsRegistry {
    fn default() -> Self {
        Self {
            editor: EditorSettings::default(),
            active_theme: "brand".to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Observer event fired when any settings layer changes (e.g., project loaded
/// with different preferences). Observers can re-read `SettingsRegistry` for
/// updated values.
#[derive(Event, Debug, Clone)]
pub struct SettingsChanged;

// ---------------------------------------------------------------------------
// Theme Definition
// ---------------------------------------------------------------------------

/// A serializable theme with ~14 color fields mapping to egui Visuals.
/// Loaded from TOML files in the themes directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeDefinition {
    /// Human-readable theme name (e.g. "Solarized Dark").
    pub name: String,
    /// Deep background — deepest UI panels. RGB.
    pub bg_deep: [u8; 3],
    /// Panel fill — panel backgrounds. RGB.
    pub bg_panel: [u8; 3],
    /// Surface — interactive surface areas. RGB.
    pub bg_surface: [u8; 3],
    /// Widget inactive fill. RGB.
    pub widget_inactive: [u8; 3],
    /// Widget hovered fill. RGB.
    pub widget_hovered: [u8; 3],
    /// Widget active fill. RGB.
    pub widget_active: [u8; 3],
    /// Primary accent color — selection, active states. RGB.
    pub accent_primary: [u8; 3],
    /// Secondary accent — emphasis, headings. RGB.
    pub accent_secondary: [u8; 3],
    /// Primary text color. RGB.
    pub text_primary: [u8; 3],
    /// Secondary text color. RGB.
    pub text_secondary: [u8; 3],
    /// Border/divider color. RGB.
    pub border: [u8; 3],
    /// Danger/error color. RGB.
    pub danger: [u8; 3],
    /// Success/confirmation color. RGB.
    pub success: [u8; 3],
}
