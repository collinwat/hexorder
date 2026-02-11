# Feature Log: game_system

## Status: complete

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-02-09 | Contract types already existed in `src/contracts/game_system.rs` -- no contract changes needed | Types were defined ahead of consumers as part of spec process |
| 2026-02-09 | Three chained Startup systems: setup_game_system, setup_cell_type_registry, setup_active_cell_type | Chain ensures registry is available when active cell type reads it |
| 2026-02-09 | Moved resource creation from Startup systems to `build()` using `app.insert_resource()` | Deferred `commands.insert_resource()` in Startup caused cross-plugin ordering crash — CellPlugin's Startup expected resources before they were flushed. Factory functions replace systems. |
| 2026-02-09 | Active cell type defaults to first registry entry using `registry.first().map(...)` | Avoids unwrap; returns None if registry happens to be empty |
| 2026-02-09 | Plugin placed before cell/editor_ui in main.rs load order | Required by coordination.md -- cell and editor_ui plugins depend on game_system resources |

## Test Results

| Date | Command | Result | Notes |
|------|---------|--------|-------|
| 2026-02-09 | `cargo build` | PASS | Clean compilation with plugin registered |
| 2026-02-09 | `cargo clippy -- -D warnings` | PASS | Zero warnings |
| 2026-02-09 | `cargo test` | PASS | 54/54 tests pass (10 new game_system tests) |

### Tests implemented (10):
1. `game_system_resource_exists_after_startup` -- GameSystem has non-empty id and version "0.1.0"
2. `registry_has_starter_types` -- CellTypeRegistry has 5 types
3. `starter_types_have_names` -- all starter types have non-empty names
4. `starter_types_have_distinct_ids` -- all IDs are unique
5. `active_cell_type_defaults_to_first` -- ActiveCellType references first type
6. `property_type_variants_are_distinct` -- all 6 PropertyType variants distinguishable
7. `property_value_default_for_each_type` -- PropertyValue::default_for returns correct variant
8. `type_id_generates_unique` -- two TypeId::new() calls produce different values
9. `registry_get_by_id` -- CellTypeRegistry::get() finds a type by id
10. `registry_get_enum_by_id` -- CellTypeRegistry::get_enum() works

## Blockers

| Blocker | Waiting On | Raised | Resolved |
|---------|-----------|--------|----------|
| (none) | | | |

## Status Updates

| Date | Status | Notes |
|------|--------|-------|
| 2026-02-08 | speccing | Initial spec created for M2 |
| 2026-02-09 | complete | Plugin implemented, all tests pass, clippy clean |
