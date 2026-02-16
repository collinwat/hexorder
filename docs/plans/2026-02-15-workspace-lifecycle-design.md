# Design: Workspace Lifecycle (#53)

## Open Questions

1. **Workspace name in file format**: DECIDED -- Derive from filename when loading v2 files. When a
   v2 file (no `name` field) is loaded, the workspace name is derived from the `.hexorder` filename
   (strip the extension, use the stem as the name). For new projects created via the launcher, the
   user must provide a name -- there is no "Untitled" default. The `#[serde(default)]` on
   `GameSystemFile.name` still deserializes to an empty string for v2 files, and the load handler
   checks: if name is empty, derive from filename.

2. **Board entity cleanup on Close Project**: DECIDED -- Despawn on `OnExit(AppScreen::Editor)`. All
   board entities (`HexTile`, `UnitInstance`) are despawned when exiting the editor state. This is
   the correct approach over a more targeted fix.

3. **Launcher name input behavior**: DECIDED -- Enter triggers Create. Pressing Enter in the name
   text field has the same effect as clicking the Create button, for a keyboard-friendly flow.

4. **`GameSystem.id` on new project**: DECIDED -- Yes, workspace name round-trips through save/load.
   `Workspace.name` is written to `GameSystemFile.name` on save and read back on load.
   `GameSystem.id` remains a UUID and is not affected.

5. **`Workspace.name` display in top bar**: DECIDED -- Show both workspace name and truncated ID.
   The workspace name is displayed prominently as the primary heading. The truncated `GameSystem.id`
   remains as secondary info below it (smaller, dimmed). This replaces the original proposal to
   remove the ID entirely. Keep "hexorder" branding + version.

6. **Camera and indicator entity cleanup**: DECIDED -- Despawn everything on exit.
   `OnExit(AppScreen::Editor)` despawns ALL editor-spawned entities: tiles, units, camera,
   indicators, and overlays. This is a clean-slate approach -- every `OnEnter(Editor)` system starts
   fresh with no leftover entities from a previous session. No idempotency guards needed.

7. **Cmd+N from editor**: DECIDED -- Go to launcher (option a). Cmd+N triggers `CloseProjectEvent`,
   transitioning to the launcher where the user names and creates a new project. Consistent and
   simple.

## Overview

This pitch addresses four workspace lifecycle gaps: no project naming, no default save location, no
return-to-launcher, and no workspace concept. The implementation touches three areas: the
persistence contract (`src/contracts/persistence.rs`), the persistence plugin (`src/persistence/`),
and the editor UI plugin (`src/editor_ui/`).

### Scope Summary

| Element                | Area                   | What changes                                       |
| ---------------------- | ---------------------- | -------------------------------------------------- |
| Workspace resource     | contracts              | New `Workspace` resource, retire `CurrentFilePath` |
| New Project dialog     | editor_ui              | Launcher gets name input + Create flow             |
| Default save location  | persistence            | Pre-fill `~/Documents/Hexorder/{name}.hexorder`    |
| Return to launcher     | editor_ui, persistence | File > Close Project menu item, cleanup systems    |
| Display workspace name | editor_ui              | Top bar shows `Workspace.name`                     |

## Contract Changes

### New: `Workspace` Resource

Lives in `src/contracts/persistence.rs`, replaces `CurrentFilePath`.

```rust
/// Tool-level session state for the currently open project.
/// Initialized on NewProjectEvent and LoadRequestEvent.
/// Reset on CloseProjectEvent / return-to-launcher.
#[derive(Resource, Debug, Clone)]
pub struct Workspace {
    /// Human-readable project name (display only, not an identifier).
    pub name: String,
    /// Path to the last-saved file. None if never saved.
    pub file_path: Option<PathBuf>,
    /// Whether the project has unsaved changes.
    /// Placeholder for future use -- not actively tracked in this pitch.
    pub dirty: bool,
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            name: String::new(),
            file_path: None,
            dirty: false,
        }
    }
}
```

A manual `Default` sets `name` to an empty string. There is no "Untitled" default -- new projects
always require the user to provide a name via the launcher. The empty default is only used for
resource initialization before a project is created or loaded.

### Retire: `CurrentFilePath`

`CurrentFilePath` is removed. All code referencing `CurrentFilePath` migrates to `Workspace`:

| Old                           | New                                |
| ----------------------------- | ---------------------------------- |
| `file_path.path`              | `workspace.file_path`              |
| `file_path.path = Some(path)` | `workspace.file_path = Some(path)` |
| `file_path.path = None`       | workspace reset to default         |

### Affected consumers

- `src/persistence/mod.rs` -- resource registration changes from `CurrentFilePath` to `Workspace`
- `src/persistence/systems.rs` -- `handle_save_request` (line 38), `handle_load_request` (line 113),
  `handle_new_project` (line 171) all reference `CurrentFilePath`
- `src/editor_ui/systems.rs` -- will display workspace name in top bar, launcher passes name to new
  project event

### New: `CloseProjectEvent`

```rust
/// Triggers closing the current project and returning to the launcher.
#[derive(Event, Debug)]
pub struct CloseProjectEvent;
```

### Modified: `NewProjectEvent`

Add a `name` field so the launcher can pass the user-entered name:

```rust
/// Triggers creation of a new empty project.
#[derive(Event, Debug)]
pub struct NewProjectEvent {
    /// Display name for the new workspace.
    pub name: String,
}
```

This is a breaking change to the event signature. All trigger sites must be updated. With the
recommended approach (Cmd+N from editor triggers `CloseProjectEvent` instead), only the launcher
"Create" button triggers `NewProjectEvent`.

### Modified: `GameSystemFile`

Add a `name` field for workspace name persistence:

```rust
pub struct GameSystemFile {
    pub format_version: u32,       // bumps from 2 to 3
    #[serde(default)]
    pub name: String,              // NEW -- workspace display name
    pub game_system: GameSystem,
    pub entity_types: EntityTypeRegistry,
    pub enums: EnumRegistry,
    pub structs: StructRegistry,
    pub concepts: ConceptRegistry,
    pub relations: RelationRegistry,
    pub constraints: ConstraintRegistry,
    pub map_radius: u32,
    pub tiles: Vec<TileSaveData>,
    pub units: Vec<UnitSaveData>,
}
```

`FORMAT_VERSION` increments from 2 to 3. The `#[serde(default)]` attribute ensures v2 files
deserialize correctly with `name` defaulting to an empty string. The load handler then checks: if
`name` is empty (v2 file), derive the workspace name from the `.hexorder` filename (strip the
extension, use the stem). For v3+ files, the `name` field is always populated.

### Contract spec update checklist

`docs/contracts/persistence.md` must be updated to:

- Add `Workspace` resource documentation
- Remove `CurrentFilePath` documentation
- Update `GameSystemFile` table with `name` field
- Add `CloseProjectEvent` documentation
- Update `NewProjectEvent` documentation with `name` field

## Plugin Changes

### persistence

#### `PersistencePlugin::build()`

In `src/persistence/mod.rs`:

```diff
- app.init_resource::<CurrentFilePath>();
+ app.init_resource::<Workspace>();
+ app.add_observer(systems::handle_close_project);
+ app.add_systems(OnExit(AppScreen::Editor), systems::cleanup_editor_entities);
```

#### `handle_save_request`

Changes to `src/persistence/systems.rs` (line 26):

1. Replace `ResMut<CurrentFilePath>` parameter with `ResMut<Workspace>`
2. Read `workspace.file_path` instead of `file_path.path`
3. When `file_path` is None (first save), pre-fill the save dialog:
    - Directory: `~/Documents/Hexorder/` (create with `std::fs::create_dir_all` if missing)
    - Filename: `{sanitized_workspace_name}.hexorder`
4. After successful save, set `workspace.file_path = Some(path)` and `workspace.dirty = false`
5. Write `workspace.name` into the `GameSystemFile.name` field

Current save dialog construction (line 45-48):

```rust
let dialog = rfd::FileDialog::new()
    .add_filter("Hexorder", &["hexorder"])
    .set_file_name("untitled.hexorder");
```

New construction:

```rust
let sanitized_name = sanitize_filename(&workspace.name);
let file_name = format!("{sanitized_name}.hexorder");

let mut dialog = rfd::FileDialog::new()
    .add_filter("Hexorder", &["hexorder"])
    .set_file_name(&file_name);

// Pre-fill default directory on first save.
if let Some(default_dir) = default_save_directory() {
    if std::fs::create_dir_all(&default_dir).is_ok() {
        dialog = dialog.set_directory(&default_dir);
    }
}
```

#### `handle_load_request`

Changes to `src/persistence/systems.rs` (line 103):

1. Replace `ResMut<CurrentFilePath>` with `ResMut<Workspace>`
2. After loading, derive the workspace name:
    - If `file.name` is non-empty (v3+ file): use `file.name`
    - If `file.name` is empty (v2 file): derive from the `.hexorder` filename by stripping the
      extension and using the file stem (e.g., `"My WW2 Game.hexorder"` becomes `"My WW2 Game"`)
3. Set `workspace.file_path = Some(path)`, `workspace.dirty = false`

#### `handle_new_project`

Changes to `src/persistence/systems.rs` (line 160):

1. Replace `ResMut<CurrentFilePath>` with `ResMut<Workspace>`
2. Read `name` from `NewProjectEvent.name`
3. Set `workspace.name = event.name`, `workspace.file_path = None`, `workspace.dirty = false`

The function currently transitions to `AppScreen::Editor`. This is correct -- the event is only
triggered from the launcher, which is in `AppScreen::Launcher`.

#### New: `handle_close_project`

Observer for `CloseProjectEvent`. Performs the same registry reset as `handle_new_project` but
transitions to `AppScreen::Launcher`:

```rust
pub fn handle_close_project(
    _trigger: On<CloseProjectEvent>,
    mut workspace: ResMut<Workspace>,
    mut game_system: ResMut<GameSystem>,
    mut entity_types: ResMut<EntityTypeRegistry>,
    mut enum_registry: ResMut<EnumRegistry>,
    mut struct_registry: ResMut<StructRegistry>,
    mut concepts: ResMut<ConceptRegistry>,
    mut relations: ResMut<RelationRegistry>,
    mut constraints: ResMut<ConstraintRegistry>,
    mut schema: ResMut<SchemaValidation>,
    mut selected_unit: ResMut<SelectedUnit>,
    mut next_state: ResMut<NextState<AppScreen>>,
) {
    *workspace = Workspace::default();

    // Reset to factory defaults.
    *game_system = crate::game_system::create_game_system();
    *entity_types = crate::game_system::create_entity_type_registry();
    *enum_registry = crate::game_system::create_enum_registry();
    *struct_registry = StructRegistry::default();
    *concepts = ConceptRegistry::default();
    *relations = RelationRegistry::default();
    *constraints = ConstraintRegistry::default();
    *schema = SchemaValidation::default();
    selected_unit.entity = None;

    next_state.set(AppScreen::Launcher);
}
```

Both `handle_close_project` and `handle_new_project` share the same registry reset logic. Extract a
shared helper function `reset_all_registries()` to avoid duplication:

```rust
fn reset_all_registries(
    game_system: &mut GameSystem,
    entity_types: &mut EntityTypeRegistry,
    enum_registry: &mut EnumRegistry,
    struct_registry: &mut StructRegistry,
    concepts: &mut ConceptRegistry,
    relations: &mut RelationRegistry,
    constraints: &mut ConstraintRegistry,
    schema: &mut SchemaValidation,
    selected_unit: &mut SelectedUnit,
) {
    *game_system = crate::game_system::create_game_system();
    *entity_types = crate::game_system::create_entity_type_registry();
    *enum_registry = crate::game_system::create_enum_registry();
    *struct_registry = StructRegistry::default();
    *concepts = ConceptRegistry::default();
    *relations = RelationRegistry::default();
    *constraints = ConstraintRegistry::default();
    *schema = SchemaValidation::default();
    selected_unit.entity = None;
}
```

Both observers call this helper, then handle their specific workspace updates and state transitions.

#### New: `cleanup_editor_entities`

System registered on `OnExit(AppScreen::Editor)`. Despawns ALL editor-spawned entities to ensure a
clean slate when re-entering the editor. This is a comprehensive cleanup -- every entity type that
is spawned on `OnEnter(AppScreen::Editor)` is despawned on exit:

```rust
pub fn cleanup_editor_entities(
    mut commands: Commands,
    tiles: Query<Entity, With<HexTile>>,
    units: Query<Entity, With<UnitInstance>>,
    cameras: Query<Entity, With<Camera3d>>,
    indicators: Query<Entity, With<HexIndicator>>,
    overlays: Query<Entity, With<MoveOverlay>>,
) {
    for entity in tiles.iter()
        .chain(units.iter())
        .chain(cameras.iter())
        .chain(indicators.iter())
        .chain(overlays.iter())
    {
        commands.entity(entity).despawn();
    }
}
```

**All editor-spawned entity types are covered:**

| Entity type      | Spawned by                     | Cleanup                                |
| ---------------- | ------------------------------ | -------------------------------------- |
| Hex tiles        | `hex_grid::spawn_grid`         | Despawned by `cleanup_editor_entities` |
| Unit instances   | unit placement / board load    | Despawned by `cleanup_editor_entities` |
| Camera           | `camera::spawn_camera`         | Despawned by `cleanup_editor_entities` |
| Move overlays    | `hex_grid::sync_move_overlays` | Despawned by `cleanup_editor_entities` |
| Hover indicator  | `hex_grid::setup_indicators`   | Despawned by `cleanup_editor_entities` |
| Select indicator | `hex_grid::setup_indicators`   | Despawned by `cleanup_editor_entities` |

Note: The exact marker component names (`HexIndicator`, `MoveOverlay`) may need adjustment based on
the actual component types used in the codebase. The important design decision is that ALL
editor-spawned entities are despawned -- no idempotency guards needed in the `OnEnter` systems.

#### Filename sanitization helper

```rust
/// Sanitize a workspace name for use as a filename.
/// Replaces disallowed characters with hyphens, trims, and falls back to "untitled".
fn sanitize_filename(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = sanitized.trim().to_string();
    if trimmed.is_empty() {
        "untitled".to_string()
    } else {
        trimmed
    }
}
```

#### Default save directory helper

```rust
/// Returns the default save directory for new projects: ~/Documents/Hexorder/
fn default_save_directory() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join("Documents").join("Hexorder"))
}
```

Uses `std::env::var("HOME")` rather than adding a `dirs` crate dependency. This is macOS-specific
but acceptable per the constitution (macOS-only platform).

### editor_ui

#### Launcher system

The current `launcher_system` (line 65-98 of `src/editor_ui/systems.rs`) triggers `NewProjectEvent`
immediately when "New Game System" is clicked. The new flow adds an inline name input.

**State fields added to `EditorState` (`src/editor_ui/components.rs`):**

```rust
/// Whether the new project name input is visible on the launcher.
pub launcher_name_input_visible: bool,
/// Text content of the new project name input.
pub launcher_project_name: String,
```

Defaults: `launcher_name_input_visible: false`, `launcher_project_name: String::new()`.

**Updated `launcher_system` signature:**

```rust
pub fn launcher_system(
    mut contexts: EguiContexts,
    mut editor_state: ResMut<EditorState>,
    mut commands: Commands,
) {
```

**UI flow (inside the existing `CentralPanel`):**

```
[hexorder]                    // RichText, size 32, strong
[v{CARGO_PKG_VERSION}]       // small, gray
(24px space)

if NOT editor_state.launcher_name_input_visible:
    [New Game System] (200x36)
        on click: set launcher_name_input_visible = true,
                  set launcher_project_name = "" (empty -- user must provide a name)
else:
    "Project Name:" label
    [text_edit_singleline(&mut editor_state.launcher_project_name)]
        hint_text: "e.g., My WW2 Campaign"
    horizontal:
        [Create] button (DISABLED when trimmed name is empty)
            on click: trigger NewProjectEvent { name: trimmed },
                      reset launcher state
        [Cancel] button
            on click: hide input, reset state

(8px space)
[Open...] (200x36, always visible)
```

**Enter key handling**: After the `text_edit_singleline` response, check:

```rust
let response = ui.text_edit_singleline(&mut editor_state.launcher_project_name);
let enter_pressed = response.lost_focus()
    && ui.input(|i| i.key_pressed(egui::Key::Enter));
```

If `enter_pressed` and name is non-empty after trimming, trigger Create. If the name is empty, Enter
does nothing (same as the Create button being disabled).

**Request focus**: When the name input first appears, request keyboard focus on the text field:

```rust
if just_became_visible {
    response.request_focus();
}
```

This requires tracking whether the input was just revealed (e.g., a `Local<bool>` or comparing
previous frame state).

#### Editor panel -- File menu

Add "Close Project" menu item. Change File > New to trigger `CloseProjectEvent`. In
`editor_panel_system` at line 127-149 of `src/editor_ui/systems.rs`:

```rust
ui.menu_button("File", |ui| {
    if ui.button("New          Cmd+N").clicked() {
        commands.trigger(CloseProjectEvent);  // Changed: was NewProjectEvent
        ui.close();
    }
    if ui.button("Open...      Cmd+O").clicked() {
        commands.trigger(LoadRequestEvent);
        ui.close();
    }
    ui.separator();
    if ui.button("Save         Cmd+S").clicked() {
        commands.trigger(SaveRequestEvent { save_as: false });
        ui.close();
    }
    if ui.button("Save As...   Cmd+Shift+S").clicked() {
        commands.trigger(SaveRequestEvent { save_as: true });
        ui.close();
    }
    ui.separator();
    if ui.button("Close Project").clicked() {
        commands.trigger(CloseProjectEvent);
        ui.close();
    }
});
```

Note: File > New now triggers `CloseProjectEvent` (goes to launcher) rather than `NewProjectEvent`
(which stayed in editor). The Cmd+N keyboard shortcut in `src/persistence/systems.rs` (line 219)
must also change to trigger `CloseProjectEvent`.

#### Editor panel -- Top bar / Game System Info

Replace `render_game_system_info` to show workspace name prominently with truncated ID as secondary
info. In `src/editor_ui/systems.rs` (line 280):

**Current:**

```
Hexorder                  v0.1.0
ID: a1b2c3d4...
---
```

**New:**

```
{workspace.name}          v0.1.0
hexorder | a1b2c3d4...
---
```

The workspace name becomes the primary heading. "hexorder" branding and the truncated
`GameSystem.id` are shown together as a small secondary line. Both pieces of information are
preserved -- the workspace name is prominent, the ID is demoted but still visible.

Implementation:

```rust
pub(crate) fn render_workspace_header(
    ui: &mut egui::Ui,
    workspace: &Workspace,
    gs: &GameSystem,
) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(&workspace.name).strong().size(15.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("v{}", gs.version))
                    .small()
                    .color(egui::Color32::GRAY),
            );
        });
    });
    let truncated_id = &gs.id[..8.min(gs.id.len())];
    ui.label(
        egui::RichText::new(format!("hexorder | {truncated_id}"))
            .small()
            .color(egui::Color32::from_gray(120)),
    );
    ui.separator();
}
```

The call site in `editor_panel_system` changes from:

```rust
render_game_system_info(ui, &game_system);
```

to:

```rust
render_workspace_header(ui, &workspace, &game_system);
```

The `editor_panel_system` signature gains `workspace: Res<Workspace>`.

### contracts

`src/contracts/persistence.rs` changes:

- Remove `CurrentFilePath` struct
- Add `Workspace` struct with manual `Default`
- Add `CloseProjectEvent` struct
- Add `name: String` field to `NewProjectEvent`
- Add `name: String` field to `GameSystemFile` with `#[serde(default)]`
- Bump `FORMAT_VERSION` from 2 to 3

`docs/contracts/persistence.md` changes:

- Remove `CurrentFilePath` section
- Add `Workspace` section
- Add `CloseProjectEvent` section
- Update `NewProjectEvent` with `name` field
- Update `GameSystemFile` table with `name` field and version bump note

## Save/Load Flow

### Save (first time)

```
User presses Cmd+S (or File > Save)
  -> SaveRequestEvent { save_as: false }
  -> handle_save_request checks workspace.file_path -> None
  -> Determine default directory: ~/Documents/Hexorder/
  -> Create directory if missing: std::fs::create_dir_all(dir)
  -> Sanitize workspace name for filename: "My WW2 Game" -> "My WW2 Game.hexorder"
  -> Open file dialog with:
      .set_directory(documents_hexorder_dir)
      .set_file_name("{sanitized_name}.hexorder")
      .add_filter("Hexorder", &["hexorder"])
  -> User confirms (or changes path)
  -> Build GameSystemFile with name: workspace.name
  -> Write to disk
  -> Set workspace.file_path = Some(chosen_path)
```

### Save (subsequent)

```
User presses Cmd+S
  -> SaveRequestEvent { save_as: false }
  -> handle_save_request checks workspace.file_path -> Some(path)
  -> Build GameSystemFile with name: workspace.name
  -> Write directly to path (no dialog)
```

### Save As

```
User presses Cmd+Shift+S (or File > Save As...)
  -> SaveRequestEvent { save_as: true }
  -> handle_save_request always shows dialog
  -> Pre-fill with current workspace.file_path directory and filename
      (or default directory if no file_path)
  -> After save, update workspace.file_path to new path
```

### Load

```
User clicks Open... (launcher or File > Open)
  -> LoadRequestEvent
  -> File dialog opens
  -> User picks .hexorder file
  -> load_from_file() reads and deserializes
  -> If format_version <= 2: name field is empty (serde default)
  -> Derive workspace name:
     - If file.name is non-empty (v3+): use file.name
     - If file.name is empty (v2): derive from filename stem
       (e.g., "My WW2 Game.hexorder" -> "My WW2 Game")
  -> Overwrite all registries (existing behavior)
  -> Set workspace.name = derived name
  -> Set workspace.file_path = Some(chosen_path)
  -> Set workspace.dirty = false
  -> Transition to AppScreen::Editor
```

### Default save directory

```rust
fn default_save_directory() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join("Documents").join("Hexorder"))
}
```

If `HOME` is unset (highly unlikely on macOS), falls back to no pre-filled directory (same as
current behavior). No new crate dependency needed.

## Launcher UI

### Current state

The launcher (`src/editor_ui/systems.rs` line 65-98) renders a centered panel with:

- "hexorder" heading (RichText, size 32, strong)
- Version label (small, gray)
- 24px space
- "New Game System" button (200x36) -- triggers `NewProjectEvent` immediately
- 8px space
- "Open..." button (200x36) -- triggers `LoadRequestEvent`

### New state

The launcher gains an inline name input flow. UI state is tracked in `EditorState` (which is already
a resource in the editor UI plugin).

```
[hexorder]                    // size 32, strong
[v0.9.0]                      // small, gray
(24px space)

--- if name input NOT visible: ---
[New Game System] (200x36)    // click reveals name input
--- else: ---
"Project Name:" label
[________________]            // text_edit_singleline, empty (user must type a name)
[Create] [Cancel]             // Create disabled until name is non-empty
--- end if ---

(8px space)
[Open...] (200x36)            // always visible
```

The name input flow is entirely within the existing `CentralPanel` and `vertical_centered` layout.
No new panels, no modals, no separate windows.

### EditorState additions

Two new fields in `EditorState` (`src/editor_ui/components.rs`):

```rust
pub launcher_name_input_visible: bool,   // default: false
pub launcher_project_name: String,       // default: String::new()
```

These are reset when the launcher transitions to the editor (in the Create click handler) and when
the Close Project flow returns to the launcher (the `EditorState` is not reset, but the
`launcher_name_input_visible` flag defaults to false).

## Close Project Flow

### Full sequence

```
1. User clicks File > Close Project (or Cmd+N from editor)
2. commands.trigger(CloseProjectEvent)
3. handle_close_project observer fires:
   a. *workspace = Workspace::default()
   b. Reset all registries via reset_all_registries() helper
   c. next_state.set(AppScreen::Launcher)
4. State transition fires:
   a. OnExit(AppScreen::Editor) -> cleanup_editor_entities runs:
      - Despawn all entities With<HexTile>
      - Despawn all entities With<UnitInstance>
      - Despawn all entities With<Camera3d>
      - Despawn all entities With<HexIndicator> (hover + select indicators)
      - Despawn all entities With<MoveOverlay>
   b. AppScreen becomes Launcher
   c. Launcher screen renders (name input hidden by default)
5. When user creates or opens a new project:
   a. OnEnter(AppScreen::Editor) fires
   b. hex_grid: setup_grid_config, setup_materials, spawn_grid (fresh grid)
   c. camera: spawn_camera, configure_bounds (fresh camera)
   d. cell: setup_cell_materials
   e. unit: setup_unit_visuals
   f. scripting: init_lua
```

### Cmd+N from editor (changed behavior)

**Current**: Cmd+N triggers `NewProjectEvent` which resets registries and stays in
`AppScreen::Editor`. The `OnEnter`/`OnExit` systems do NOT fire because the state does not change.
Existing board entities persist (latent bug -- stale tiles and units remain).

**New**: Cmd+N triggers `CloseProjectEvent` which transitions to `AppScreen::Launcher`. The user
then uses the launcher to name and create a new project. This is correct and consistent: the
launcher is always the entry point for new projects.

The keyboard shortcut handler in `src/persistence/systems.rs` (line 219) changes:

```diff
  } else if input.just_pressed(KeyCode::KeyN) {
-     commands.trigger(NewProjectEvent);
+     commands.trigger(CloseProjectEvent);
  }
```

### Registry reset deduplication

Both `handle_close_project` and `handle_new_project` perform the same registry reset. The extracted
`reset_all_registries` helper (described in the persistence plugin changes section) avoids this
duplication. Both observers call the helper, then handle their specific workspace updates and state
transitions:

- `handle_new_project`: sets `workspace.name = event.name`, transitions to `AppScreen::Editor`
- `handle_close_project`: sets `*workspace = Workspace::default()`, transitions to
  `AppScreen::Launcher`

### Entity cleanup details

All editor-spawned entities are despawned on `OnExit(AppScreen::Editor)` by
`cleanup_editor_entities`. Clean-slate approach -- no idempotency guards needed.

| Entity type      | Spawned by                     | Cleanup approach                       |
| ---------------- | ------------------------------ | -------------------------------------- |
| `HexTile`        | `hex_grid::spawn_grid`         | Despawned by `cleanup_editor_entities` |
| `UnitInstance`   | unit placement / board load    | Despawned by `cleanup_editor_entities` |
| Camera           | `camera::spawn_camera`         | Despawned by `cleanup_editor_entities` |
| Move overlays    | `hex_grid::sync_move_overlays` | Despawned by `cleanup_editor_entities` |
| Hover indicator  | `hex_grid::setup_indicators`   | Despawned by `cleanup_editor_entities` |
| Select indicator | `hex_grid::setup_indicators`   | Despawned by `cleanup_editor_entities` |

## First Piece

**Build the Workspace resource and Close Project flow first.**

Rationale:

- The Workspace resource is the foundation that everything else depends on
- Close Project exercises the full state transition (Editor -> Launcher -> Editor) which is the most
  architecturally novel piece
- It surfaces the board entity cleanup concern early (the biggest risk in this pitch)
- Once Close Project works, the other elements (name input, default save path, display name) are
  straightforward UI additions

### Build order

1. **Workspace resource + CurrentFilePath migration** (contract change)
    - Add `Workspace` to `src/contracts/persistence.rs`
    - Remove `CurrentFilePath`
    - Update `docs/contracts/persistence.md`
    - Update all references in persistence and editor_ui plugins
    - Run `cargo build` to verify compilation

2. **CloseProjectEvent + cleanup_editor_entities** (persistence plugin)
    - Add `CloseProjectEvent` to contracts
    - Implement `handle_close_project` observer
    - Implement `cleanup_editor_entities` on `OnExit(Editor)` -- despawns ALL editor entities
      (tiles, units, camera, indicators, overlays)
    - Add "Close Project" to File menu in editor_ui
    - Change Cmd+N to trigger `CloseProjectEvent`
    - Test: create project, close, verify return to launcher, create new, verify clean state

3. **Launcher name input** (editor_ui)
    - Add state fields to `EditorState`
    - Modify `launcher_system` for inline name input flow
    - Add `name` field to `NewProjectEvent`
    - Update `handle_new_project` to use name from event
    - Validate: Create button disabled when name input is empty
    - Test: launch app, click New, enter name, verify Workspace.name is set

4. **Default save location** (persistence)
    - Add `sanitize_filename` helper
    - Add `default_save_directory` helper
    - Modify `handle_save_request` to pre-fill dialog
    - Test: create named project, Cmd+S, verify dialog pre-fills correctly

5. **Display workspace name in editor** (editor_ui)
    - Rename `render_game_system_info` to `render_workspace_header`
    - Show `Workspace.name` prominently as primary heading
    - Show truncated `GameSystem.id` as secondary info alongside "hexorder" branding
    - Add `Res<Workspace>` to `editor_panel_system` parameters
    - Test: create named project, verify name in top bar with ID visible below

6. **Persist workspace name in file** (persistence + contract)
    - Add `name` field to `GameSystemFile` with `#[serde(default)]`
    - Bump `FORMAT_VERSION` to 3
    - Update save to write name, load to read name
    - Update existing tests for new field
    - Test: save, load, verify name round-trips
    - Test: load v2 file, verify name derived from filename

## Risk Assessment

### Low risk

- **Workspace resource**: Straightforward data struct replacing `CurrentFilePath`
- **Launcher name input**: Standard egui text field + button patterns
- **Default save directory**: `std::env::var("HOME")` + `create_dir_all`
- **Display name in top bar**: Minor UI change
- **File format name field**: `#[serde(default)]` handles v2 migration

### Medium risk

- **Editor entity cleanup**: Most architecturally significant change. The clean-slate approach
  (despawn ALL editor entities on exit) is conceptually simple but requires identifying every entity
  type and its marker component. Need to trace all `OnEnter(AppScreen::Editor)` systems and ensure
  every spawned entity type has a corresponding query in `cleanup_editor_entities`. The `OnEnter`
  systems that insert resources (`setup_grid_config`, `setup_materials`) use
  `commands.insert_resource` which overwrites existing resources -- safe. The `spawn_grid`,
  `spawn_camera`, and `setup_indicators` systems unconditionally spawn entities -- all covered by
  the cleanup system.

- **Cmd+N behavior change**: Goes from instant reset (stay in editor) to two-step (launcher + name
  input). Correct but slower. Acceptable tradeoff for this pitch. A future pitch could add an inline
  rename/reset flow in the editor if speed matters.

- **`OnEnter(Editor)` re-entry safety**: Systems that insert resources overwrite safely. Systems
  that spawn entities need old entities cleaned up first. The `cleanup_editor_entities` system on
  `OnExit(Editor)` handles all entity types (tiles, units, camera, indicators, overlays), ensuring a
  clean slate for re-entry.

### Out of scope (confirmed No Gos from pitch)

- Recent files list on launcher
- Unsaved changes confirmation dialog
- Project templates
- Workspace state persistence (camera position, panels) -- separate pitch #10
- Auto-save
- Multi-project / tabbed interface
- Cross-platform save location conventions

### Dependencies

- No new crate dependency (use `std::env::var("HOME")` for save directory)
- No changes to `hex_grid`, `camera`, `cell`, `unit`, `ontology`, `rules_engine`, or `scripting`
  plugin source code -- the cleanup system queries marker components but lives in the persistence
  plugin
- Only the persistence and editor_ui plugins change, plus the shared persistence contract

### Test plan

| Test                                                 | Type        | Verifies                          |
| ---------------------------------------------------- | ----------- | --------------------------------- |
| Workspace default values                             | unit        | Resource initializes correctly    |
| CurrentFilePath removal compiles                     | build       | Migration complete                |
| CloseProjectEvent resets workspace + registries      | unit        | Registry reset + state transition |
| cleanup_editor_entities despawns all editor entities | unit        | Entity cleanup on editor exit     |
| NewProjectEvent carries name to Workspace            | unit        | Name flow from launcher           |
| sanitize_filename handles edge cases                 | unit        | Empty, special chars, whitespace  |
| default_save_directory returns expected path         | unit        | macOS Documents path              |
| GameSystemFile v3 round-trip with name               | unit        | Persistence with name field       |
| GameSystemFile v2 load derives name from file        | unit        | Backward compat + filename deriv  |
| Editor -> Launcher -> Editor cycle                   | integration | Full state transition round-trip  |
| Save pre-fills default directory and filename        | manual      | File dialog behavior              |
| Close Project -> New -> verify clean grid            | manual/UAT  | Full lifecycle flow               |
| Workspace name displays in top bar                   | manual/UAT  | UI display correctness            |
| Name input on launcher with Enter key                | manual/UAT  | Keyboard-friendly creation        |
