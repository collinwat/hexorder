# Coverage Improvement to 95% — Implementation Plan

> **For Claude:** Execute this plan sequentially in a single session using
> `superpowers:executing-plans`. No subagents or parallel worktrees — work through tasks one at a
> time, committing after each.

**Goal:** Raise test coverage from 50.98% to 95% across the hexorder workspace.

**Architecture:** Tiered bottom-up approach. Phase 0 validates editor_ui testability. Phases 1-3 add
tests in quick-win → moderate → heavy-lift order. Each phase parallelizes independent module
workstreams. Tests live in existing `tests.rs` / `ui_tests.rs` files. No production code changes
except where needed for testability.

**Tech Stack:** Rust, Bevy 0.18 (MinimalPlugins for testing), egui_kittest::Harness, RON
serialization, cargo-llvm-cov

---

## Phase 0: Feasibility Spike (editor_ui/systems.rs)

### Task 0.1: Audit editor_ui/systems.rs testability

**Files:**

- Read: `src/editor_ui/systems.rs`
- Read: `src/editor_ui/ui_tests.rs`
- Read: `src/editor_ui/tests.rs`

**Step 1: Categorize uncovered code in editor_ui/systems.rs**

Read `src/editor_ui/systems.rs` (6,211 lines, 13.9% covered). Classify uncovered functions into:

- **Category A**: Pure rendering (egui widget calls) — testable with `Harness`
- **Category B**: State mutation triggered by UI events — testable with Bevy App
- **Category C**: Bevy system glue (resource reads, scheduling) — testable with App integration

**Step 2: Write one representative test per category**

Add to `src/editor_ui/ui_tests.rs`. Model after existing pattern:

```rust
#[test]
fn render_category_a_example() {
    let mut state = EditorState::default();
    let _harness = Harness::new_ui(|ui| {
        systems::render_some_panel(ui, &mut state);
    });
    // Assert state or harness output
}
```

**Step 3: Run tests and measure coverage delta**

Run: `cargo llvm-cov --features dev --summary-only 2>&1 | grep 'editor_ui/systems.rs'`

**Step 4: Estimate total effort**

Calculate: (lines covered by 3 tests) × (number of similar functions) ≈ effort to reach 95%.

**Go/No-Go:** If representative tests each cover 50+ lines with minimal setup, proceed. If egui
rendering requires extensive mocking or the Harness can't handle the patterns, adjust the 95% target
for this file.

---

## Phase 1: Quick Wins (~325 missed lines → ~54% total)

### Task 1.A: Contracts Coverage

**Files:**

- Modify: `crates/hexorder-contracts/src/validation.rs` (add `#[cfg(test)]` module)
- Modify: `crates/hexorder-contracts/src/persistence.rs` (add tests)
- Modify: `crates/hexorder-contracts/src/settings.rs` (add tests)
- Modify: `crates/hexorder-contracts/src/editor_ui.rs` (add tests)
- Modify: `crates/hexorder-contracts/src/shortcuts.rs` (add tests)
- Modify: `crates/hexorder-contracts/src/ontology.rs` (add tests)
- Modify: `crates/hexorder-contracts/src/mechanics.rs` (add tests)
- Modify: `crates/hexorder-contracts/src/game_system.rs` (add tests)
- Modify: `crates/hexorder-contracts/src/hex_grid.rs` (add tests)
- Modify: `crates/hexorder-contracts/src/undo_redo.rs` (add tests)

#### Step 1: Write failing tests for validation.rs (0% → 95%)

All 5 lines are type definitions. Add a test module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_error_construction() {
        let error = SchemaError {
            category: SchemaErrorCategory::MissingField,
            message: "name is required".to_string(),
        };
        assert_eq!(error.message, "name is required");
    }

    #[test]
    fn schema_validation_default_is_valid() {
        let validation = SchemaValidation::default();
        assert!(validation.errors.is_empty());
    }

    #[test]
    fn valid_move_set_default_is_empty() {
        let moves = ValidMoveSet::default();
        assert!(moves.valid_positions.is_empty());
        assert!(moves.blocked_explanations.is_empty());
    }
}
```

Run: `cargo test -p hexorder-contracts -- validation::tests -v` Expected: PASS

#### Step 2: Write failing tests for persistence.rs Display impls (41% → 95%)

Add to existing test module:

```rust
#[test]
fn persistence_error_display_serialize() {
    let err = PersistenceError::Serialize("bad data".to_string());
    assert_eq!(format!("{err}"), "serialization error: bad data");
}

#[test]
fn persistence_error_display_deserialize() {
    let err = PersistenceError::Deserialize("unexpected token".to_string());
    assert_eq!(format!("{err}"), "deserialization error: unexpected token");
}

#[test]
fn game_system_file_ron_round_trip() {
    let file = GameSystemFile { /* construct with sample data */ };
    let ron_str = ron::to_string(&file).expect("serialize");
    let _: GameSystemFile = ron::from_str(&ron_str).expect("deserialize");
}

#[test]
fn workspace_with_custom_fields() {
    let ws = Workspace {
        file_path: Some(std::path::PathBuf::from("/tmp/test.ron")),
        dirty: true,
        ..Workspace::default()
    };
    assert!(ws.dirty);
    assert_eq!(ws.file_path.unwrap().to_str().unwrap(), "/tmp/test.ron");
}
```

Run: `cargo test -p hexorder-contracts -- persistence::tests -v` Expected: PASS

#### Step 3: Write tests for settings.rs ThemeLibrary (53.9% → 95%)

```rust
#[test]
fn theme_library_find_returns_matching_theme() {
    let lib = ThemeLibrary {
        themes: vec![ThemeDefinition {
            name: "Dark".to_string(),
            /* fill all required color fields */
        }],
    };
    assert!(lib.find("Dark").is_some());
    assert!(lib.find("Light").is_none());
}

#[test]
fn editor_settings_custom_values() {
    let settings = EditorSettings {
        font_size: 18.0,
        workspace_preset: "wargame".to_string(),
    };
    assert_eq!(settings.font_size, 18.0);
}
```

Run: `cargo test -p hexorder-contracts -- settings::tests -v`

#### Step 4: Write tests for shortcuts.rs methods (70.1% → 95%)

```rust
#[test]
fn key_binding_display_string_cmd_s() {
    let binding = KeyBinding {
        key: KeyCode::KeyS,
        modifiers: Modifiers::CMD,
    };
    let display = binding.display_string();
    assert!(display.contains("⌘"));
    assert!(display.contains("S"));
}

#[test]
fn shortcut_registry_lookup_returns_matching_command() {
    let mut registry = ShortcutRegistry::default();
    registry.register(/* command with KeyCode::KeyS + CMD */);
    let binding = KeyBinding { key: KeyCode::KeyS, modifiers: Modifiers::CMD };
    assert!(registry.lookup(&binding).is_some());
}

#[test]
fn shortcut_registry_bindings_for_returns_keycodes() {
    let mut registry = ShortcutRegistry::default();
    registry.register(/* command "file.save" with KeyS */);
    let bindings = registry.bindings_for("file.save");
    assert_eq!(bindings.len(), 1);
}

#[test]
fn shortcut_registry_discrete_commands_filters_correctly() {
    let mut registry = ShortcutRegistry::default();
    registry.register(/* discrete command */);
    registry.register(/* continuous command */);
    let discrete = registry.discrete_commands();
    assert_eq!(discrete.len(), 1);
}

#[test]
fn shortcut_registry_override_bindings_replaces() {
    let mut registry = ShortcutRegistry::default();
    registry.register(/* command with KeyS */);
    registry.override_bindings("file.save", vec![KeyBinding { key: KeyCode::KeyP, .. }]);
    let bindings = registry.bindings_for("file.save");
    assert!(bindings.contains(&KeyCode::KeyP));
}

#[test]
fn command_id_display() {
    let id = CommandId("file.save");
    assert_eq!(format!("{id}"), "file.save");
}
```

Run: `cargo test -p hexorder-contracts -- shortcuts::tests -v`

#### Step 5: Write tests for remaining contract gaps

Cover: `editor_ui.rs` (pointer_over_ui_panel — Bevy system, test in plugin tests), `ontology.rs`
(missing ConstraintExpr variants), `mechanics.rs` (edge cases), `game_system.rs` (get_mut mutation),
`hex_grid.rs` (empty iterator), `undo_redo.rs` (missing entity paths, debug impls).

Follow the exact patterns identified in the analysis. Each test is a simple unit test with RON
round-trip or direct method call.

#### Step 6: Run full contract coverage

Run: `cargo llvm-cov --features dev --summary-only --workspace 2>&1 | grep 'hexorder-contracts'`
Expected: All contract files ≥ 95%

#### Step 7: Commit

```bash
git add crates/hexorder-contracts/src/*.rs
git commit -m "test(contracts): improve coverage to 95%+ across all contract types"
```

---

### Task 1.B: Unit + Cell + Undo/Redo Coverage

**Files:**

- Modify: `src/unit/tests.rs`
- Modify: `src/cell/tests.rs`
- Modify: `src/undo_redo/tests.rs`

#### Step 1: Write tests for unit/systems.rs guard branches (91.2% → 95%)

```rust
#[test]
fn handle_unit_placement_noop_when_no_active_type() {
    let mut app = test_app();
    setup_unit_resources(&mut app);
    // Set EditorTool::Place but leave ActiveUnit.entity_type_id as None
    app.insert_resource(EditorTool::Place);
    app.update();
    // Trigger placement event — should return early without spawning
}

#[test]
fn handle_unit_placement_noop_when_type_not_in_registry() {
    // Set ActiveUnit with a TypeId that doesn't exist in EntityTypeRegistry
}

#[test]
fn delete_selected_unit_clears_selection_when_entity_gone() {
    // Select a unit, despawn the entity directly, then run delete system
}

#[test]
fn sync_unit_materials_creates_material_for_new_type() {
    // Add a new Token type to registry, run sync, verify material exists
}
```

Run: `cargo test --lib unit -v` Expected: PASS, coverage ≥ 95%

#### Step 2: Write tests for cell/systems.rs guard branches (92.2% → 95%)

```rust
#[test]
fn assign_default_cell_data_noop_when_no_board_position_type() {
    // Create registry with only Token types, no BoardPosition
}

#[test]
fn paint_cell_noop_when_no_active_board_type() {
    // Set EditorTool::Paint but ActiveBoard.entity_type_id is None
}

#[test]
fn sync_cell_materials_updates_existing_material_color() {
    // Change a BoardPosition type's color, run sync, verify material updated
}
```

Run: `cargo test --lib cell -v` Expected: PASS, coverage ≥ 95%

#### Step 3: Write tests for undo_redo/systems.rs remaining 2 lines (95.2% → 98%)

Identify the 2 missed lines and add targeted test.

Run: `cargo test --lib undo_redo -v`

#### Step 4: Commit

```bash
git add src/unit/tests.rs src/cell/tests.rs src/undo_redo/tests.rs
git commit -m "test(unit,cell,undo_redo): cover guard branches and edge cases"
```

---

### Task 1.C: Export Coverage

**Files:**

- Modify: `src/export/tests.rs`

#### Step 1: Write tests for counter_sheet.rs edge cases (90.1% → 95%)

```rust
#[test]
fn format_property_value_all_variants() {
    assert_eq!(format_property_value(&PropertyValue::Int(42)), "42");
    assert_eq!(format_property_value(&PropertyValue::Float(3.14)), "3.14");
    assert_eq!(format_property_value(&PropertyValue::Text("hello".into())), "hello");
    assert_eq!(format_property_value(&PropertyValue::Bool(true)), "true");
    assert_eq!(format_property_value(&PropertyValue::EntityRef(None)), "—");
    // Test remaining variants
}

#[test]
fn collect_counters_falls_back_to_type_definitions_when_no_instances() {
    // Create ExportData with token types but empty token_entities
}

#[test]
fn generate_counter_sheet_handles_oversized_counters() {
    // Set counter_size larger than page_size to trigger per_page == 0
}
```

Run: `cargo test --lib export -v`

#### Step 2: Write tests for export/systems.rs (40.3% → 90%+)

```rust
#[test]
fn handle_export_command_noop_when_already_pending() {
    // Insert PendingExport resource, trigger export command, verify no second dialog
}

#[test]
fn handle_export_command_noop_outside_editor_state() {
    // Set AppScreen to non-Editor state, trigger export, verify no action
}

#[test]
fn poll_pending_export_removes_resource_on_completion() {
    // Insert PendingExport with a ready future, run poll, verify resource removed
}

#[test]
fn run_export_triggers_success_toast() {
    // Provide mock exporter, call run_export, verify toast event
}

#[test]
fn run_export_triggers_error_toast_on_failure() {
    // Provide failing mock exporter, call run_export, verify error toast
}
```

Run: `cargo test --lib export -v`

#### Step 3: Write tests for export/mod.rs plugin registration (43.8% → 90%+)

```rust
#[test]
fn export_plugin_registers_shortcut() {
    let mut app = test_app();
    app.add_plugins(super::ExportPlugin);
    app.update();
    let registry = app.world().resource::<ShortcutRegistry>();
    assert!(registry.get("file.export_pnp").is_some());
}
```

Run: `cargo test --lib export -v`

#### Step 4: Commit

```bash
git add src/export/tests.rs
git commit -m "test(export): cover counter_sheet edge cases, systems, and plugin registration"
```

---

### Task 1.D: Main + Components Coverage

**Files:**

- Modify: `src/editor_ui/tests.rs` (for components.rs coverage)
- Modify: `src/main.rs` (for architecture test coverage — these are already test functions)

#### Step 1: Write tests for editor_ui/components.rs (86% → 95%)

```rust
#[test]
fn editor_state_default_has_expected_initial_values() {
    let state = EditorState::default();
    // Verify key fields have expected defaults
    assert_eq!(state.active_tab, OntologyTab::Types);
    // ... verify other important fields
}

#[test]
fn dock_tab_is_closeable() {
    assert!(!DockTab::MapView.is_closeable());
    // Test each variant
}

#[test]
fn dock_tab_display() {
    assert_eq!(format!("{}", DockTab::MapView), "Map View");
    // Test each variant
}

#[test]
fn create_default_dock_layout_has_expected_structure() {
    let (tree, _) = create_default_dock_layout();
    // Verify tab count, split structure
}
```

Run: `cargo test --lib editor_ui -v`

#### Step 2: Cover main.rs testable paths (85% → 92%+)

The architecture tests in main.rs are themselves test functions. The missed lines are likely in the
helper logic within those tests. Identify the specific missed lines with:

```bash
cargo llvm-cov --features dev --html 2>&1 && open target/llvm-cov/html/index.html
```

If missed lines are in `reveal_window` or `main()`, write:

```rust
#[test]
fn reveal_window_shows_after_delay_frames() {
    let mut app = headless_app();
    // Run 3 updates, verify window visibility
}
```

Run: `cargo test --lib -- reveal_window -v`

#### Step 3: Commit

```bash
git add src/editor_ui/tests.rs src/main.rs
git commit -m "test(editor_ui,main): cover components defaults, dock layouts, architecture helpers"
```

---

### Phase 1 Checkpoint

Run: `cargo llvm-cov --features dev --summary-only --workspace 2>&1 | grep TOTAL` Expected: ~54%
total coverage

---

## Phase 2: Moderate Effort (~1,443 missed lines → ~62.5% total)

### Task 2.E: hex_grid + map_gen

**Files:**

- Modify: `src/hex_grid/tests.rs` — cover systems.rs (56.4%) and mod.rs (0%)
- Modify: `src/map_gen/tests.rs` — cover biome.rs (56.9%), systems.rs (0%), mod.rs (0%)

**Key gaps:**

- `hex_grid/systems.rs`: Selection system branches, grid resize edge cases, coordinate conversion
  paths not exercised
- `hex_grid/mod.rs`: Plugin registration and `register_shortcuts()` — write App integration test
- `map_gen/systems.rs`: Entire file (97 lines) at 0% — terrain generation system untested
- `map_gen/biome.rs`: Biome assignment edge cases (56.9%)
- `map_gen/mod.rs`: Plugin registration (8 lines)

**Approach:** Bevy App tests with `MinimalPlugins`. Spawn grid entities, trigger systems, verify
state changes. For map_gen, test the generation pipeline end-to-end with deterministic seeds.

### Task 2.F: persistence + scripting

**Files:**

- Modify: `src/persistence/tests.rs` — cover systems.rs (54.9%), async_dialog.rs (58.5%)
- Modify: `src/scripting/tests.rs` — cover lua_api.rs (62.3%), mod.rs (0%), systems.rs (0%)

**Key gaps:**

- `persistence/systems.rs`: Save/load flow (284 missed lines), file dialog polling, autosave timer
- `persistence/async_dialog.rs`: Dialog state machine, cancellation paths
- `scripting/lua_api.rs`: Lua binding registration, API function coverage
- `scripting/systems.rs`: Lua runtime initialization (20 lines, 0%)

**Approach:** Mock filesystem with temp directories for persistence. For scripting, test Lua API
functions directly through the mlua runtime. Plugin registration tests for mod.rs files.

### Task 2.G: ontology + rules_engine

**Files:**

- Modify: `src/ontology/tests.rs` — cover systems.rs (65.3%)
- Modify: `src/rules_engine/tests.rs` — cover systems.rs (82.3%)

**Key gaps:**

- `ontology/systems.rs`: Concept binding resolution, constraint evaluation paths (121 missed lines)
- `rules_engine/systems.rs`: Rule execution paths, modifier application, CRT resolution (85 lines)

**Approach:** Pure data tests for rule evaluation. Bevy App tests for system integration. These
modules have well-structured logic that's straightforward to test.

### Task 2.H: camera + shortcuts config

**Files:**

- Modify: `src/camera/tests.rs` — cover mod.rs (69.9%)
- Modify: `src/shortcuts/tests.rs` — cover config.rs (70.6%)

**Key gaps:**

- `camera/mod.rs`: Plugin setup, resource initialization (43 missed lines)
- `shortcuts/config.rs`: TOML parsing, keybinding deserialization edge cases (126 missed lines)

**Approach:** App integration test for camera plugin. Pure function tests for TOML parsing with
various input formats and error cases.

### Phase 2 Checkpoint

Run: `cargo llvm-cov --features dev --summary-only --workspace 2>&1 | grep TOTAL` Expected: ~62.5%
total coverage

---

## Phase 3: Heavy Lift (~6,517 missed lines → ~95% total)

### Task 3.I: editor_ui/systems.rs + mod.rs (THE BIG ONE)

**Files:**

- Modify: `src/editor_ui/ui_tests.rs` — bulk of new UI rendering tests
- Modify: `src/editor_ui/tests.rs` — observer and system integration tests

**Key gaps (5,634 missed lines):**

- `systems.rs` (5,346 lines): Panel rendering functions, toolbar layout, property editors, dialog
  handling, dock tab content, status bar, toast rendering
- `mod.rs` (288 lines): Plugin registration, system scheduling, egui context setup

**Approach:** Systematic function-by-function coverage using `egui_kittest::Harness`. Group tests by
panel/feature area:

1. Toolbar and tool selection
2. Entity type editor panels
3. Property editor widgets
4. Dialog rendering (save/load/export)
5. Status bar and toast notifications
6. Dock tab content rendering
7. Settings panels
8. Plugin registration (App integration test)

Each rendering function gets a test that:

1. Creates minimal `EditorState` with relevant fields set
2. Wraps the function call in `Harness::new_ui(|ui| { ... })`
3. Asserts state changes or queries harness for rendered elements

**Estimated effort:** ~200-300 test functions. This is the largest single task.

### Task 3.J: camera/systems.rs + shortcuts/systems.rs

**Files:**

- Modify: `src/camera/tests.rs` — cover systems.rs (34.2%)
- Modify: `src/shortcuts/tests.rs` — cover systems.rs (11.3%), mod.rs (0%)

**Key gaps:**

- `camera/systems.rs` (260 missed lines): Mouse orbit/pan, scroll zoom, smooth interpolation, bounds
  enforcement, focus-on-selection
- `shortcuts/systems.rs` (260 missed lines): Shortcut matching, command palette toggle, command
  execution dispatch
- `shortcuts/mod.rs` (22 lines): Plugin registration

**Approach:** Bevy App tests with `ButtonInput<KeyCode>` and `ButtonInput<MouseButton>` resources to
simulate input. Test camera transforms after input simulation. For shortcuts, simulate key presses
and verify command dispatch.

### Task 3.K: settings + remaining 0% files

**Files:**

- Modify: `src/settings/tests.rs` — cover config.rs (34.8%), mod.rs (0%), systems.rs (0%)
- Verify: All remaining 0% mod.rs files covered by Phase 2 tasks

**Key gaps:**

- `settings/config.rs` (90 missed lines): TOML loading, layer merging, theme parsing
- `settings/mod.rs` (33 lines): Plugin initialization pipeline
- `settings/systems.rs` (32 lines): Settings system (project layer lifecycle)

**Approach:** Temp directory fixtures for TOML loading tests. App integration for plugin
registration. Pure function tests for layer merging logic.

### Phase 3 Checkpoint

Run: `cargo llvm-cov --features dev --summary-only --workspace 2>&1 | grep TOTAL` Expected: ~95%
total coverage

---

## Final Verification

Run full audit:

```bash
mise check:audit
```

Verify coverage gate:

```bash
cargo llvm-cov --features dev --summary-only --workspace
```

Expected: TOTAL line coverage ≥ 95%

---

## Execution Notes

- **Sequential execution**: Work through tasks one at a time in a single session — no subagents or
  parallel worktrees
- **No production code changes** except: extracting pure functions for testability where needed
- **Commit after each task** — atomic commits per module group
- **Measure coverage after each phase** — adjust subsequent phases based on actual deltas
- **User confirms** before moving to the next phase
