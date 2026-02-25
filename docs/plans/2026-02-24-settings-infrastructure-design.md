# Settings Infrastructure Design (Scope 1 + 5)

**Pitch**: #173 — Settings infrastructure **Branch**: `0.13.0-settings` **Date**: 2026-02-24

## Decision: Typed Struct Fields

The `SettingsRegistry` uses concrete typed fields rather than string-keyed dynamic values. Three
layers use `Option<T>` partial structs for merge semantics. This is type-safe, avoids runtime type
errors, and aligns with the "no plugin-specific settings registration API" no-go.

## Contract Types (`src/contracts/settings.rs`)

```rust
#[derive(Resource, Debug, Clone)]
pub struct SettingsRegistry {
    pub editor: EditorSettings,
    pub active_theme: String, // "brand" = default
}

#[derive(Debug, Clone)]
pub struct EditorSettings {
    pub font_size: f32,           // default 15.0, range 10.0-24.0
    pub workspace_preset: String, // default "" (= MapEditing)
}

#[derive(Event, Debug, Clone)]
pub struct SettingsChanged;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeDefinition {
    pub name: String,
    pub bg_deep: [u8; 3],
    pub bg_panel: [u8; 3],
    pub bg_surface: [u8; 3],
    pub widget_inactive: [u8; 3],
    pub widget_hovered: [u8; 3],
    pub widget_active: [u8; 3],
    pub accent_primary: [u8; 3],
    pub accent_secondary: [u8; 3],
    pub text_primary: [u8; 3],
    pub text_secondary: [u8; 3],
    pub border: [u8; 3],
    pub danger: [u8; 3],
    pub success: [u8; 3],
}
```

## Plugin Structure

```
src/settings/
  mod.rs          # SettingsPlugin, build()
  config.rs       # config_dir(), load_user_settings(), merge logic
  systems.rs      # apply_project_layer, clear_project_layer
  tests.rs        # Unit tests
```

**Load order**: After ExportPlugin, before EditorUiPlugin. Theme must resolve before editor_ui
renders.

## Three-Layer Merge

```
Layer 3 (project)  -> font_size: Some(18.0)  -> wins
Layer 2 (user)     -> font_size: Some(16.0)  -> overridden
Layer 1 (defaults) -> font_size: Some(15.0)  -> fallback
Result:              font_size = 18.0
```

Internal `PartialSettings` struct (plugin-private, `Option<T>` fields). Merge is field-by-field:
`project.or(user).or(defaults)`.

**Merge triggers**:

- Startup: defaults + user -> insert SettingsRegistry
- `OnEnter(AppScreen::Editor)`: read Workspace -> apply project layer -> re-merge -> fire
  SettingsChanged
- `OnExit(AppScreen::Editor)`: clear project layer -> re-merge -> fire SettingsChanged

## Config File Format

`~/.../hexorder/settings.toml`:

```toml
[editor]
font_size = 16.0
workspace_preset = "unit_design"

theme = "brand"
```

Config directory: same `#[cfg(feature)]` pattern as `shortcuts/config.rs`.

- `macos-app` -> `~/Library/Application Support/hexorder/`
- Otherwise -> `config/`

## Theme Files

`~/.../hexorder/themes/solarized.toml`:

```toml
name = "Solarized Dark"
bg_deep = [0, 43, 54]
bg_panel = [7, 54, 66]
# ...
```

The default "brand" theme is constructed from `BrandTheme` constants in code.

## SettingsPlugin::build()

1. Build defaults layer (hardcoded `PartialSettings`)
2. Load user layer from `settings.toml` (graceful missing-file, same as shortcuts)
3. Merge defaults + user -> `SettingsRegistry`
4. Insert resource
5. Add systems for project layer lifecycle
