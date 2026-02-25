# Async Dialog Migration Plan (Scope 2)

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan
> task-by-task.

**Goal:** Migrate all blocking `rfd::FileDialog` and `rfd::MessageDialog` calls in
`src/persistence/systems.rs` to the async infrastructure from Scope 1.

**Architecture:** Each observer becomes a thin dispatcher that queues an exclusive-world closure via
`commands.queue()`. Pure helpers (`save_to_path`, `load_from_path`, etc.) take `&mut World` for
flexible resource access. Dialog chaining uses `then: Option<PendingAction>` on
`DialogKind::SaveFile` to chain confirm → save → action without nested blocking.

**Tech Stack:** Bevy 0.18 observers, `rfd::AsyncFileDialog`/`AsyncMessageDialog` (via Scope 1 spawn
helpers), `commands.queue()` for exclusive world access.

---

### Task 1: Type Changes — Add Clone and `then` Field

**Files:**

- Modify: `src/persistence/async_dialog.rs`

**Step 1: Add `Clone` derive to `DialogKind` and `DialogResult`**

Change the `DialogKind` derive (line 24):

```rust
#[derive(Debug, Clone)]
pub(crate) enum DialogKind {
```

Change the `DialogResult` derive (line 36):

```rust
#[derive(Debug, Clone)]
pub(crate) enum DialogResult {
```

**Step 2: Add `then: Option<PendingAction>` to `SaveFile` variant**

Replace the `SaveFile` variant (line 26):

```rust
    /// File save picker (save or save-as), with optional continuation action.
    SaveFile {
        save_as: bool,
        then: Option<PendingAction>,
    },
```

**Step 3: Update test reference**

In the `poll_completes_ready_task` test (line 218), update the `SaveFile` construction:

```rust
            kind: DialogKind::SaveFile {
                save_as: false,
                then: None,
            },
```

**Step 4: Run tests**

Run: `cargo test persistence::async_dialog`

Expected: All 5 async_dialog tests pass.

**Step 5: Commit**

```bash
git add src/persistence/async_dialog.rs
git commit -m "refactor(persistence): add Clone derives and then field to dialog types"
```

---

### Task 2: Extract `save_to_path()` and `build_game_system_file()` Helpers

**Files:**

- Modify: `src/persistence/systems.rs`

**Step 1: Write the `build_game_system_file` helper**

Add this function in the Shared Helpers section of `systems.rs`, after the existing
`sanitize_filename` function:

```rust
/// Build a `GameSystemFile` from current world state and pre-collected board data.
fn build_game_system_file(
    world: &World,
    tiles: &[(HexPosition, EntityData)],
    units: &[(HexPosition, EntityData)],
) -> GameSystemFile {
    let workspace = world.resource::<Workspace>();
    let game_system = world.resource::<GameSystem>();
    let entity_types = world.resource::<EntityTypeRegistry>();
    let enum_registry = world.resource::<EnumRegistry>();
    let struct_registry = world.resource::<StructRegistry>();
    let concepts = world.resource::<ConceptRegistry>();
    let relations = world.resource::<RelationRegistry>();
    let constraints = world.resource::<ConstraintRegistry>();
    let turn_structure = world.resource::<TurnStructure>();
    let crt = world.resource::<CombatResultsTable>();
    let combat_modifiers = world.resource::<CombatModifierRegistry>();
    let config = world.resource::<HexGridConfig>();

    let tile_data: Vec<TileSaveData> = tiles
        .iter()
        .map(|(pos, data)| TileSaveData {
            position: *pos,
            entity_type_id: data.entity_type_id,
            properties: data.properties.clone(),
        })
        .collect();

    let unit_data: Vec<UnitSaveData> = units
        .iter()
        .map(|(pos, data)| UnitSaveData {
            position: *pos,
            entity_type_id: data.entity_type_id,
            properties: data.properties.clone(),
        })
        .collect();

    GameSystemFile {
        format_version: FORMAT_VERSION,
        name: workspace.name.clone(),
        game_system: game_system.clone(),
        entity_types: entity_types.clone(),
        enums: enum_registry.clone(),
        structs: struct_registry.clone(),
        concepts: concepts.clone(),
        relations: relations.clone(),
        constraints: constraints.clone(),
        turn_structure: turn_structure.clone(),
        combat_results_table: crt.clone(),
        combat_modifiers: combat_modifiers.clone(),
        map_radius: config.map_radius,
        tiles: tile_data,
        units: unit_data,
        workspace_preset: workspace.workspace_preset.clone(),
        font_size_base: workspace.font_size_base,
    }
}
```

**Step 2: Write the `save_to_path` helper**

Add this function after `build_game_system_file`:

```rust
/// Save the current project to the given path. Returns `true` on success.
/// Updates workspace path and dirty flag on success.
/// No dialog logic — pure file I/O and state update.
fn save_to_path(path: &std::path::Path, world: &mut World) -> bool {
    // Collect board data via queries (releases world borrow after each block).
    let tiles: Vec<(HexPosition, EntityData)> = {
        let mut q = world.query_filtered::<(&HexPosition, &EntityData), With<HexTile>>();
        q.iter(world).map(|(p, d)| (*p, d.clone())).collect()
    };
    let units: Vec<(HexPosition, EntityData)> = {
        let mut q =
            world.query_filtered::<(&HexPosition, &EntityData), With<UnitInstance>>();
        q.iter(world).map(|(p, d)| (*p, d.clone())).collect()
    };

    let file = build_game_system_file(world, &tiles, &units);

    // Write to disk — scope the storage borrow.
    let write_result = {
        let storage = world.resource::<Storage>();
        storage.provider().save_at(path, &file)
    };

    match write_result {
        Ok(()) => {
            info!("Saved to {}", path.display());
            let mut workspace = world.resource_mut::<Workspace>();
            workspace.file_path = Some(path.to_path_buf());
            workspace.dirty = false;
            drop(workspace);

            world.trigger(ToastEvent {
                message: "Project saved".to_string(),
                kind: ToastKind::Success,
            });
            true
        }
        Err(e) => {
            error!("Failed to save: {e}");
            world.trigger(ToastEvent {
                message: format!("Save failed: {e}"),
                kind: ToastKind::Error,
            });
            false
        }
    }
}
```

**Step 3: Run tests**

Run: `cargo test persistence`

Expected: All existing persistence tests pass (the new helpers are not called by anything yet).

**Step 4: Commit**

```bash
git add src/persistence/systems.rs
git commit -m "refactor(persistence): extract save_to_path and build_game_system_file helpers"
```

---

### Task 3: Extract `load_from_path()` Helper

**Files:**

- Modify: `src/persistence/systems.rs`

**Step 1: Write the `load_from_path` helper**

Add this function after `save_to_path`:

```rust
/// Load a project from the given path. Returns `true` on success.
/// Overwrites all registries, updates workspace, inserts `PendingBoardLoad`,
/// and transitions to Editor state. No dialog logic.
fn load_from_path(path: &std::path::Path, world: &mut World) -> bool {
    // Read file from disk — scope the storage borrow.
    let file = {
        let storage = world.resource::<Storage>();
        storage.provider().load(path)
    };

    let file = match file {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to load: {e}");
            world.trigger(ToastEvent {
                message: format!("Load failed: {e}"),
                kind: ToastKind::Error,
            });
            return false;
        }
    };

    // Overwrite registries.
    *world.resource_mut::<GameSystem>() = file.game_system;
    *world.resource_mut::<EntityTypeRegistry>() = file.entity_types;
    *world.resource_mut::<EnumRegistry>() = file.enums;
    *world.resource_mut::<StructRegistry>() = file.structs;
    *world.resource_mut::<ConceptRegistry>() = file.concepts;
    *world.resource_mut::<RelationRegistry>() = file.relations;
    *world.resource_mut::<ConstraintRegistry>() = file.constraints;
    *world.resource_mut::<TurnStructure>() = file.turn_structure;
    *world.resource_mut::<CombatResultsTable>() = file.combat_results_table;
    *world.resource_mut::<CombatModifierRegistry>() = file.combat_modifiers;
    *world.resource_mut::<SchemaValidation>() = SchemaValidation::default();

    // Derive workspace name: use file name field if present (v3+),
    // otherwise derive from filename stem (v2 backward compat).
    let name = if file.name.is_empty() {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string()
    } else {
        file.name
    };

    {
        let mut workspace = world.resource_mut::<Workspace>();
        workspace.name = name;
        workspace.file_path = Some(path.to_path_buf());
        workspace.dirty = false;
        workspace.workspace_preset = file.workspace_preset;
        workspace.font_size_base = file.font_size_base;
    }

    // Insert pending board load for deferred application.
    world.insert_resource(PendingBoardLoad {
        tiles: file.tiles,
        units: file.units,
    });

    // Transition to editor (may already be in editor if loading from editor).
    world.resource_mut::<NextState<AppScreen>>().set(AppScreen::Editor);

    world.trigger(ToastEvent {
        message: "Project loaded".to_string(),
        kind: ToastKind::Success,
    });

    let gs_id = world.resource::<GameSystem>().id.clone();
    info!("Loaded game system: {gs_id}");

    true
}
```

**Step 2: Run tests**

Run: `cargo test persistence`

Expected: All existing persistence tests pass.

**Step 3: Commit**

```bash
git add src/persistence/systems.rs
git commit -m "refactor(persistence): extract load_from_path helper"
```

---

### Task 4: Extract Reset and Close Helpers

**Files:**

- Modify: `src/persistence/systems.rs`

**Step 1: Write `reset_all_registries_world` helper**

Add this function after `load_from_path`, replacing the world-access pattern used by both new
project and close:

```rust
/// Reset all registries and derived state to factory defaults using world access.
fn reset_all_registries_world(world: &mut World) {
    *world.resource_mut::<GameSystem>() = crate::game_system::create_game_system();
    *world.resource_mut::<EntityTypeRegistry>() =
        crate::game_system::create_entity_type_registry();
    *world.resource_mut::<EnumRegistry>() = crate::game_system::create_enum_registry();
    *world.resource_mut::<StructRegistry>() = StructRegistry::default();
    *world.resource_mut::<ConceptRegistry>() = ConceptRegistry::default();
    *world.resource_mut::<RelationRegistry>() = RelationRegistry::default();
    *world.resource_mut::<ConstraintRegistry>() = ConstraintRegistry::default();
    *world.resource_mut::<SchemaValidation>() = SchemaValidation::default();
    world.resource_mut::<SelectedUnit>().entity = None;
}
```

**Step 2: Write `reset_to_new_project` helper**

```rust
/// Reset all state and initialize a new project with the given name.
fn reset_to_new_project(name: &str, world: &mut World) {
    reset_all_registries_world(world);

    // Reset mechanics to factory defaults.
    *world.resource_mut::<TurnStructure>() =
        crate::game_system::create_default_turn_structure();
    *world.resource_mut::<CombatResultsTable>() = crate::game_system::create_default_crt();
    *world.resource_mut::<CombatModifierRegistry>() = CombatModifierRegistry::default();
    *world.resource_mut::<TurnState>() = TurnState::default();
    *world.resource_mut::<ActiveCombat>() = ActiveCombat::default();

    {
        let mut workspace = world.resource_mut::<Workspace>();
        workspace.name = name.to_string();
        workspace.file_path = None;
        workspace.dirty = false;
        workspace.workspace_preset = String::new();
        workspace.font_size_base = 15.0;
    }

    world.resource_mut::<NextState<AppScreen>>().set(AppScreen::Editor);
}
```

**Step 3: Write `close_project` helper**

```rust
/// Reset all state and return to the launcher screen.
fn close_project(world: &mut World) {
    *world.resource_mut::<Workspace>() = Workspace::default();
    reset_all_registries_world(world);
    world.resource_mut::<NextState<AppScreen>>().set(AppScreen::Launcher);
}
```

**Step 4: Write `spawn_save_dialog_for_current_project` helper**

This shared helper is used by both `handle_save_request` (save-as path) and `dispatch_dialog_result`
(confirm → yes → no path). It reads workspace/storage to configure the dialog, then spawns an
`AsyncDialogTask`.

```rust
/// Spawn an async save dialog configured for the current project.
/// `then` specifies what to do after the save completes (if anything).
fn spawn_save_dialog_for_current_project(
    world: &mut World,
    then: Option<PendingAction>,
) {
    let (file_name, dir_from_workspace) = {
        let workspace = world.resource::<Workspace>();
        let sanitized = sanitize_filename(&workspace.name);
        let file_name = format!("{sanitized}.hexorder");
        let dir = workspace
            .file_path
            .as_ref()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf());
        (file_name, dir)
    };

    let initial_dir = dir_from_workspace.or_else(|| {
        let storage = world.resource::<Storage>();
        let base = storage.provider().base_dir().to_path_buf();
        std::fs::create_dir_all(&base).ok()?;
        Some(base)
    });

    let task = super::async_dialog::spawn_save_dialog(
        initial_dir.as_deref(),
        &file_name,
    );
    world.insert_resource(AsyncDialogTask {
        kind: DialogKind::SaveFile {
            save_as: true,
            then,
        },
        task,
    });
}
```

**Step 5: Add the async_dialog import block**

Add this import at the top of `systems.rs`, after the existing contract imports:

```rust
use crate::persistence::async_dialog::{
    AsyncDialogTask, ConfirmChoice, DialogCompleted, DialogKind, DialogResult, PendingAction,
    spawn_confirm_dialog, spawn_open_dialog,
};
```

**Step 6: Run tests**

Run: `cargo test persistence`

Expected: All existing persistence tests pass. The new helpers are not wired in yet.

**Step 7: Commit**

```bash
git add src/persistence/systems.rs
git commit -m "refactor(persistence): extract reset, close, and save-dialog helpers"
```

---

### Task 5: Add `handle_dialog_completed` Observer and Dispatch

**Files:**

- Modify: `src/persistence/systems.rs`
- Modify: `src/persistence/mod.rs`

**Step 1: Write `execute_pending_action`**

Add this function in the Shared Helpers section:

```rust
/// Dispatch a pending action after a dialog chain resolves.
fn execute_pending_action(action: PendingAction, world: &mut World) {
    match action {
        PendingAction::Load => {
            // Spawn async open-file dialog.
            let task = spawn_open_dialog();
            world.insert_resource(AsyncDialogTask {
                kind: DialogKind::OpenFile,
                task,
            });
        }
        PendingAction::NewProject { name } => {
            reset_to_new_project(&name, world);
        }
        PendingAction::CloseProject => {
            close_project(world);
        }
    }
}
```

**Step 2: Write `dispatch_dialog_result`**

```rust
/// Central router for dialog completion results. Handles all dialog kind + result
/// combinations including dialog chaining (confirm → save → action).
fn dispatch_dialog_result(
    kind: DialogKind,
    result: DialogResult,
    world: &mut World,
) {
    match (kind, result) {
        // --- Confirm Unsaved Changes ---
        (
            DialogKind::ConfirmUnsavedChanges { then },
            DialogResult::Confirmed(choice),
        ) => match choice {
            ConfirmChoice::Yes => {
                // Save first, then execute the pending action.
                let maybe_path =
                    world.resource::<Workspace>().file_path.clone();
                if let Some(path) = maybe_path {
                    if save_to_path(&path, world) {
                        execute_pending_action(then, world);
                    }
                    // Save failed → abort chain.
                } else {
                    // No existing path — spawn save-as dialog with chained action.
                    spawn_save_dialog_for_current_project(world, Some(then));
                }
            }
            ConfirmChoice::No => {
                // Skip save, execute the pending action directly.
                execute_pending_action(then, world);
            }
            ConfirmChoice::Cancel => {
                // User cancelled — do nothing.
            }
        },

        // --- Save File ---
        (
            DialogKind::SaveFile { then, .. },
            DialogResult::FilePicked(Some(path)),
        ) => {
            if save_to_path(&path, world) {
                if let Some(action) = then {
                    execute_pending_action(action, world);
                }
            }
        }
        (DialogKind::SaveFile { .. }, DialogResult::FilePicked(None)) => {
            // User cancelled save dialog — abort chain.
        }

        // --- Open File ---
        (DialogKind::OpenFile, DialogResult::FilePicked(Some(path))) => {
            load_from_path(&path, world);
        }
        (DialogKind::OpenFile, DialogResult::FilePicked(None)) => {
            // User cancelled — do nothing.
        }

        // --- Unhandled combinations ---
        (kind, result) => {
            warn!("Unhandled dialog completion: {kind:?} + {result:?}");
        }
    }
}
```

**Step 3: Write `handle_dialog_completed` observer**

```rust
/// Observer for `DialogCompleted` events. Clones event data and queues an
/// exclusive-world command for dispatch (observers can't take `&mut World`
/// alongside `On<E>`).
pub(crate) fn handle_dialog_completed(
    trigger: On<DialogCompleted>,
    mut commands: Commands,
) {
    let kind = trigger.event().kind.clone();
    let result = trigger.event().result.clone();

    commands.queue(move |world: &mut World| {
        dispatch_dialog_result(kind, result, world);
    });
}
```

**Step 4: Register the observer in mod.rs**

In `src/persistence/mod.rs`, add the observer registration after the existing observers:

```rust
        app.add_observer(systems::handle_dialog_completed);
```

**Step 5: Run tests**

Run: `cargo test persistence`

Expected: All existing tests pass. The handler is registered but won't fire until dialogs are
spawned asynchronously.

**Step 6: Commit**

```bash
git add src/persistence/systems.rs src/persistence/mod.rs
git commit -m "feat(persistence): add handle_dialog_completed observer and dispatch logic"
```

---

### Task 6: Convert Trigger Observers to Async

**Files:**

- Modify: `src/persistence/systems.rs`

**Step 1: Replace `handle_save_request`**

Replace the entire `handle_save_request` function with:

```rust
/// Handles save requests. If the workspace has a path and this is not save-as,
/// saves directly. Otherwise spawns an async save dialog.
pub fn handle_save_request(
    trigger: On<SaveRequestEvent>,
    mut commands: Commands,
) {
    let save_as = trigger.event().save_as;
    commands.queue(move |world: &mut World| {
        if world.contains_resource::<AsyncDialogTask>() {
            return; // Dialog already open.
        }

        let maybe_path = if save_as {
            None
        } else {
            world.resource::<Workspace>().file_path.clone()
        };

        if let Some(path) = maybe_path {
            save_to_path(&path, world);
        } else {
            spawn_save_dialog_for_current_project(world, None);
        }
    });
}
```

**Step 2: Replace `handle_load_request`**

Replace the entire `handle_load_request` function with:

```rust
/// Handles load requests. If the workspace is dirty, spawns a confirm dialog
/// first. Otherwise spawns an async open-file dialog directly.
pub fn handle_load_request(
    _trigger: On<LoadRequestEvent>,
    mut commands: Commands,
) {
    commands.queue(move |world: &mut World| {
        if world.contains_resource::<AsyncDialogTask>() {
            return;
        }

        let dirty = world.resource::<Workspace>().dirty;
        if dirty {
            let task = spawn_confirm_dialog();
            world.insert_resource(AsyncDialogTask {
                kind: DialogKind::ConfirmUnsavedChanges {
                    then: PendingAction::Load,
                },
                task,
            });
        } else {
            let task = spawn_open_dialog();
            world.insert_resource(AsyncDialogTask {
                kind: DialogKind::OpenFile,
                task,
            });
        }
    });
}
```

**Step 3: Replace `handle_new_project`**

Replace the entire `handle_new_project` function with:

```rust
/// Handles new project requests. If the workspace is dirty, spawns a confirm
/// dialog first. Otherwise resets to a new project directly.
pub fn handle_new_project(
    trigger: On<NewProjectEvent>,
    mut commands: Commands,
) {
    let name = trigger.event().name.clone();
    commands.queue(move |world: &mut World| {
        if world.contains_resource::<AsyncDialogTask>() {
            return;
        }

        let dirty = world.resource::<Workspace>().dirty;
        if dirty {
            let task = spawn_confirm_dialog();
            world.insert_resource(AsyncDialogTask {
                kind: DialogKind::ConfirmUnsavedChanges {
                    then: PendingAction::NewProject { name },
                },
                task,
            });
        } else {
            reset_to_new_project(&name, world);
        }
    });
}
```

**Step 4: Replace `handle_close_project`**

Replace the entire `handle_close_project` function with:

```rust
/// Handles close project requests. If the workspace is dirty, spawns a confirm
/// dialog first. Otherwise closes the project directly.
pub fn handle_close_project(
    _trigger: On<CloseProjectEvent>,
    mut commands: Commands,
) {
    commands.queue(move |world: &mut World| {
        if world.contains_resource::<AsyncDialogTask>() {
            return;
        }

        let dirty = world.resource::<Workspace>().dirty;
        if dirty {
            let task = spawn_confirm_dialog();
            world.insert_resource(AsyncDialogTask {
                kind: DialogKind::ConfirmUnsavedChanges {
                    then: PendingAction::CloseProject,
                },
                task,
            });
        } else {
            close_project(world);
        }
    });
}
```

**Step 5: Run tests**

Run: `cargo test persistence`

Expected: Existing tests pass. The `check_unsaved_changes_proceeds_when_clean` test still passes
because the old function exists (removed in next task).

**Step 6: Commit**

```bash
git add src/persistence/systems.rs
git commit -m "feat(persistence): convert all trigger observers to async dialog pattern"
```

---

### Task 7: Remove Dead Code

**Files:**

- Modify: `src/persistence/systems.rs`
- Modify: `src/persistence/tests.rs`

**Step 1: Remove dead functions and types from systems.rs**

Remove these items (they are no longer called by any live code):

- `clear_keyboard_after_dialog` function (lines 40–52)
- `reset_all_registries` function (lines 56–77) — replaced by `reset_all_registries_world`
- `ConfirmAction` enum (lines 100–109)
- `check_unsaved_changes` function (lines 117–134)
- `do_save` function (lines 140–248)

**Step 2: Remove unused imports from systems.rs**

Remove `use bevy::input::keyboard::KeyCode;` (line 5) — was only used by
`clear_keyboard_after_dialog`.

Also remove unused items from `hexorder_contracts::game_system` import if `SelectedUnit` is no
longer needed by the remaining code. Check after removal.

**Step 3: Remove dead test from tests.rs**

Remove the `check_unsaved_changes_proceeds_when_clean` test (lines 335–345) since the function it
tests no longer exists.

**Step 4: Run tests**

Run: `cargo test persistence`

Expected: All remaining tests pass.

**Step 5: Run clippy to verify no unused code warnings**

Run: `cargo clippy --all-targets`

Expected: No warnings. Fix any unused import warnings that clippy flags.

**Step 6: Commit**

```bash
git add src/persistence/systems.rs src/persistence/tests.rs
git commit -m "refactor(persistence): remove blocking dialog code and dead helpers"
```

---

### Task 8: Add Tests for New Code

**Files:**

- Modify: `src/persistence/tests.rs`

**Step 1: Add `save_to_path` test**

```rust
/// `save_to_path` writes file to disk and updates workspace.
#[test]
fn save_to_path_writes_file_and_updates_workspace() {
    let mut app = test_app();
    app.insert_resource(HexGridConfig {
        layout: hexx::HexLayout {
            orientation: hexx::HexOrientation::Pointy,
            scale: bevy::math::Vec2::splat(1.0),
            origin: bevy::math::Vec2::ZERO,
        },
        map_radius: 5,
    });
    app.update();

    let tmp = std::env::temp_dir().join("hexorder_test_save_to_path.hexorder");
    let _ = std::fs::remove_file(&tmp);

    let result = super::systems::save_to_path(&tmp, app.world_mut());

    assert!(result, "save_to_path should succeed");
    assert!(tmp.exists(), "file should be written to disk");

    let workspace = app
        .world()
        .resource::<hexorder_contracts::persistence::Workspace>();
    assert_eq!(workspace.file_path.as_deref(), Some(tmp.as_path()));
    assert!(!workspace.dirty);

    let _ = std::fs::remove_file(&tmp);
}
```

Note: `save_to_path` must be `pub(crate)` (not just `fn`) for test access. Update its visibility
when adding this test.

**Step 2: Add `load_from_path` test**

```rust
/// `load_from_path` overwrites registries and inserts PendingBoardLoad.
#[test]
fn load_from_path_overwrites_registries() {
    use hexorder_contracts::storage::Storage;

    let mut app = test_app();
    app.insert_resource(HexGridConfig {
        layout: hexx::HexLayout {
            orientation: hexx::HexOrientation::Pointy,
            scale: bevy::math::Vec2::splat(1.0),
            origin: bevy::math::Vec2::ZERO,
        },
        map_radius: 5,
    });
    app.update();

    // Write a test file to disk first.
    let file = test_game_system_file();
    let tmp = std::env::temp_dir().join("hexorder_test_load_from_path.hexorder");
    {
        let storage = app.world().resource::<Storage>();
        storage
            .provider()
            .save_at(&tmp, &file)
            .expect("write test file");
    }

    let result = super::systems::load_from_path(&tmp, app.world_mut());

    assert!(result, "load_from_path should succeed");

    let game_system = app.world().resource::<GameSystem>();
    assert_eq!(game_system.id, "test-save");

    let workspace = app
        .world()
        .resource::<hexorder_contracts::persistence::Workspace>();
    assert_eq!(workspace.name, "Test Project");
    assert_eq!(workspace.file_path.as_deref(), Some(tmp.as_path()));
    assert!(!workspace.dirty);

    assert!(
        app.world().get_resource::<PendingBoardLoad>().is_some(),
        "PendingBoardLoad should be inserted"
    );

    let _ = std::fs::remove_file(&tmp);
}
```

Note: `load_from_path` must also be `pub(crate)` for test access.

**Step 3: Add `dispatch_dialog_result` confirm-no test**

```rust
/// `dispatch_dialog_result` executes pending action on confirm No (skip save).
#[test]
fn dispatch_confirm_no_executes_pending_action() {
    use crate::persistence::async_dialog::*;

    let mut app = test_app();
    app.insert_resource(HexGridConfig {
        layout: hexx::HexLayout {
            orientation: hexx::HexOrientation::Pointy,
            scale: bevy::math::Vec2::splat(1.0),
            origin: bevy::math::Vec2::ZERO,
        },
        map_radius: 5,
    });
    app.update();

    // Dispatch: confirm No with pending NewProject action.
    super::systems::dispatch_dialog_result(
        DialogKind::ConfirmUnsavedChanges {
            then: PendingAction::NewProject {
                name: "Test".to_string(),
            },
        },
        DialogResult::Confirmed(ConfirmChoice::No),
        app.world_mut(),
    );

    // NewProject should have set workspace name and transitioned to Editor.
    let workspace = app
        .world()
        .resource::<hexorder_contracts::persistence::Workspace>();
    assert_eq!(workspace.name, "Test");
}
```

Note: `dispatch_dialog_result` must be `pub(crate)` for test access.

**Step 4: Add `dispatch_dialog_result` confirm-cancel test**

```rust
/// `dispatch_dialog_result` does nothing on confirm Cancel.
#[test]
fn dispatch_confirm_cancel_does_nothing() {
    use crate::persistence::async_dialog::*;

    let mut app = test_app();
    app.insert_resource(HexGridConfig {
        layout: hexx::HexLayout {
            orientation: hexx::HexOrientation::Pointy,
            scale: bevy::math::Vec2::splat(1.0),
            origin: bevy::math::Vec2::ZERO,
        },
        map_radius: 5,
    });
    app.update();

    // Set workspace name so we can verify it's unchanged.
    app.world_mut()
        .resource_mut::<hexorder_contracts::persistence::Workspace>()
        .name = "Original".to_string();

    super::systems::dispatch_dialog_result(
        DialogKind::ConfirmUnsavedChanges {
            then: PendingAction::CloseProject,
        },
        DialogResult::Confirmed(ConfirmChoice::Cancel),
        app.world_mut(),
    );

    // Workspace should be unchanged.
    let workspace = app
        .world()
        .resource::<hexorder_contracts::persistence::Workspace>();
    assert_eq!(workspace.name, "Original");
}
```

**Step 5: Run tests**

Run: `cargo test persistence`

Expected: All tests pass including 4 new tests.

**Step 6: Commit**

```bash
git add src/persistence/systems.rs src/persistence/tests.rs
git commit -m "test(persistence): add tests for save_to_path, load_from_path, and dispatch"
```

---

### Task 9: Quality Checks and Documentation

**Files:**

- Modify: `docs/plugins/persistence/spec.md`
- Modify: `docs/plugins/persistence/log.md`

**Step 1: Run full quality gate**

```bash
mise check
```

Expected: All checks pass. Fix any clippy warnings, formatting issues, or boundary violations.

**Step 2: Update spec.md**

In the Constraints section, update the async dialog constraint to note that all dialogs are now
async:

```markdown
- Async file dialogs via `AsyncDialogTask` resource — only one dialog at a time, polled each frame
- Dialog chaining via `then: Option<PendingAction>` on `SaveFile` enables confirm → save → action
  flows
```

**Step 3: Update log.md**

Append a new scope entry:

```markdown
### Scope 2: Async Dialog Migration

- Migrated all 4 blocking dialog call sites to async infrastructure
- Extracted pure helpers: `save_to_path`, `load_from_path`, `reset_to_new_project`, `close_project`
- Added `handle_dialog_completed` observer with central dispatch for all dialog result combinations
- Dialog chaining: `then: Option<PendingAction>` on `SaveFile` enables confirm → save → action
- Removed: `check_unsaved_changes()`, `ConfirmAction`, `clear_keyboard_after_dialog()`, `do_save()`
- All observers converted to thin dispatchers using `commands.queue()` for exclusive world access
- N tests total (M new), all passing
```

Replace N and M with actual counts after running tests.

**Step 4: Commit**

```bash
git add docs/plugins/persistence/spec.md docs/plugins/persistence/log.md
git commit -m "docs(persistence): log Scope 2 async dialog migration"
```
