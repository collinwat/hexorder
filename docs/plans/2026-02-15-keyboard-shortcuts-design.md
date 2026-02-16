# Design: Keyboard-First Command Access (#80)

## Open Questions

These questions require user input before implementation begins.

1. **New plugin or extend `editor_ui`?** The pitch touches two areas: camera and editor-ui. A new
   `shortcuts` plugin (registered before `editor_ui`) would own the registry, config loading, and
   palette UI. Alternatively, the registry could be a contract and the palette UI could live in
   `editor_ui`. Recommendation: new `shortcuts` plugin with a new `shortcuts` contract. See
   Architecture section for rationale. **DECIDED**: Accepted recommendation. New `shortcuts`
   plugin + `shortcuts` contract.

2. **Config file format: TOML or JSON?** The pitch says "JSON or TOML." TOML aligns with Rust
   ecosystem conventions and is already a project dependency (taplo for linting). JSON requires no
   new dependency either (serde_json). Recommendation: TOML, using the `toml` crate. Consistent with
   Cargo.toml conventions and human-editable. **DECIDED**: Accepted recommendation. TOML.

3. **Config file location**: `~/.config/hexorder/shortcuts.toml` (XDG on Linux, standard on macOS)?
   Or alongside the binary? Recommendation: use the `dirs` crate for `config_dir()` which resolves
   to `~/Library/Application Support/hexorder/` on macOS. **DECIDED**: Deviates from recommendation.
   Compile-time configuration: a `cfg` flag or build-time constant that switches between a local dev
   path (project-relative) and the standard macOS app bundle path
   (`~/Library/Application Support/hexorder/`). When compiled as a packaged macOS app, use the macOS
   standard path. During local development, use a package-defined local configuration path. The
   `dirs` crate is not needed -- the paths are determined at compile time. See updated Config File
   section.

4. **Command execution model: Events or enum dispatch?** Commands need to trigger actions across
   plugins (save, load, switch tool, zoom). The cleanest Bevy-idiomatic approach is to fire observer
   events. But each command would need its own event type, or we use a single `CommandExecutedEvent`
   with a `CommandId` that each plugin observes and matches. Which pattern is preferred? See the
   Command Abstraction section for the tradeoffs. **DECIDED**: Accepted recommendation. Single
   `CommandExecutedEvent` with `CommandId`.

5. **Fuzzy matching crate**: The command palette needs fuzzy search. Options: `sublime_fuzzy`,
   `nucleo`, or a simple substring/prefix match. For a Small Batch, a simple case-insensitive
   substring filter may suffice. Should we add a fuzzy matching dependency or keep it minimal?
   Recommendation: start with case-insensitive substring matching. Add a fuzzy crate later if
   needed. **DECIDED**: Rejected recommendation. Add a fuzzy matching crate now (e.g.,
   `sublime_fuzzy` or `nucleo`) rather than starting with substring matching. Fuzzy matching is
   expected from a command palette from the start. See updated Command Palette UX section.

6. **Scope of "all existing shortcuts"**: The audit below found 14 distinct shortcuts. The pitch
   says "register all existing shortcuts plus common editor actions." How many new commands beyond
   the migrated shortcuts? Recommendation: migrate the 14 existing shortcuts, add tool mode
   switching (Select/Paint/Place), and keep the initial set small (~18-20 commands). **DECIDED**:
   Rejected recommendation. Target ~25-30 total commands from the start, not ~18-20. Include toggle
   panels, mode switches (Editor/Play), and other discoverable actions so the command palette feels
   comprehensive on launch. See updated Initial Command Set section.

7. **Research #25 status**: The pitch references shortcut management library research (#25). That
   research issue is still open and was not completed in the wiki. Should we do a quick spike on
   existing Bevy shortcut crates before building custom, or proceed with custom implementation?
   Recommendation: proceed custom. The scope is small enough (a `HashMap` registry) that a library
   would add more coupling than value. **DECIDED**: Rejected recommendation. Conduct a quick
   research spike first (a few hours) to evaluate existing Bevy shortcut/input-action crates before
   committing to a custom implementation. If a crate fits well, adopt it; otherwise, proceed custom
   with confidence that alternatives were evaluated. See Pre-Implementation section below.

---

## Overview

This pitch adds three capabilities to Hexorder:

1. **Shortcut Registry** -- a centralized `ShortcutRegistry` resource mapping key combinations to
   named commands. All existing scattered shortcuts migrate here.
2. **Command Palette** -- Cmd+K opens a floating egui search panel with fuzzy-match filtering,
   showing command names and their shortcuts.
3. **Customization** -- a TOML config file in the user's config directory for overriding default
   bindings. Loaded at startup, merged over defaults. No UI editor.

---

## Existing Shortcuts Audit

Every keyboard shortcut currently in the codebase. These all need to migrate to the registry.

### Camera Plugin (`src/camera/systems.rs`)

| Key(s)         | Modifier | Action                    | System           | Run Condition                        |
| -------------- | -------- | ------------------------- | ---------------- | ------------------------------------ |
| W / ArrowUp    | none     | Pan camera up             | `keyboard_pan`   | `not(egui_wants_any_keyboard_input)` |
| S / ArrowDown  | none     | Pan camera down           | `keyboard_pan`   | `not(egui_wants_any_keyboard_input)` |
| A / ArrowLeft  | none     | Pan camera left           | `keyboard_pan`   | `not(egui_wants_any_keyboard_input)` |
| D / ArrowRight | none     | Pan camera right          | `keyboard_pan`   | `not(egui_wants_any_keyboard_input)` |
| =              | none     | Zoom in                   | `view_shortcuts` | `not(egui_wants_any_keyboard_input)` |
| -              | none     | Zoom out                  | `view_shortcuts` | `not(egui_wants_any_keyboard_input)` |
| C              | none     | Center grid               | `view_shortcuts` | `not(egui_wants_any_keyboard_input)` |
| F              | none     | Fit grid to viewport      | `view_shortcuts` | `not(egui_wants_any_keyboard_input)` |
| 0              | none     | Fit + center (reset view) | `view_shortcuts` | `not(egui_wants_any_keyboard_input)` |

**Note**: WASD panning already guards against Cmd modifier
(`keys.any_pressed([SuperLeft, SuperRight])` returns early). This prevents Cmd+S from also panning.

### Persistence Plugin (`src/persistence/systems.rs`)

| Key(s) | Modifier  | Action      | System               | Run Condition                 |
| ------ | --------- | ----------- | -------------------- | ----------------------------- |
| S      | Cmd       | Save        | `keyboard_shortcuts` | `in_state(AppScreen::Editor)` |
| S      | Cmd+Shift | Save As     | `keyboard_shortcuts` | `in_state(AppScreen::Editor)` |
| O      | Cmd       | Open        | `keyboard_shortcuts` | `in_state(AppScreen::Editor)` |
| N      | Cmd       | New project | `keyboard_shortcuts` | `in_state(AppScreen::Editor)` |

### Hex Grid Plugin (`src/hex_grid/systems.rs`)

| Key(s) | Modifier | Action       | System               | Run Condition                        |
| ------ | -------- | ------------ | -------------------- | ------------------------------------ |
| Escape | none     | Deselect hex | `deselect_on_escape` | `not(egui_wants_any_keyboard_input)` |

### Editor UI Plugin (`src/editor_ui/`)

No keyboard shortcuts found in editor_ui systems. Tool mode switching (Select/Paint/Place) is
mouse-only via `selectable_label` clicks in `render_tool_mode`.

### Summary

**Total existing shortcuts**: 14 key bindings across 3 plugins (camera: 9, persistence: 4, hex_grid:
1).

**Two categories**:

- **Held keys** (continuous): WASD/arrows for camera pan. These read `pressed()` every frame and
  scale with `delta_time`. They do NOT fit the command palette model (you cannot "execute" a pan
  from the palette). They should still register in the shortcut registry for discoverability and
  customization, but their execution is continuous, not discrete.
- **Just-pressed keys** (discrete): Everything else. These read `just_pressed()` and fire once.
  These map cleanly to palette commands.

---

## Architecture

### Decision: New `shortcuts` Plugin + New `shortcuts` Contract

**Rationale**: The shortcut registry is consumed by multiple plugins (camera, persistence, hex_grid,
editor_ui). Per the constitution, shared types must live in `src/contracts/`. The palette UI and
registry management logic warrant their own plugin module rather than bloating `editor_ui`.

**New module**: `src/shortcuts/` with plugin struct `ShortcutsPlugin`.

**New contract**: `src/contracts/shortcuts.rs` (spec: `docs/contracts/shortcuts.md`).

**Plugin load order**: ShortcutsPlugin must load **before** all plugins that register shortcuts
(camera, hex_grid, persistence) and before EditorUiPlugin. Insert at position 3 (after CameraPlugin,
before GameSystemPlugin). Since registration happens in `build()` (synchronous), the registry
resource exists immediately for subsequent plugins.

Proposed load order update in `main.rs`:

```
1. DefaultPlugins
2. HexGridPlugin
3. ShortcutsPlugin      <-- NEW
4. CameraPlugin
5. GameSystemPlugin
6. OntologyPlugin
7. CellPlugin
8. UnitPlugin
9. RulesEnginePlugin
10. ScriptingPlugin
11. PersistencePlugin
12. EditorUiPlugin
```

### Decision: Contract Types

The `shortcuts` contract exposes the following shared types:

```rust
// src/contracts/shortcuts.rs

/// Identifies a registered command.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommandId(pub &'static str);

/// A key combination: a primary key plus optional modifiers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub key: KeyCode,
    pub modifiers: Modifiers,
}

/// Modifier key flags.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Modifiers {
    pub cmd: bool,    // Super (Cmd on macOS)
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
}

/// A registered command with metadata.
#[derive(Debug, Clone)]
pub struct CommandEntry {
    pub id: CommandId,
    pub name: String,
    pub description: String,
    pub bindings: Vec<KeyBinding>,
    pub category: CommandCategory,
    /// Whether this is a continuous (held) command vs discrete (just_pressed).
    pub continuous: bool,
}

/// Command grouping for palette display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandCategory {
    Camera,
    File,
    Edit,
    View,
    Tool,
    Mode,
}

/// Central registry of all commands and their bindings.
#[derive(Resource, Debug, Default)]
pub struct ShortcutRegistry {
    commands: Vec<CommandEntry>,
    // Lookup: binding -> command ID (for fast matching on key press)
    binding_map: HashMap<KeyBinding, CommandId>,
}

/// Observer event fired when a command is executed (via shortcut or palette).
#[derive(Event, Debug)]
pub struct CommandExecutedEvent {
    pub command_id: CommandId,
}

/// Resource controlling command palette visibility.
#[derive(Resource, Debug, Default)]
pub struct CommandPaletteState {
    pub open: bool,
    pub query: String,
}
```

### Command Execution Model

**Recommended approach**: A single `CommandExecutedEvent` with a `CommandId`. Each plugin observes
this event and matches on the command ID it cares about.

```rust
// In camera plugin:
app.add_observer(|trigger: On<CommandExecutedEvent>, mut camera_state: ResMut<CameraState>| {
    match trigger.event().command_id.0 {
        "camera.center" => { /* center logic */ }
        "camera.fit" => { /* fit logic */ }
        "camera.reset_view" => { /* reset logic */ }
        "camera.zoom_in" => { /* zoom in */ }
        "camera.zoom_out" => { /* zoom out */ }
        _ => {} // Not our command
    }
});
```

**Why not separate events per command?** With ~25-30 commands, that would mean 25-30 event types
plus 25-30 observer registrations just for initial scope. A single event with string matching is
simpler and scales better. The string matching cost is negligible (a few comparisons per keypress).

**Why observer events (not messages)?** Commands are discrete, immediate actions. The observer
pattern (`Event` + `commands.trigger()`) is correct per the Bevy guide: "Immediate response to a
discrete action."

**Continuous commands (WASD pan)**: These cannot use the event model. WASD commands register in the
registry for discoverability and customization, but their execution stays in the existing
`keyboard_pan` system. The system reads the registry to look up which `KeyCode` maps to
"camera.pan_up" etc., rather than hardcoding `KeyCode::KeyW`.

### Migration Strategy

Each plugin migrates in two steps:

1. **Registration**: In `build()`, register commands with the `ShortcutRegistry`.
2. **Execution**: Replace hardcoded `keys.just_pressed(KeyCode::X)` checks with either:
    - Observer on `CommandExecutedEvent` (for discrete commands)
    - Registry lookup for the bound `KeyCode` (for continuous commands)

The `ShortcutsPlugin` owns a single system in `Update` that checks `ButtonInput<KeyCode>` every
frame, matches against the `binding_map`, and fires `CommandExecutedEvent` for any matched
just-pressed binding. This replaces the scattered `just_pressed` checks in individual plugins.

For continuous commands, individual plugins still read `ButtonInput<KeyCode>` but look up the bound
key from the registry instead of hardcoding it.

---

## New Contracts

### `shortcuts` (`docs/contracts/shortcuts.md`, `src/contracts/shortcuts.rs`)

**Purpose**: Shared types for the centralized shortcut registry, command execution events, and
command palette state.

**Producers**: `ShortcutsPlugin` (registry resource, execution event dispatch)

**Consumers**: All plugins that register or respond to commands (camera, persistence, hex_grid,
editor_ui)

**Types**:

- `CommandId` -- string-based command identifier
- `KeyBinding` -- key + modifier combination
- `Modifiers` -- modifier flags (cmd, shift, alt, ctrl)
- `CommandEntry` -- full command metadata
- `CommandCategory` -- grouping enum for palette display
- `ShortcutRegistry` -- the central resource (`Resource`)
- `CommandExecutedEvent` -- observer event fired on command execution (`Event`)
- `CommandPaletteState` -- palette open/query state (`Resource`)

---

## Plugin Changes

### New: `shortcuts` Plugin (`src/shortcuts/`)

**Module structure**:

```
src/shortcuts/
  mod.rs         -- ShortcutsPlugin definition
  systems.rs     -- shortcut matching system, Cmd+K intercept, config loading
  tests.rs       -- unit tests
```

**Plugin responsibilities**:

1. Insert `ShortcutRegistry` resource (in `build()`, immediate)
2. Insert `CommandPaletteState` resource (in `build()`, immediate)
3. Load user config file at startup (Startup system)
4. Run shortcut matching system in `Update` (before all consumer plugins)
5. Intercept Cmd+K to toggle command palette

**Systems**:

- `load_user_config` (Startup): reads the compile-time config path (local dev:
  `./config/shortcuts.toml`; macOS app: `~/Library/Application Support/hexorder/shortcuts.toml`),
  parses overrides, merges into registry. Missing file is not an error (use defaults). Invalid
  entries log warnings.

- `match_shortcuts` (Update): reads `ButtonInput<KeyCode>`, checks each just-pressed key against
  `ShortcutRegistry.binding_map`, fires `CommandExecutedEvent` for matches. Gated by
  `not(egui_wants_any_keyboard_input)` **except** for Cmd+K which must always work. Skips all
  matching while `CommandPaletteState.open` is true (prevents search typing from triggering
  commands).

- `intercept_command_palette_toggle` (PreUpdate or very early Update): checks for Cmd+K regardless
  of egui focus. This must run before egui processes input. Toggles `CommandPaletteState.open`.

**Cmd+K interception strategy**: The pitch identifies this as a rabbit hole. egui consumes keyboard
input when a text field is focused. The command palette itself is a text field. Solution:

- Check `ButtonInput<KeyCode>` for Cmd+K in a system that runs in `PreUpdate` (before egui's
  `EguiPreUpdateSet::ProcessInput`). At this point, Bevy has populated `ButtonInput` but egui has
  not consumed it yet.
- When Cmd+K is detected, toggle `CommandPaletteState.open` and clear the query.
- The `match_shortcuts` system skips all other shortcut processing while the palette is open (to
  prevent typing in the search field from firing shortcuts).

Alternative (if Bevy absorb clears ButtonInput too early): detect Cmd+K inside the egui system using
`ctx.input(|i| i.key_pressed(egui::Key::K) && i.modifiers.command)`. See Risk Assessment.

**Shortcut conflict handling**: The `ShortcutRegistry::register()` method checks for duplicate
bindings. If a duplicate is found, it logs a warning and the new registration wins (last-registered
wins). User overrides from the config file are applied after all plugin registrations, so they
always take priority.

### Camera Plugin (`src/camera/`) -- Migration

**Registration** (in `build()`):

```rust
let mut registry = app.world_mut().resource_mut::<ShortcutRegistry>();
registry.register(CommandEntry {
    id: CommandId("camera.pan_up"),
    name: "Pan Up".into(),
    bindings: vec![
        KeyBinding::new(KeyCode::KeyW, Modifiers::NONE),
        KeyBinding::new(KeyCode::ArrowUp, Modifiers::NONE),
    ],
    category: CommandCategory::Camera,
    continuous: true,
    ..
});
// ... repeat for all 9 camera shortcuts
```

**System changes**:

- `keyboard_pan`: Instead of hardcoding `KeyCode::KeyW`, look up the bound keys for
  `"camera.pan_up"` from the registry. This allows user customization of pan keys. Implementation:
  read `Res<ShortcutRegistry>`, call `registry.bindings_for("camera.pan_up")`, check
  `keys.any_pressed(bound_keys)`.

- `view_shortcuts`: Remove entirely. These are discrete commands that migrate to
  `CommandExecutedEvent` observers. Add an observer in `build()` that handles `"camera.center"`,
  `"camera.fit"`, `"camera.reset_view"`, `"camera.zoom_in"`, `"camera.zoom_out"`.

### Hex Grid Plugin (`src/hex_grid/`) -- Migration

**Registration**: Register `"edit.deselect"` (Escape key).

**System changes**: `deselect_on_escape` migrates to a `CommandExecutedEvent` observer.

### Persistence Plugin (`src/persistence/`) -- Migration

**Registration**: Register `"file.save"` (Cmd+S), `"file.save_as"` (Cmd+Shift+S), `"file.open"`
(Cmd+O), `"file.new"` (Cmd+N).

**System changes**: Remove `keyboard_shortcuts` system. Add a `CommandExecutedEvent` observer that
fires `SaveRequestEvent`, `LoadRequestEvent`, or `NewProjectEvent` as appropriate.

### Editor UI Plugin (`src/editor_ui/`) -- Palette UI + Tool Shortcuts

**New system**: `command_palette_system` in `EguiPrimaryContextPass`. Renders the floating command
palette when `CommandPaletteState.open` is true.

**Palette UI sketch**:

```rust
egui::Window::new("Command Palette")
    .fixed_pos(centered_position)    // centered horizontally, ~1/4 from top
    .fixed_size([400.0, 300.0])      // or auto-size
    .title_bar(false)                 // no title bar, clean floating panel
    .frame(custom_frame)             // match brand palette
    .show(ctx, |ui| {
        // Search field (auto-focused)
        let response = ui.text_edit_singleline(&mut palette_state.query);

        // Filtered command list (fuzzy-matched, ranked by score)
        for entry in registry.fuzzy_search(&palette_state.query) {
            ui.horizontal(|ui| {
                let text = egui::RichText::new(&entry.name);
                if ui.selectable_label(false, text).clicked() {
                    // capture command_id for execution after closure
                }
                // Right-aligned shortcut hint
                if let Some(binding) = entry.bindings.first() {
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            ui.label(
                                egui::RichText::new(binding.display_string())
                                    .color(egui::Color32::from_gray(120))
                                    .small()
                            );
                        },
                    );
                }
            });
        }
    });
// Fire CommandExecutedEvent OUTSIDE the closure (multi-pass safe)
```

**Tool mode observer**: `editor_ui` adds an observer on `CommandExecutedEvent` to handle
`"tool.select"`, `"tool.paint"`, `"tool.place"` by writing to `ResMut<EditorTool>`.

**Mode switching observer**: `editor_ui` (or a top-level system) observes `"mode.editor"` and
`"mode.play"` to switch `AppScreen` state.

**Panel toggle observers**: `editor_ui` observes `"view.toggle_inspector"`, `"view.toggle_toolbar"`,
and `"view.toggle_grid_overlay"` to toggle panel visibility resources.

---

## Command Palette UX

### Activation

- **Cmd+K** opens the palette from anywhere, including when egui text fields have focus
- **Esc** closes the palette
- **Enter** on highlighted command executes it
- Clicking a command executes it
- Clicking outside the palette closes it (egui window loss of focus)

### Search Behavior

- **Fuzzy matching** on command name using a dedicated crate (e.g., `sublime_fuzzy` or `nucleo`).
  Results are ranked by match score, with the best matches at the top. This provides the standard
  command-palette experience users expect (e.g., typing "sv" matches "Save", "svs" matches "Save
  As").
- Results grouped by category (Camera, File, Edit, View, Tool, Mode) with category headers
- Each result shows: command name (left), shortcut hint (right, dimmed)
- Continuous commands (WASD pan) are excluded from palette results (they cannot be "executed" as a
  discrete action)

### Visual Design

- Floating window, no title bar, centered horizontally, positioned ~25% from top
- Dark background matching brand palette (`from_gray(25)` panel fill)
- Search field auto-focused on open
- Selected row highlighted with teal accent (`from_rgb(0, 92, 128)`)
- Shortcut hints in secondary text color (`from_gray(120)`)

### egui Integration Details

**Schedule**: The palette system runs in `EguiPrimaryContextPass`, same as other editor UI systems.
It should run before `editor_panel_system` so the palette renders on top.

**Input passthrough**: While the palette is open, the search field captures keyboard input via
egui's normal text field handling. The `match_shortcuts` system (in `Update`) must check
`CommandPaletteState.open` and skip shortcut matching while the palette is open (otherwise typing
"s" in the search box would trigger Save).

**Focus management**: Use `response.request_focus()` on the text field when the palette first opens.
Track "just opened" state to call this once.

**Multi-pass safety**: Side effects (firing `CommandExecutedEvent`) must happen outside the egui
closure. Capture the selected command ID in a local variable, then fire the event after `.show()`
returns.

---

## Config File

### Location

The config file path is determined at **compile time** via a Cargo feature flag or build-time
constant. This supports two deployment modes:

| Mode              | Compile Flag                             | Config Path                                             |
| ----------------- | ---------------------------------------- | ------------------------------------------------------- |
| Local development | default (no flag)                        | `./config/shortcuts.toml` (project-relative)            |
| macOS app bundle  | `--cfg macos_app` or feature `macos-app` | `~/Library/Application Support/hexorder/shortcuts.toml` |

**Implementation approach**: Define a `config_dir()` function that uses conditional compilation:

```rust
fn config_dir() -> PathBuf {
    #[cfg(feature = "macos-app")]
    {
        let home = std::env::var("HOME").expect("HOME not set");
        PathBuf::from(home)
            .join("Library/Application Support/hexorder")
    }

    #[cfg(not(feature = "macos-app"))]
    {
        PathBuf::from("config")
    }
}
```

The feature flag is declared in `Cargo.toml` under `[features]`. The macOS app packaging step
(future work) will compile with `--features macos-app`. During development, the default local path
is used.

No `dirs` crate dependency is needed -- the paths are simple constants determined at compile time.

### Format

```toml
# Hexorder Shortcut Overrides
# Only include bindings you want to change. Missing commands keep their defaults.
# Format: command_id = "modifier+key"
# Modifiers: cmd, shift, alt, ctrl (combine with +)
# Key names match Bevy KeyCode variant names (lowercase): key_s, key_w, escape, etc.

[bindings]
"camera.center" = "key_h"          # remap center from C to H
"camera.zoom_in" = "cmd+equal"     # add Cmd modifier to zoom
"file.save" = "cmd+key_s"          # explicit (same as default)
"tool.select" = "digit1"           # number keys for tool switching
"tool.paint" = "digit2"
"tool.place" = "digit3"

# Multiple bindings use array syntax:
"camera.pan_up" = ["key_w", "arrow_up"]

# To unbind a command, set it to empty string:
# "camera.fit" = ""
```

### Parsing

- Read the file at startup. Missing file = use all defaults (not an error).
- Parse with the `toml` crate (add to Cargo.toml dependencies).
- For each entry, parse the value string into a `KeyBinding`:
    - Split on `+` to extract modifiers and key name
    - Map modifier names to `Modifiers` flags
    - Map key name to `KeyCode` via a lookup table
- Invalid entries log a warning and are skipped.
- Apply overrides after all plugins have registered their defaults.

### New Dependencies

```toml
# Cargo.toml additions
toml = "0.8"          # TOML parsing for shortcut config
sublime_fuzzy = "0.7" # fuzzy matching for command palette search
```

Note: `dirs` crate is **not** needed -- config path is compile-time (see Config File Location). The
fuzzy crate choice (`sublime_fuzzy` vs `nucleo`) will be confirmed during the research spike.

---

## Initial Command Set

### Migrated from Existing Code (14 bindings)

| Command ID          | Name        | Default Binding | Category | Type       |
| ------------------- | ----------- | --------------- | -------- | ---------- |
| `camera.pan_up`     | Pan Up      | W / ArrowUp     | Camera   | continuous |
| `camera.pan_down`   | Pan Down    | S / ArrowDown   | Camera   | continuous |
| `camera.pan_left`   | Pan Left    | A / ArrowLeft   | Camera   | continuous |
| `camera.pan_right`  | Pan Right   | D / ArrowRight  | Camera   | continuous |
| `camera.zoom_in`    | Zoom In     | =               | Camera   | discrete   |
| `camera.zoom_out`   | Zoom Out    | -               | Camera   | discrete   |
| `camera.center`     | Center View | C               | Camera   | discrete   |
| `camera.fit`        | Fit to Grid | F               | Camera   | discrete   |
| `camera.reset_view` | Reset View  | 0               | Camera   | discrete   |
| `file.save`         | Save        | Cmd+S           | File     | discrete   |
| `file.save_as`      | Save As     | Cmd+Shift+S     | File     | discrete   |
| `file.open`         | Open        | Cmd+O           | File     | discrete   |
| `file.new`          | New Project | Cmd+N           | File     | discrete   |
| `edit.deselect`     | Deselect    | Escape          | Edit     | discrete   |

**Note on dual-key bindings**: Camera pan commands currently accept two keys each (e.g., W and
ArrowUp). The registry supports multiple bindings per command via `Vec<KeyBinding>`. This preserves
existing behavior. The config file uses array syntax for multiple bindings.

### New Commands (~14-16 bindings, targeting ~25-30 total)

#### Tool Switching

| Command ID    | Name        | Default Binding | Category | Type     |
| ------------- | ----------- | --------------- | -------- | -------- |
| `tool.select` | Select Tool | 1               | Tool     | discrete |
| `tool.paint`  | Paint Tool  | 2               | Tool     | discrete |
| `tool.place`  | Place Tool  | 3               | Tool     | discrete |

Tool mode bindings (1/2/3) are new. These add keyboard-first tool switching that currently requires
mouse clicks in the editor panel. Number keys are standard in design tools (Maya uses Q/W/E/R,
Photoshop uses single letters).

#### View / UI

| Command ID                 | Name                   | Default Binding | Category | Type     |
| -------------------------- | ---------------------- | --------------- | -------- | -------- |
| `palette.toggle`           | Command Palette        | Cmd+K           | View     | discrete |
| `view.toggle_inspector`    | Toggle Inspector Panel | Cmd+I           | View     | discrete |
| `view.toggle_toolbar`      | Toggle Toolbar         | Cmd+T           | View     | discrete |
| `view.toggle_grid_overlay` | Toggle Grid Overlay    | G               | View     | discrete |

#### Mode Switching

| Command ID    | Name        | Default Binding | Category | Type     |
| ------------- | ----------- | --------------- | -------- | -------- |
| `mode.editor` | Editor Mode | Cmd+1           | Mode     | discrete |
| `mode.play`   | Play Mode   | Cmd+2           | Mode     | discrete |

#### Edit Actions

| Command ID        | Name             | Default Binding    | Category | Type     |
| ----------------- | ---------------- | ------------------ | -------- | -------- |
| `edit.undo`       | Undo             | Cmd+Z              | Edit     | discrete |
| `edit.redo`       | Redo             | Cmd+Shift+Z        | Edit     | discrete |
| `edit.select_all` | Select All       | Cmd+A              | Edit     | discrete |
| `edit.delete`     | Delete Selection | Backspace / Delete | Edit     | discrete |

#### Navigation / Discovery

| Command ID               | Name              | Default Binding | Category | Type     |
| ------------------------ | ----------------- | --------------- | -------- | -------- |
| `view.zoom_to_selection` | Zoom to Selection | Z               | View     | discrete |
| `view.toggle_fullscreen` | Toggle Fullscreen | Cmd+F           | View     | discrete |

**Total new commands**: ~15 (exact count depends on which panels and modes exist at implementation
time). Combined with the 14 migrated shortcuts, this reaches the ~25-30 target. The goal is for the
command palette to feel comprehensive and discoverable from day one, surfacing actions users might
not know exist.

**Note**: Some of these commands (undo/redo, select all, delete) may not have full implementations
yet. They should still be registered in the palette for discoverability, with their observers
logging a "not yet implemented" message or being no-ops until the backing feature exists.

---

## Pre-Implementation: Research Spike (Research #25)

Before building the shortcuts system, spend **a few hours** evaluating existing Bevy shortcut and
input-action crates. This addresses the open Research #25 issue and ensures we are not reinventing
the wheel.

**Scope**: Evaluate crates for compatibility with Bevy 0.18, suitability for our use case (registry,
customization, command palette integration), and maintenance health.

**Candidates to evaluate**:

- `leafwing-input-manager` -- the most established Bevy input-action mapping crate
- `bevy_input_actionmap` -- simpler action mapping
- Any other crates found during the search

**Evaluation criteria**:

1. Bevy 0.18 support (or reasonable migration path)
2. Supports key binding customization at runtime / from config
3. Does not conflict with egui input handling
4. Supports our two-category model (continuous held keys + discrete just-pressed)
5. License compatibility (MIT/Apache-2.0)
6. Active maintenance

**Decision**: If a crate meets all criteria, adopt it and simplify the `shortcuts` contract to wrap
the crate's types. If no crate fits, proceed with the custom `HashMap` registry as designed above,
with the confidence that alternatives were evaluated.

**Output**: A brief summary posted to the pitch issue (#80) and recorded in the plugin log
(`docs/plugins/shortcuts/log.md`).

---

## First Piece

**Build the shortcut registry and migrate one plugin end-to-end.**

Specifically:

1. Create the `shortcuts` contract (spec + code)
2. Create the `ShortcutsPlugin` with `ShortcutRegistry` resource
3. Implement `match_shortcuts` system (reads `ButtonInput<KeyCode>`, fires `CommandExecutedEvent`)
4. Migrate the persistence plugin's 4 shortcuts (discrete, simple, well-tested)
5. Verify: pressing Cmd+S still saves, but now goes through the registry

This is the most core, novel piece. It proves the registry + event dispatch pattern works before
migrating the more complex camera shortcuts or building the palette UI. If this works, the rest is
mechanical.

**Second piece**: Migrate camera discrete shortcuts (zoom, center, fit, reset). Then migrate camera
continuous shortcuts (pan). This validates the continuous-command pattern.

**Third piece**: Command palette UI. By this point all commands are registered and the palette just
reads the registry and fires events.

**Fourth piece**: Config file loading. Pure additive -- reads a file, merges overrides.

---

## Risk Assessment

### Low Risk

- **Contract definition**: Small, well-scoped types. No existing contracts need modification.
- **Config file loading**: Pure additive. Missing file = defaults. Cannot break existing behavior.
- **Tool mode shortcuts**: Simple observer that writes to `EditorTool` resource.
- **Migration of discrete shortcuts**: Straightforward event dispatch replacement.

### Medium Risk

- **Cmd+K interception**: Must work even when egui has keyboard focus. The proposed PreUpdate timing
  should work because `ButtonInput<KeyCode>` is populated in Bevy's input stage before egui's
  `EguiPreUpdateSet::ProcessInput`. However, the `enable_absorb_bevy_input_system = true` setting
  (currently enabled in `src/editor_ui/mod.rs` for text field input) may clear `ButtonInput` before
  our system reads it. **Mitigation**: test this interaction early. If absorb clears the buffer too
  early, use one of the fallback approaches below.

- **Continuous shortcut migration (WASD)**: These systems read `pressed()` every frame and scale
  with delta time. The registry lookup must be cheap (it is -- one HashMap lookup per frame per
  continuous command). **Mitigation**: cache the resolved `KeyCode` in a `Local<>` instead of
  looking up every frame.

- **Command palette egui rendering**: Rendering a floating window in egui is straightforward, but
  the interaction between palette focus, Escape to close, and Enter to execute needs careful
  testing. Multi-pass mode means the closure may run multiple times -- side effects (firing events)
  must happen outside the closure. **Mitigation**: capture action state in the closure, fire events
  after `.show()` returns.

### Higher Risk

- **Plugin load order**: `ShortcutsPlugin` must be registered before plugins that register
  shortcuts. Since registration happens in `build()` (synchronous), this is just a matter of correct
  ordering in `main.rs`. If a plugin tries to register before the registry exists, it will panic.
  **Mitigation**: defensive `Option<ResMut<ShortcutRegistry>>` in registration, or document the
  ordering requirement clearly.

- **`enable_absorb_bevy_input_system` interaction**: This is the biggest unknown. When egui absorb
  is enabled and a text field is focused, Bevy's `ButtonInput` may be cleared. The shortcut system
  relies on `ButtonInput`. If absorb clears it before our system runs, we cannot detect Cmd+K.
  **Mitigation options**:
    1. Read `MessageReader<KeyboardInput>` (raw events, not affected by absorb) instead of
       `ButtonInput<KeyCode>` for the Cmd+K check specifically.
    2. Use egui's own input system: check
       `ctx.input(|i| i.key_pressed(egui::Key::K) && i.modifiers.command)` inside the egui system.
    3. Disable absorb and rely solely on run conditions (but this would break text input in editor
       fields).

    Option 2 is likely the most robust: detect Cmd+K inside the egui system itself, since egui
    already has the keyboard event. The palette toggle becomes an egui-level check rather than a
    Bevy-level check.

---

## Dependency Graph Update

After implementation, add to `docs/architecture.md`:

```
shortcuts (contract) --> camera
shortcuts (contract) --> hex_grid
shortcuts (contract) --> persistence
shortcuts (contract) --> editor_ui
```

Updated plugin dependencies:

```
camera: depends on shortcuts contract
hex_grid: depends on shortcuts + validation contracts
persistence: depends on shortcuts + game_system + hex_grid + ontology + persistence contracts
editor_ui: depends on shortcuts + hex_grid + game_system + ontology + validation + persistence contracts
```

Plugin load order in `main.rs`:

```
1. DefaultPlugins
2. HexGridPlugin
3. ShortcutsPlugin      <-- NEW (must be before consumers)
4. CameraPlugin
5. GameSystemPlugin
6. OntologyPlugin
7. CellPlugin
8. UnitPlugin
9. RulesEnginePlugin
10. ScriptingPlugin
11. PersistencePlugin
12. EditorUiPlugin
```

---

## Test Plan

### Unit Tests (`src/shortcuts/tests.rs`)

1. Registry accepts command registration and stores entries
2. Duplicate binding logs warning, last-registered wins
3. `binding_map` lookup returns correct `CommandId` for a `KeyBinding`
4. Fuzzy search function filters and ranks commands by match score
5. Continuous commands are excluded from search results
6. Config file parsing: valid TOML produces correct overrides
7. Config file parsing: invalid entries are skipped with warning
8. Config file parsing: missing file returns empty overrides
9. Modifier parsing: "cmd+shift+key_s" parses correctly
10. Key name parsing: all used `KeyCode` variants are mapped

### Integration Tests (`src/main.rs::integration_tests`)

1. `ShortcutsPlugin` + `PersistencePlugin`: Cmd+S triggers `SaveRequestEvent` via registry
2. `ShortcutsPlugin` + `CameraPlugin`: zoom command updates `CameraState.target_scale`
3. Registry resource exists before first update (immediate insertion in `build()`)

### Manual UAT

1. Launch app, press Cmd+K -- command palette appears centered
2. Type "sv" -- fuzzy match results show Save and Save As ranked by score, with shortcut hints
3. Press Enter on Save -- save dialog appears, palette closes
4. Press Escape -- palette closes without action
5. Press Cmd+S -- save works as before (regression)
6. Press 1/2/3 -- tool mode switches
