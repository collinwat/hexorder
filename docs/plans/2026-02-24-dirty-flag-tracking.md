# Dirty Flag Tracking Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan
> task-by-task.

**Goal:** Wire `Workspace.dirty` so it tracks unsaved changes via the UndoStack, and show a
confirmation dialog when the user attempts to load/new/close with unsaved work.

**Architecture:** Add a `has_new_records` flag to `UndoStack` that gets set automatically by
`record()`. A sync system in the persistence plugin reads this flag each frame and propagates it to
`Workspace.dirty`. The three destructive observer handlers (load, new, close) gain a dirty guard
that shows an `rfd::MessageDialog` before proceeding.

**Tech Stack:** Rust, Bevy 0.18, rfd 0.15 (`MessageDialog`)

---

## Task 1: Add `has_new_records` flag to UndoStack

**Files:**

- Modify: `src/contracts/undo_redo.rs:43-95` (UndoStack struct + record method)
- Modify: `docs/contracts/undo-redo.md` (document new field and methods)

**Step 1: Write failing tests for the new flag**

Add these tests to the existing `mod tests` block at the bottom of `src/contracts/undo_redo.rs`:

```rust
#[test]
fn record_sets_has_new_records() {
    let mut stack = UndoStack::default();
    assert!(!stack.has_new_records());

    stack.record(make_cmd("action"));
    assert!(stack.has_new_records());
}

#[test]
fn acknowledge_records_clears_flag() {
    let mut stack = UndoStack::default();
    stack.record(make_cmd("action"));
    assert!(stack.has_new_records());

    stack.acknowledge_records();
    assert!(!stack.has_new_records());
}

#[test]
fn clear_resets_has_new_records() {
    let mut stack = UndoStack::default();
    stack.record(make_cmd("action"));
    stack.clear();
    assert!(!stack.has_new_records());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib contracts::undo_redo` Expected: FAIL — `has_new_records` and
`acknowledge_records` do not exist.

**Step 3: Implement the flag**

In `src/contracts/undo_redo.rs`, add the field and methods:

1. Add field to `UndoStack` struct (after `pending_redo`):

```rust
    /// Set by `record()`, cleared by `acknowledge_records()`.
    /// Used by the persistence sync system to detect new commands.
    has_new_records: bool,
```

2. Add `has_new_records: false` to both `Default::default()` and `with_max_depth()`.

3. Add `self.has_new_records = true;` at the end of the `record()` method.

4. Add `self.has_new_records = false;` at the end of the `clear()` method.

5. Add the `has_new_records` field to the `Debug` impl's `debug_struct`.

6. Add two new public methods after `clear()`:

```rust
    /// Returns `true` if commands have been recorded since the last
    /// `acknowledge_records()` call.
    #[must_use]
    pub fn has_new_records(&self) -> bool {
        self.has_new_records
    }

    /// Clear the `has_new_records` flag. Called by the persistence sync
    /// system after propagating dirty state.
    pub fn acknowledge_records(&mut self) {
        self.has_new_records = false;
    }
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --lib contracts::undo_redo` Expected: All tests PASS (existing + 3 new).

**Step 5: Update the contract spec**

In `docs/contracts/undo-redo.md`, add to the UndoStack fields:

- `has_new_records: bool` — set by `record()`, cleared by `acknowledge_records()`

Add to Methods:

- `has_new_records() -> bool`
- `acknowledge_records()` — clear the flag after syncing

**Step 6: Commit**

```bash
git add src/contracts/undo_redo.rs docs/contracts/undo-redo.md
git commit -m "feat(undo_redo): add has_new_records flag to UndoStack (ref #172)"
```

---

## Task 2: Add dirty flag sync system to persistence plugin

**Files:**

- Modify: `src/persistence/systems.rs` (add `sync_dirty_flag` system)
- Modify: `src/persistence/mod.rs` (register the system)
- Modify: `src/persistence/tests.rs` (add test)

**Step 1: Write failing test**

Add to `src/persistence/tests.rs`:

```rust
use crate::contracts::undo_redo::UndoStack;

/// `sync_dirty_flag` sets `workspace.dirty` when UndoStack has new records.
#[test]
fn sync_dirty_flag_sets_dirty_on_new_records() {
    let mut app = test_app();
    app.init_resource::<UndoStack>();
    app.update();

    // Record a command.
    app.world_mut()
        .resource_mut::<UndoStack>()
        .record(Box::new(crate::contracts::undo_redo::SetPropertyCommand {
            entity: Entity::PLACEHOLDER,
            property_id: TypeId::new(),
            old_value: crate::contracts::game_system::PropertyValue::Int(0),
            new_value: crate::contracts::game_system::PropertyValue::Int(1),
            label: "test".to_string(),
        }));

    app.update(); // sync_dirty_flag runs

    let workspace = app.world().resource::<crate::contracts::persistence::Workspace>();
    assert!(workspace.dirty, "workspace should be dirty after record");

    // Flag should be acknowledged.
    let stack = app.world().resource::<UndoStack>();
    assert!(
        !stack.has_new_records(),
        "has_new_records should be cleared after sync"
    );
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib persistence::tests::sync_dirty_flag_sets_dirty_on_new_records` Expected: FAIL
— `sync_dirty_flag` does not exist / is not registered.

**Step 3: Implement the sync system**

Add to `src/persistence/systems.rs` (in the Update Systems section):

```rust
/// Propagates the UndoStack's `has_new_records` flag to `Workspace.dirty`.
/// Runs every frame in `Update`. When new commands have been recorded,
/// sets dirty to true and acknowledges the records.
pub fn sync_dirty_flag(
    mut undo_stack: ResMut<crate::contracts::undo_redo::UndoStack>,
    mut workspace: ResMut<Workspace>,
) {
    if undo_stack.has_new_records() {
        workspace.dirty = true;
        undo_stack.acknowledge_records();
    }
}
```

Register it in `src/persistence/mod.rs` — add to the `Update` systems block:

```rust
app.add_systems(
    Update,
    (
        systems::apply_pending_board_load
            .run_if(in_state(AppScreen::Editor).or(in_state(AppScreen::Play))),
        systems::sync_dirty_flag
            .run_if(in_state(AppScreen::Editor).or(in_state(AppScreen::Play))),
    ),
);
```

(Replace the existing single `add_systems(Update, ...)` call with a tuple.)

**Step 4: Run tests to verify they pass**

Run: `cargo test --lib persistence` Expected: All persistence tests PASS.

**Step 5: Commit**

```bash
git add src/persistence/systems.rs src/persistence/mod.rs src/persistence/tests.rs
git commit -m "feat(persistence): add sync_dirty_flag system (ref #172)"
```

---

## Task 3: Extract save logic into a shared helper

The confirmation dialog needs to optionally save before proceeding. Extract the save logic from
`handle_save_request` into a helper that both the observer and the dirty guard can call.

**Files:**

- Modify: `src/persistence/systems.rs` (extract helper, refactor handler)

**Step 1: Run existing tests as baseline**

Run: `cargo test --lib persistence` Expected: All PASS.

**Step 2: Extract `do_save` helper**

Add a helper function in the Shared Helpers section of `src/persistence/systems.rs`. This function
contains the save-to-disk logic extracted from `handle_save_request`. It returns `true` if the save
succeeded (or was not needed), `false` if the user cancelled or it failed:

```rust
/// Perform the save operation. Returns `true` if save succeeded, `false` if
/// cancelled or failed. When `force_dialog` is true, always shows the file
/// picker (Save As behavior).
#[allow(clippy::too_many_arguments)]
fn do_save(
    force_dialog: bool,
    workspace: &mut Workspace,
    game_system: &GameSystem,
    entity_types: &EntityTypeRegistry,
    enum_registry: &EnumRegistry,
    struct_registry: &StructRegistry,
    concepts: &ConceptRegistry,
    relations: &RelationRegistry,
    constraints: &ConstraintRegistry,
    turn_structure: &TurnStructure,
    crt: &CombatResultsTable,
    combat_modifiers: &CombatModifierRegistry,
    config: &HexGridConfig,
    tiles: &[(HexPosition, EntityData)],
    units: &[(HexPosition, EntityData)],
    storage: &Storage,
    commands: &mut Commands,
) -> bool {
    // Determine target path.
    let path = if force_dialog || workspace.file_path.is_none() {
        let sanitized_name = sanitize_filename(&workspace.name);
        let file_name = format!("{sanitized_name}.hexorder");

        let mut dialog = rfd::FileDialog::new()
            .add_filter("Hexorder", &["hexorder"])
            .set_file_name(&file_name);

        if let Some(ref existing) = workspace.file_path {
            if let Some(parent) = existing.parent() {
                dialog = dialog.set_directory(parent);
            }
        } else {
            let base = storage.provider().base_dir();
            if std::fs::create_dir_all(base).is_ok() {
                dialog = dialog.set_directory(base);
            }
        }

        let result = dialog.save_file();
        clear_keyboard_after_dialog(commands);
        match result {
            Some(p) => p,
            None => return false, // User cancelled.
        }
    } else {
        workspace.file_path.clone().expect("checked is_some above")
    };

    // Build save data.
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

    let file = GameSystemFile {
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
    };

    match storage.provider().save_at(&path, &file) {
        Ok(()) => {
            info!("Saved to {}", path.display());
            workspace.file_path = Some(path);
            workspace.dirty = false;
            commands.trigger(ToastEvent {
                message: "Project saved".to_string(),
                kind: ToastKind::Success,
            });
            true
        }
        Err(e) => {
            error!("Failed to save: {e}");
            commands.trigger(ToastEvent {
                message: format!("Save failed: {e}"),
                kind: ToastKind::Error,
            });
            false
        }
    }
}
```

**Step 3: Refactor `handle_save_request` to use the helper**

Replace the body of `handle_save_request` with:

```rust
pub fn handle_save_request(
    trigger: On<SaveRequestEvent>,
    game_system: Res<GameSystem>,
    entity_types: Res<EntityTypeRegistry>,
    enum_registry: Res<EnumRegistry>,
    struct_registry: Res<StructRegistry>,
    concepts: Res<ConceptRegistry>,
    relations: Res<RelationRegistry>,
    constraints: Res<ConstraintRegistry>,
    turn_structure: Res<TurnStructure>,
    crt: Res<CombatResultsTable>,
    combat_modifiers: Res<CombatModifierRegistry>,
    config: Res<HexGridConfig>,
    tiles: Query<(&HexPosition, &EntityData), With<HexTile>>,
    units: Query<(&HexPosition, &EntityData), With<UnitInstance>>,
    storage: Res<Storage>,
    mut workspace: ResMut<Workspace>,
    mut commands: Commands,
) {
    let tile_vec: Vec<_> = tiles.iter().map(|(p, d)| (*p, d.clone())).collect();
    let unit_vec: Vec<_> = units.iter().map(|(p, d)| (*p, d.clone())).collect();

    do_save(
        trigger.event().save_as,
        &mut workspace,
        &game_system,
        &entity_types,
        &enum_registry,
        &struct_registry,
        &concepts,
        &relations,
        &constraints,
        &turn_structure,
        &crt,
        &combat_modifiers,
        &config,
        &tile_vec,
        &unit_vec,
        &storage,
        &mut commands,
    );
}
```

**Step 4: Run tests to verify refactor is clean**

Run: `cargo test --lib persistence && cargo clippy --all-targets` Expected: All PASS, zero warnings.

**Step 5: Commit**

```bash
git add src/persistence/systems.rs
git commit -m "refactor(persistence): extract do_save helper for reuse (ref #172)"
```

---

## Task 4: Add dirty guard to load/new/close handlers

**Files:**

- Modify: `src/persistence/systems.rs` (add guard logic to three handlers)

**Step 1: Write failing test for the dirty guard helper**

Add to `src/persistence/tests.rs`:

```rust
use crate::persistence::systems::ConfirmAction;

/// `check_unsaved_changes` returns `Proceed` when workspace is clean.
#[test]
fn check_unsaved_changes_proceeds_when_clean() {
    let workspace = crate::contracts::persistence::Workspace::default();
    assert_eq!(
        crate::persistence::systems::check_unsaved_changes(&workspace),
        ConfirmAction::Proceed,
    );
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib persistence::tests::check_unsaved_changes_proceeds_when_clean` Expected: FAIL
— `check_unsaved_changes` and `ConfirmAction` do not exist.

**Step 3: Implement the dirty guard**

Add to `src/persistence/systems.rs` (in the Shared Helpers section):

```rust
/// Result of the unsaved-changes confirmation dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfirmAction {
    /// No unsaved changes, or user chose "Don't Save" — proceed with the action.
    Proceed,
    /// User chose "Save" and save succeeded — proceed with the action.
    SavedThenProceed,
    /// User cancelled or save failed — abort the action.
    Cancel,
}

/// Check for unsaved changes and prompt the user if dirty.
/// Returns the action to take. When `workspace.dirty` is false, returns
/// `Proceed` immediately without showing a dialog.
///
/// This function is NOT called in tests because it shows a blocking dialog.
/// Tests verify the `ConfirmAction` enum and the clean-path logic.
pub(crate) fn check_unsaved_changes(workspace: &Workspace) -> ConfirmAction {
    if !workspace.dirty {
        return ConfirmAction::Proceed;
    }

    let result = rfd::MessageDialog::new()
        .set_title("Unsaved Changes")
        .set_description("You have unsaved changes. Do you want to save before continuing?")
        .set_buttons(rfd::MessageButtons::YesNoCancel)
        .set_level(rfd::MessageLevel::Warning)
        .show();

    match result {
        rfd::MessageDialogResult::Yes => ConfirmAction::SavedThenProceed,
        rfd::MessageDialogResult::No => ConfirmAction::Proceed,
        _ => ConfirmAction::Cancel,
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib persistence::tests::check_unsaved_changes_proceeds_when_clean` Expected:
PASS.

**Step 5: Add dirty guard to `handle_load_request`**

At the top of `handle_load_request` (after the function signature opens), add:

```rust
    let confirm = check_unsaved_changes(&workspace);
    match confirm {
        ConfirmAction::Cancel => return,
        ConfirmAction::SavedThenProceed => {
            let tile_vec: Vec<_> = tiles.iter().map(|(p, d)| (*p, d.clone())).collect();
            let unit_vec: Vec<_> = units.iter().map(|(p, d)| (*p, d.clone())).collect();
            if !do_save(
                false, &mut workspace, &game_system, &entity_types,
                &enum_registry, &struct_registry, &concepts, &relations,
                &constraints, &turn_structure, &crt, &combat_modifiers,
                &config, &tile_vec, &unit_vec, &storage, &mut commands,
            ) {
                return; // Save cancelled or failed — abort load.
            }
        }
        ConfirmAction::Proceed => {}
    }
```

This requires adding extra parameters to `handle_load_request`'s signature:

- `tiles: Query<(&HexPosition, &EntityData), With<HexTile>>`
- `units_q: Query<(&HexPosition, &EntityData), With<UnitInstance>>`
- `config: Res<HexGridConfig>`
- `turn_structure: Res<TurnStructure>`
- `crt: Res<CombatResultsTable>`
- `combat_modifiers: Res<CombatModifierRegistry>`

**Step 6: Add dirty guard to `handle_new_project`**

At the top of `handle_new_project`, add the same pattern. Since `handle_new_project` already has
most parameters, add any missing ones (tiles, units queries, config, storage) for the save path.

**Step 7: Add dirty guard to `handle_close_project`**

Same pattern at the top of `handle_close_project`. Add the required parameters.

**Step 8: Run all tests**

Run: `cargo test --lib persistence && cargo clippy --all-targets` Expected: All PASS, zero warnings.

**Step 9: Commit**

```bash
git add src/persistence/systems.rs src/persistence/tests.rs
git commit -m "feat(persistence): add unsaved-changes confirmation dialog (ref #172)"
```

---

## Task 5: Remove the TODO comment and update spec

**Files:**

- Modify: `src/contracts/persistence.rs:48` (remove TODO)
- Modify: `docs/contracts/persistence.md` (update dirty field description)
- Modify: `docs/plugins/editor-ui/spec.md` (update success criteria)

**Step 1: Remove the TODO comment**

In `src/contracts/persistence.rs`, replace line 48:

```
    /// TODO(#111): not actively tracked yet — see unsaved changes issue.
```

with:

```
    /// Tracked via `UndoStack.has_new_records()` — see `sync_dirty_flag` system.
```

**Step 2: Update the persistence contract spec**

In `docs/contracts/persistence.md`, change the `dirty` field description from:

```
Whether project has unsaved changes (placeholder)
```

to:

```
Whether project has unsaved changes (tracked by sync_dirty_flag)
```

**Step 3: Run full check**

Run: `cargo test && cargo clippy --all-targets` Expected: All PASS.

**Step 4: Commit**

```bash
git add src/contracts/persistence.rs docs/contracts/persistence.md docs/plugins/editor-ui/spec.md
git commit -m "docs(persistence): update dirty flag documentation (ref #172)"
```

---

## Task 6: Run full audit and verify

**Step 1: Run the full check suite**

Run: `mise check` Expected: All checks pass.

**Step 2: Run boundary check**

Run: `mise check:boundary` Expected: No cross-plugin internal imports.

**Step 3: Verify manually**

- [ ] `UndoStack.has_new_records` is set on `record()` only
- [ ] `sync_dirty_flag` runs each frame and propagates to `Workspace.dirty`
- [ ] Save clears `Workspace.dirty`
- [ ] Load/New/Close show confirmation dialog when dirty
- [ ] Confirmation dialog offers Save/Don't Save/Cancel
- [ ] "Save" performs save, then continues
- [ ] "Don't Save" continues without saving
- [ ] "Cancel" aborts the action
- [ ] No `unwrap()` in production code
