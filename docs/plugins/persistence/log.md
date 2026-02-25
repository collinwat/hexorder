# Plugin Log: Persistence

## 2026-02-13 — M5 Implementation

### Phase 1: Serde Foundation

- Added `serde 1.0` and `ron 0.12` to Cargo.toml
- Initially tried `ron 0.10` but Bevy uses `ron 0.12`, causing cargo deny ban failure (duplicate
  crate)
- Added Serialize/Deserialize to all persistent types in game_system, ontology, hex_grid contracts
- Added Clone to GameSystem, EntityTypeRegistry, ConceptRegistry, RelationRegistry,
  ConstraintRegistry
- 3 serde round-trip tests added (132 total)

### Phase 2: Persistence Contract + File I/O

- Created `src/contracts/persistence.rs` with GameSystemFile, TileSaveData, UnitSaveData
- Created `docs/contracts/persistence.md`
- RON pretty-printing for human-readable save files
- Format version check for forward compatibility
- 4 file I/O tests added (136 total)

### Phase 3: AppScreen State Machine

- Added AppScreen (Launcher/Editor) to contracts
- Gated all plugin systems with `in_state(AppScreen::Editor)` or `OnEnter(AppScreen::Editor)`
- Discovery: MinimalPlugins lacks StatesPlugin — must add `bevy::state::app::StatesPlugin`
  explicitly
- Updated all 7 feature test helpers + headless_app() integration helper
- 136 tests pass after state machine migration

### Phase 4: Save/Load Systems

- Created `src/persistence/` module with PersistencePlugin, systems, tests
- Added `rfd 0.15` for native macOS file dialogs (blocking API)
- Observer events: SaveRequestEvent, LoadRequestEvent, NewProjectEvent
- handle_save_request builds GameSystemFile from ECS queries
- handle_load_request overwrites registries and inserts PendingBoardLoad
- handle*new_project resets to factory defaults via game_system::create*\*
- keyboard_shortcuts uses Option<Res<ButtonInput<KeyCode>>> for test compatibility
- apply_pending_board_load spawns units with core components only (sync systems add visuals)
- 3 persistence tests added (139 total)

### Phase 5: Launcher UI + Integration

- Moved event types (SaveRequestEvent, LoadRequestEvent, NewProjectEvent) to contracts
- Added launcher_system to editor_ui with centered New/Open buttons
- Registered launcher_system with in_state(AppScreen::Launcher) gate
- Created feature spec and log files
- Updated coordination.md
- 139 tests pass, clippy clean

## 2026-02-24 — Cycle 8: Async File Dialogs (#175)

### Scope 1: Async Dialog Wrapper

- Created `src/persistence/async_dialog.rs` with infrastructure types:
    - `AsyncDialogTask` resource — holds in-flight `Task<DialogResult>`
    - `DialogKind` enum — SaveFile, OpenFile, PickFolder, ConfirmUnsavedChanges
    - `DialogResult` enum — FilePicked, FolderPicked, Confirmed
    - `ConfirmChoice` enum — Yes, No, Cancel
    - `PendingAction` enum — Load, NewProject, CloseProject
    - `DialogCompleted` observer event
- Spawn helpers: `spawn_save_dialog`, `spawn_open_dialog`, `spawn_folder_dialog`,
  `spawn_confirm_dialog`
- `poll_async_dialog` exclusive system — polls via `block_on(poll_once(...))`, zero-cost when idle
- Uses `IoTaskPool` (not `AsyncComputeTaskPool`) per pitch guidance
- Uses `bevy::tasks::block_on` / `bevy::tasks::poll_once` re-exports (no direct `futures-lite` dep)
- 5 unit tests added (386 total), all passing

### Scope 2: Async Dialog Migration

- Migrated all 4 blocking observer functions to async dialog infrastructure:
    - `handle_save_request` — spawns `SaveFile` dialog via `AsyncDialogTask`
    - `handle_load_request` — checks unsaved changes, spawns confirm or open dialog
    - `handle_new_project` — checks unsaved changes, spawns confirm or resets directly
    - `handle_close_project` — checks unsaved changes, spawns confirm or closes directly
- Added `then: Option<PendingAction>` to `DialogKind::SaveFile` for dialog chaining
- Extracted pure helpers that take `&mut World` for flexible resource access:
    - `build_game_system_file` — assembles `GameSystemFile` from world state
    - `save_to_path` — writes file and updates workspace
    - `load_from_path` — reads file, overwrites registries, inserts `PendingBoardLoad`
    - `reset_to_new_project` — resets all state for new project
    - `close_project` — returns to launcher screen
    - `spawn_save_dialog_for_current_project` — opens save dialog with workspace context
    - `reset_all_registries_world` — resets all registries to defaults
    - `execute_pending_action` — dispatches deferred actions after dialog completion
    - `dispatch_dialog_result` — central router for all dialog kind + result combinations
- Added `handle_dialog_completed` observer to bridge `DialogCompleted` event to dispatch
- Removed dead code: `clear_keyboard_after_dialog`, `reset_all_registries`, `ConfirmAction` enum,
  `check_unsaved_changes`, `do_save`, unused `KeyCode` import
- Net code change: significant reduction — ~332 lines removed from observer functions, ~213 lines of
  dead code removed; replaced with ~300 lines of cleaner async code
- 4 new tests added (389 total), all passing:
    - `save_to_path_writes_file_and_updates_workspace`
    - `load_from_path_overwrites_registries`
    - `dispatch_confirm_no_executes_pending_action`
    - `dispatch_confirm_cancel_does_nothing`
- Added `OntologyPlugin` to test_app for ontology registry availability
- All checks pass: `mise check` clean, zero clippy errors, no boundary violations

### Scope 3: Export Plugin Async Migration

- Migrated `src/export/systems.rs` from blocking `rfd::FileDialog::pick_folder()` to async
  `rfd::AsyncFileDialog`
- Self-contained pattern: `PendingExport` resource holds `ExportData` + `Task<Option<PathBuf>>`
  (avoids cross-plugin import of persistence's `AsyncDialogTask`)
- Converted `handle_export_command` observer to thin dispatcher using `commands.queue()` — collects
  ECS data, spawns async dialog, inserts `PendingExport` resource
- Added `poll_pending_export` exclusive system — zero-cost polling via `block_on(poll_once(...))`
- Extracted `run_export` helper — runs all exporters and triggers toast events
- Removed keyboard reset workaround (`clear_keyboard_after_dialog` was already dead code after
  Scope 2)
- Registered `poll_pending_export` in `ExportPlugin::build`
- 3 new tests added (392 total), all passing:
    - `poll_noop_when_no_pending_export`
    - `poll_removes_resource_and_writes_files_on_completion`
    - `poll_removes_resource_when_user_cancels`
- Note: `PickFolder`, `FolderPicked`, `spawn_folder_dialog` in persistence's `async_dialog.rs` are
  now unused (export uses its own pattern) — dead code warnings remain, cleanup deferred
- All checks pass: `mise check` clean, 392 tests, no boundary violations
