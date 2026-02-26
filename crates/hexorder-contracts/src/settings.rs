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
    /// Active workspace preset identifier (e.g. "`map_editing`"). Empty = default.
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
// System Sets
// ---------------------------------------------------------------------------

/// System set indicating the settings registry has been populated for this
/// state transition. Consumer plugins (e.g. `editor_ui`) should schedule
/// their restore systems `.after(SettingsReady)` on `OnEnter(AppScreen::Editor)`.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SettingsReady;

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Observer event fired when any settings layer changes (e.g., project loaded
/// with different preferences). Observers can re-read `SettingsRegistry` for
/// updated values.
#[derive(Event, Debug, Clone)]
pub struct SettingsChanged;

// ---------------------------------------------------------------------------
// Theme Library
// ---------------------------------------------------------------------------

/// Available themes for the editor. Loaded at startup by `SettingsPlugin`.
/// The brand theme is always present as the first entry.
#[derive(Resource, Debug, Clone, Default)]
pub struct ThemeLibrary {
    /// All available themes, with brand theme first.
    pub themes: Vec<ThemeDefinition>,
}

impl ThemeLibrary {
    /// Find a theme by name. Returns `None` if not found.
    #[must_use]
    pub fn find(&self, name: &str) -> Option<&ThemeDefinition> {
        self.themes.iter().find(|t| t.name == name)
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_settings_default() {
        let s = EditorSettings::default();
        assert!((s.font_size - 15.0).abs() < f32::EPSILON);
        assert!(s.workspace_preset.is_empty());
    }

    #[test]
    fn editor_settings_custom_values() {
        let s = EditorSettings {
            font_size: 18.0,
            workspace_preset: "wargame".to_string(),
        };
        assert!((s.font_size - 18.0).abs() < f32::EPSILON);
        assert_eq!(s.workspace_preset, "wargame");
    }

    #[test]
    fn settings_registry_default() {
        let sr = SettingsRegistry::default();
        assert!((sr.editor.font_size - 15.0).abs() < f32::EPSILON);
        assert_eq!(sr.active_theme, "brand");
    }

    #[test]
    fn theme_library_find_returns_matching_theme() {
        let lib = ThemeLibrary {
            themes: vec![ThemeDefinition {
                name: "Dark".to_string(),
                bg_deep: [0, 0, 0],
                bg_panel: [20, 20, 20],
                bg_surface: [30, 30, 30],
                widget_inactive: [40, 40, 40],
                widget_hovered: [50, 50, 50],
                widget_active: [60, 60, 60],
                accent_primary: [0, 128, 255],
                accent_secondary: [255, 180, 0],
                text_primary: [230, 230, 230],
                text_secondary: [160, 160, 160],
                border: [60, 60, 60],
                danger: [220, 50, 50],
                success: [50, 180, 50],
            }],
        };
        assert!(lib.find("Dark").is_some());
        assert!(lib.find("Light").is_none());
    }

    #[test]
    fn theme_library_default_is_empty() {
        let lib = ThemeLibrary::default();
        assert!(lib.themes.is_empty());
        assert!(lib.find("anything").is_none());
    }

    #[test]
    fn settings_changed_event_debug() {
        let evt = SettingsChanged;
        let debug = format!("{evt:?}");
        assert!(debug.contains("SettingsChanged"));
    }
}
