# Feature: unit

## Summary
Allows users to define unit types (game entities) within the Game System, place unit tokens on the hex grid, select them, move them between tiles, and delete them. No rule enforcement — pure placement and relocation.

## Plugin
- Module: `src/unit/`
- Plugin struct: `UnitPlugin`
- Schedule: Startup (material/mesh setup), Update (sync chain, deletion), Observer (placement, interaction)

## Dependencies
- **Contracts consumed**: game_system (UnitTypeRegistry, UnitType, UnitData, UnitInstance, ActiveUnitType, SelectedUnit, UnitPlacedEvent, PropertyValue), hex_grid (HexPosition, HexGridConfig, HexSelectedEvent, HexMoveEvent), editor_ui (EditorTool)
- **Contracts produced**: none (all unit types are in game_system contract)
- **Crate dependencies**: none new

## Requirements
1. [REQ-1] Setup creates per-type materials and a shared cylinder mesh at Startup
2. [REQ-2] In Place mode, clicking a hex tile spawns a unit entity with UnitInstance, HexPosition, UnitData, mesh, material, and Transform
3. [REQ-3] Placement verifies the clicked position is within grid bounds (map_radius)
4. [REQ-4] Placement fires a UnitPlacedEvent with entity, position, and unit_type_id
5. [REQ-5] In Select mode, clicking a hex with a unit selects it (sets SelectedUnit)
6. [REQ-6] In Select mode, clicking a different hex while a unit is selected moves the unit there
7. [REQ-7] Movement updates HexPosition, Transform, fires HexMoveEvent, and deselects the unit
8. [REQ-8] Movement respects grid bounds (cannot move off-grid)
9. [REQ-9] Unit deletion despawns the selected entity when triggered by the editor UI
10. [REQ-10] Material sync reacts to UnitTypeRegistry changes (change detection)
11. [REQ-11] Visual sync reacts to UnitData changes (change detection)
12. [REQ-12] Unit tokens render as colored cylinders at Y=0.25 above the hex tile

## Success Criteria
- [x] [SC-1] `unit_materials_created_for_all_types` test — materials exist for each registered unit type after Startup
- [x] [SC-2] `unit_mesh_resource_exists` test — UnitMesh resource exists after Startup
- [x] [SC-3] `place_unit_creates_entity` test — spawns entity with correct components in Place mode
- [x] [SC-4] `place_unit_skipped_in_select_mode` test — no entity spawned when tool is Select
- [x] [SC-5] `select_unit_sets_selected` test — clicking unit hex sets SelectedUnit
- [x] [SC-6] `move_unit_updates_position` test — position and transform updated after move
- [x] [SC-7] `move_unit_respects_grid_bounds` test — off-grid moves are rejected
- [x] [SC-8] `sync_unit_visuals_updates_material` test — material changes when UnitData changes
- [x] [SC-9] `sync_unit_materials_adds_new_type` test — new type in registry gets a material
- [x] [SC-BUILD] `cargo build` succeeds with this plugin registered
- [x] [SC-CLIPPY] `cargo clippy -- -D warnings` passes
- [x] [SC-TEST] `cargo test` passes (71 tests, all pass)
- [x] [SC-BOUNDARY] No imports from other features' internals — all cross-feature types come from `crate::contracts::`

## Constraints
- Unit entities share HexPosition with hex tiles but are separate entities (not children of tiles)
- Multiple units can occupy the same hex (no stacking rules in M3)
- Unit selection is position-based (check HexPosition match), not raycast-based
- Clicking the same hex as the selected unit deselects it

## Open Questions
- None (all resolved during M3 planning)
