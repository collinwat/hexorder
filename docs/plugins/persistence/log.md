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
