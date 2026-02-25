# Settings Infrastructure (Scope 1 + 5) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan
> task-by-task.

**Goal:** Build the settings contract and plugin infrastructure — three-layer merge of defaults,
user TOML, and project overrides into a typed `SettingsRegistry` resource.

**Architecture:** Contract types in `src/contracts/settings.rs` define the public API
(`SettingsRegistry`, `EditorSettings`, `SettingsChanged`, `ThemeDefinition`). The `SettingsPlugin`
in `src/settings/` loads user config from `~/.../hexorder/settings.toml`, merges layers using
plugin-private `PartialSettings` structs with `Option<T>` fields, and inserts the resolved registry
resource at startup. Project-layer systems react to `AppScreen` state transitions.

**Tech Stack:** Bevy 0.18, `toml` crate (already in deps), `serde` (already in deps)

**Design doc:** `docs/plans/2026-02-24-settings-infrastructure-design.md`

**Bevy guide:** `docs/guides/bevy.md` — Observer events (`#[derive(Event)]`, `commands.trigger()`,
`app.add_observer()`). `EventReader`/`EventWriter`/`app.add_event` are deprecated.

---

### Task 1: Settings Contract — Types

**Files:**

- Create: `src/contracts/settings.rs`
- Modify: `src/contracts/mod.rs`

**Step 1: Create the contract module**

Create `src/contracts/settings.rs`:

```rust
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
```

**Step 2: Register the module in contracts/mod.rs**

Add to `src/contracts/mod.rs` (alphabetical, after `pub mod persistence`):

```rust
#[allow(dead_code)]
pub mod settings;
```

**Step 3: Verify it compiles**

Run: `cargo build` Expected: Clean compilation.

**Step 4: Commit**

```
feat(contracts): add settings contract types

Adds SettingsRegistry, EditorSettings, SettingsChanged event, and
ThemeDefinition to src/contracts/settings.rs. (ref #173)
```

---

### Task 2: Settings Contract — Spec Document

**Files:**

- Create: `docs/contracts/settings.md`

**Step 1: Write the contract spec**

Create `docs/contracts/settings.md`:

```markdown
# Contract: Settings

## Owner

`settings` plugin

## Purpose

Defines the settings registry, editor preferences, theme definitions, and a change notification
event. Systems across all plugins read `SettingsRegistry` to access merged settings. The settings
plugin manages the three-layer merge (defaults, user config, project overrides).

## Consumers

- editor_ui (reads font_size, active_theme for rendering)
- persistence (reads/writes workspace_preset and font_size to project files)

## Producers

- settings (inserts and manages SettingsRegistry, fires SettingsChanged)

## Types

### `EditorSettings`

Resolved editor preferences. All fields are concrete (non-Optional).

| Field              | Type     | Default | Description                                  |
| ------------------ | -------- | ------- | -------------------------------------------- |
| `font_size`        | `f32`    | `15.0`  | Base font size in points. Range 10.0-24.0    |
| `workspace_preset` | `String` | `""`    | Active workspace preset ID. Empty = default. |

### `SettingsRegistry`

Central settings resource holding the resolved merged view.

| Field          | Type             | Default    | Description                 |
| -------------- | ---------------- | ---------- | --------------------------- |
| `editor`       | `EditorSettings` | (defaults) | Resolved editor preferences |
| `active_theme` | `String`         | `"brand"`  | Name of the active theme    |

### `SettingsChanged`

Observer event fired when any settings layer changes. No fields — observers re-read
`Res<SettingsRegistry>` for updated values.

### `ThemeDefinition`

Serializable theme with ~14 color fields. Loaded from TOML.

| Field              | Type      | Description                     |
| ------------------ | --------- | ------------------------------- |
| `name`             | `String`  | Human-readable theme name       |
| `bg_deep`          | `[u8; 3]` | Deep background RGB             |
| `bg_panel`         | `[u8; 3]` | Panel fill RGB                  |
| `bg_surface`       | `[u8; 3]` | Surface/interactive area RGB    |
| `widget_inactive`  | `[u8; 3]` | Widget inactive fill RGB        |
| `widget_hovered`   | `[u8; 3]` | Widget hovered fill RGB         |
| `widget_active`    | `[u8; 3]` | Widget active fill RGB          |
| `accent_primary`   | `[u8; 3]` | Primary accent (selection) RGB  |
| `accent_secondary` | `[u8; 3]` | Secondary accent (emphasis) RGB |
| `text_primary`     | `[u8; 3]` | Primary text RGB                |
| `text_secondary`   | `[u8; 3]` | Secondary text RGB              |
| `border`           | `[u8; 3]` | Border/divider RGB              |
| `danger`           | `[u8; 3]` | Danger/error RGB                |
| `success`          | `[u8; 3]` | Success/confirmation RGB        |

## Invariants

- `SettingsRegistry` is inserted during `SettingsPlugin::build()` (immediate, before consumers)
- `SettingsPlugin` must be registered before `EditorUiPlugin` in `main.rs`
- `SettingsChanged` is fired via `commands.trigger()` (observer event, not deprecated EventWriter)
- Missing user config file is not an error — all defaults are used

## Changelog

| Date       | Change             | Reason                               |
| ---------- | ------------------ | ------------------------------------ |
| 2026-02-24 | Initial definition | Pitch #173 — settings infrastructure |
```

**Step 2: Run prettier**

Run: `npx prettier --write docs/contracts/settings.md`

**Step 3: Commit**

```
docs(contracts): add settings contract spec

Spec-code parity for src/contracts/settings.rs. (ref #173)
```

---

### Task 3: Settings Plugin — Config Loading

**Files:**

- Create: `src/settings/config.rs`

**Step 1: Write the config module**

Create `src/settings/config.rs`. This contains the `PartialSettings` structs (plugin-private,
`Option<T>` fields for merge), the `config_dir()` function (same `#[cfg(feature)]` pattern as
`src/shortcuts/config.rs`), and the merge logic.

```rust
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
```

**Step 2: Verify it compiles**

This file won't compile standalone yet (no mod.rs references it). That's OK — Task 5 wires it up.
Proceed to Task 4.

---

### Task 4: Settings Plugin — Config Tests

**Files:**

- Create: `src/settings/tests.rs`

**Step 1: Write merge tests**

Create `src/settings/tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use crate::contracts::settings::SettingsRegistry;
    use crate::settings::config::{merge, PartialEditorSettings, PartialSettings};

    fn empty() -> PartialSettings {
        PartialSettings::default()
    }

    fn defaults() -> PartialSettings {
        PartialSettings::defaults()
    }

    #[test]
    fn merge_defaults_only() {
        let result = merge(&defaults(), &empty(), &empty());
        assert!((result.editor.font_size - 15.0).abs() < f32::EPSILON);
        assert!(result.editor.workspace_preset.is_empty());
        assert_eq!(result.active_theme, "brand");
    }

    #[test]
    fn user_layer_overrides_defaults() {
        let user = PartialSettings {
            editor: PartialEditorSettings {
                font_size: Some(18.0),
                ..Default::default()
            },
            theme: Some("solarized".to_string()),
        };
        let result = merge(&defaults(), &user, &empty());
        assert!((result.editor.font_size - 18.0).abs() < f32::EPSILON);
        assert_eq!(result.active_theme, "solarized");
        // workspace_preset falls through to default
        assert!(result.editor.workspace_preset.is_empty());
    }

    #[test]
    fn project_layer_overrides_user() {
        let user = PartialSettings {
            editor: PartialEditorSettings {
                font_size: Some(18.0),
                ..Default::default()
            },
            ..Default::default()
        };
        let project = PartialSettings {
            editor: PartialEditorSettings {
                font_size: Some(20.0),
                ..Default::default()
            },
            ..Default::default()
        };
        let result = merge(&defaults(), &user, &project);
        assert!((result.editor.font_size - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn project_overrides_only_specified_fields() {
        let user = PartialSettings {
            editor: PartialEditorSettings {
                font_size: Some(18.0),
                workspace_preset: Some("unit_design".to_string()),
            },
            theme: Some("solarized".to_string()),
        };
        let project = PartialSettings {
            editor: PartialEditorSettings {
                font_size: Some(20.0),
                workspace_preset: None, // not overridden
            },
            theme: None, // not overridden
        };
        let result = merge(&defaults(), &user, &project);
        assert!((result.editor.font_size - 20.0).abs() < f32::EPSILON);
        assert_eq!(result.editor.workspace_preset, "unit_design"); // from user
        assert_eq!(result.active_theme, "solarized"); // from user
    }

    #[test]
    fn all_empty_layers_use_hardcoded_fallbacks() {
        let result = merge(&empty(), &empty(), &empty());
        assert!((result.editor.font_size - 15.0).abs() < f32::EPSILON);
        assert!(result.editor.workspace_preset.is_empty());
        assert_eq!(result.active_theme, "brand");
    }

    #[test]
    fn partial_settings_defaults_has_all_values() {
        let d = defaults();
        assert!(d.editor.font_size.is_some());
        assert!(d.editor.workspace_preset.is_some());
        assert!(d.theme.is_some());
    }

    #[test]
    fn partial_settings_deserializes_from_toml() {
        let toml_str = r#"
            theme = "dark"

            [editor]
            font_size = 16.5
            workspace_preset = "rule_authoring"
        "#;
        let parsed: PartialSettings = toml::from_str(toml_str).expect("should parse");
        assert!((parsed.editor.font_size.expect("set") - 16.5).abs() < f32::EPSILON);
        assert_eq!(
            parsed.editor.workspace_preset.as_deref(),
            Some("rule_authoring")
        );
        assert_eq!(parsed.theme.as_deref(), Some("dark"));
    }

    #[test]
    fn partial_settings_deserializes_empty_toml() {
        let parsed: PartialSettings = toml::from_str("").expect("should parse empty");
        assert!(parsed.editor.font_size.is_none());
        assert!(parsed.editor.workspace_preset.is_none());
        assert!(parsed.theme.is_none());
    }

    #[test]
    fn partial_settings_deserializes_partial_toml() {
        let toml_str = r#"
            [editor]
            font_size = 12.0
        "#;
        let parsed: PartialSettings = toml::from_str(toml_str).expect("should parse");
        assert!((parsed.editor.font_size.expect("set") - 12.0).abs() < f32::EPSILON);
        assert!(parsed.editor.workspace_preset.is_none());
        assert!(parsed.theme.is_none());
    }

    #[test]
    fn settings_registry_default() {
        let reg = SettingsRegistry::default();
        assert!((reg.editor.font_size - 15.0).abs() < f32::EPSILON);
        assert!(reg.editor.workspace_preset.is_empty());
        assert_eq!(reg.active_theme, "brand");
    }
}
```

**Step 2: Proceed to Task 5** (tests won't compile until mod.rs exists).

---

### Task 5: Settings Plugin — mod.rs and Systems

**Files:**

- Create: `src/settings/mod.rs`
- Create: `src/settings/systems.rs`
- Modify: `src/main.rs`

**Step 1: Create systems.rs**

Create `src/settings/systems.rs`:

```rust
//! Systems for the settings plugin.

use bevy::prelude::*;

use crate::contracts::persistence::{AppScreen, Workspace};
use crate::contracts::settings::{SettingsChanged, SettingsRegistry};

use super::config::{merge, PartialEditorSettings, PartialSettings};
use super::SettingsLayers;

/// On entering the editor, read the Workspace resource and apply the project
/// layer to the settings registry.
pub(crate) fn apply_project_layer(
    workspace: Res<Workspace>,
    mut layers: ResMut<SettingsLayers>,
    mut registry: ResMut<SettingsRegistry>,
    mut commands: Commands,
) {
    layers.project = PartialSettings {
        editor: PartialEditorSettings {
            font_size: Some(workspace.font_size_base),
            workspace_preset: if workspace.workspace_preset.is_empty() {
                None
            } else {
                Some(workspace.workspace_preset.clone())
            },
        },
        theme: None, // project-level theme override not yet supported
    };

    *registry = merge(&layers.defaults, &layers.user, &layers.project);
    commands.trigger(SettingsChanged);
    info!("Settings: applied project layer (font_size={})", registry.editor.font_size);
}

/// On exiting the editor, clear the project layer and re-merge.
pub(crate) fn clear_project_layer(
    mut layers: ResMut<SettingsLayers>,
    mut registry: ResMut<SettingsRegistry>,
    mut commands: Commands,
) {
    layers.project = PartialSettings::default();
    *registry = merge(&layers.defaults, &layers.user, &layers.project);
    commands.trigger(SettingsChanged);
    info!("Settings: cleared project layer");
}
```

**Step 2: Create mod.rs**

Create `src/settings/mod.rs`:

```rust
//! Settings plugin.
//!
//! Provides a three-layer settings infrastructure merging compiled defaults,
//! user config (TOML), and project overrides into a typed `SettingsRegistry`.

use bevy::prelude::*;

use crate::contracts::persistence::AppScreen;
use crate::contracts::settings::SettingsRegistry;

mod config;
mod systems;

#[cfg(test)]
mod tests;

/// Internal resource holding the three settings layers for re-merge.
/// Plugin-private — not exposed through contracts.
#[derive(Resource, Debug)]
pub(crate) struct SettingsLayers {
    pub(crate) defaults: config::PartialSettings,
    pub(crate) user: config::PartialSettings,
    pub(crate) project: config::PartialSettings,
}

/// Plugin that manages layered settings.
#[derive(Debug)]
pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        // 1. Build layers.
        let defaults = config::PartialSettings::defaults();
        let user = config::load_user_settings();
        let project = config::PartialSettings::default();

        // 2. Merge and insert.
        let registry = config::merge(&defaults, &user, &project);
        app.insert_resource(registry);
        app.insert_resource(SettingsLayers {
            defaults,
            user,
            project,
        });

        // 3. Project layer lifecycle.
        app.add_systems(OnEnter(AppScreen::Editor), systems::apply_project_layer);
        app.add_systems(OnExit(AppScreen::Editor), systems::clear_project_layer);
    }
}
```

**Step 3: Register the plugin in main.rs**

Add `mod settings;` to the module declarations (alphabetical, after `mod scripting;`).

Add `.add_plugins(settings::SettingsPlugin)` after `.add_plugins(export::ExportPlugin)` and before
`.add_plugins(editor_ui::EditorUiPlugin)`.

**Step 4: Run tests**

Run: `cargo test` Expected: All tests pass, including the new settings tests.

**Step 5: Run clippy**

Run: `cargo clippy --all-targets` Expected: Zero warnings.

**Step 6: Run boundary check**

Run: `mise check:boundary` Expected: No cross-plugin import violations.

**Step 7: Commit**

```
feat(settings): add SettingsPlugin with three-layer merge

SettingsPlugin loads user config from settings.toml, merges defaults +
user + project layers, and inserts SettingsRegistry. Project layer
applied on editor entry, cleared on exit. (ref #173)
```

---

### Task 6: Update Architecture Doc and Plugin Log

**Files:**

- Modify: `docs/architecture.md`
- Modify: `docs/plugins/settings/log.md`

**Step 1: Update architecture.md — plugin load order**

Add settings plugin between ExportPlugin and EditorUiPlugin:

```
14. SettingsPlugin (NEW 0.13.0 — after ExportPlugin, before EditorUiPlugin)
15. EditorUiPlugin (must be last — reads all resources, renders launcher + editor)
```

Update the number for EditorUiPlugin from 14 to 15.

**Step 2: Update architecture.md — dependency graph**

Add to the dependency graph text:

```
settings (contract)    ──→ editor_ui
persistence (contract) ──→ settings

settings: depends on persistence contract (Workspace for project layer)
```

**Step 3: Update plugin log**

Add initial entry to `docs/plugins/settings/log.md`:

```markdown
## Log

### 2026-02-24: Scope 1+5 — Settings infrastructure + contract

- **Decision**: Typed struct fields for SettingsRegistry (not string-keyed dynamic). Simpler,
  type-safe, aligns with "no plugin registration API" no-go.
- **Decision**: Three partial layers with Option<T> fields. Merge is field-by-field
  project.or(user).or(defaults).
- **Decision**: Config dir reuses same #[cfg(feature)] pattern as shortcuts/config.rs.
- **Test results**: merge tests pass — all layer priority scenarios covered.
- **Files**: src/contracts/settings.rs, src/settings/{mod,config,systems,tests}.rs,
  docs/contracts/settings.md
```

**Step 4: Run prettier**

Run: `npx prettier --write docs/architecture.md docs/plugins/settings/log.md`

**Step 5: Commit**

```
docs(settings): update architecture and plugin log for Scope 1+5

Add SettingsPlugin to load order and dependency graph. Record design
decisions in plugin log. (ref #173)
```

---

### Task 7: Full Audit and Scope Completion

**Step 1: Run full check suite**

Run: `mise check` Expected: All checks pass (fmt, clippy, test, deny, typos, taplo, boundary,
unwrap).

**Step 2: Post scope completion comment on pitch issue**

```bash
gh issue comment 173 --body "Scope 1+5 complete (commit <SHA>, +N/-N across M files): Settings infrastructure and contract in place. SettingsRegistry with typed struct fields, three-layer merge (defaults/user/project), PartialSettings with Option<T> for merge semantics. Config loading follows shortcuts/config.rs pattern. 9 tests covering merge priority, TOML deserialization, and edge cases. No abstraction needed — the PartialSettings/merge pattern is simple and direct."
```

Replace `<SHA>` and line counts with actual values from the commits.
