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

### `SettingsReady`

`SystemSet` indicating the settings registry has been populated for a state transition. Consumer
plugins schedule their restore systems `.after(SettingsReady)` on `OnEnter(AppScreen::Editor)`.

### `SettingsChanged`

Observer event fired when any settings layer changes. No fields — observers re-read
`Res<SettingsRegistry>` for updated values.

### `ThemeLibrary`

Resource holding all available themes. Loaded at startup by `SettingsPlugin`. Brand theme is always
present as the first entry.

| Field    | Type                   | Description                         |
| -------- | ---------------------- | ----------------------------------- |
| `themes` | `Vec<ThemeDefinition>` | Available themes, brand theme first |

Methods:

- `find(&self, name: &str) -> Option<&ThemeDefinition>` — look up a theme by name

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
- `ThemeLibrary` is inserted during `SettingsPlugin::build()` (brand theme always present)
- `SettingsPlugin` must be registered before `EditorUiPlugin` in `main.rs`
- `SettingsChanged` is fired via `commands.trigger()` (observer event, not deprecated EventWriter)
- Missing user config file is not an error — all defaults are used
- `SettingsReady` runs on `OnEnter(AppScreen::Editor)` — consumers use `.after(SettingsReady)`

## Changelog

| Date       | Change              | Reason                                  |
| ---------- | ------------------- | --------------------------------------- |
| 2026-02-24 | Initial definition  | Pitch #173 — settings infrastructure    |
| 2026-02-24 | Add `SettingsReady` | Scope 2 — system ordering for consumers |
| 2026-02-24 | Add `ThemeLibrary`  | Scope 3 — custom theme loading          |
