# Plugin: unit

## Summary

Allows users to place unit tokens (Token role entity types) on the hex grid, select them, move them
between tiles, and delete them.

M4: Migrates from UnitTypeRegistry/UnitData to EntityTypeRegistry/EntityData. Movement now consults
ValidMoveSet from the rules_engine — if constraints exist and the destination is not in the valid
set, the move is rejected.

## Plugin

- Module: `src/unit/`
- Plugin struct: `UnitPlugin`
- Schedule: Startup (material/mesh setup), Update (sync chain, deletion), Observer (placement,
  interaction)

## Dependencies

- **Contracts consumed**: game_system (EntityTypeRegistry, EntityType, EntityRole, EntityData,
  UnitInstance, ActiveTokenType, SelectedUnit, UnitPlacedEvent, TypeId, PropertyValue), hex_grid
  (HexPosition, HexGridConfig, HexSelectedEvent, HexMoveEvent), editor_ui (EditorTool), validation
  (ValidMoveSet)
- **Contracts produced**: none
- **Crate dependencies**: none new

## Requirements

### M3 (retained, evolved for M4)

1. [REQ-1] Setup creates per-type materials and a shared cylinder mesh at Startup. Reads
   EntityTypeRegistry filtered by Token role.
2. [REQ-2] In Place mode, clicking a hex tile spawns a unit entity with UnitInstance, HexPosition,
   EntityData, mesh, material, and Transform
3. [REQ-3] Placement verifies the clicked position is within grid bounds (map_radius)
4. [REQ-4] Placement fires a UnitPlacedEvent with entity, position, and entity_type_id
5. [REQ-5] In Select mode, clicking a hex with a unit selects it (sets SelectedUnit)
6. [REQ-6] In Select mode, clicking a different hex while a unit is selected attempts to move it
7. [REQ-7] Movement updates HexPosition, Transform, fires HexMoveEvent, and deselects the unit
8. [REQ-8] Movement respects grid bounds (cannot move off-grid)
9. [REQ-9] Unit deletion despawns the selected entity when triggered by the editor UI
10. [REQ-10] Material sync reacts to EntityTypeRegistry changes (change detection, Token filter)
11. [REQ-11] Visual sync reacts to EntityData changes (change detection)
12. [REQ-12] Unit tokens render as colored cylinders at Y=0.25 above the hex tile

### M4 (new — constraint-aware movement)

13. [REQ-13] Before executing a move, check if the destination is in `ValidMoveSet.valid_positions`.
    If it is, proceed with the move. If not, reject the move (do not update position or fire
    HexMoveEvent).
14. [REQ-14] If no constraints exist (empty ontology), all positions within grid bounds are valid —
    preserving M3 free-movement behavior.
15. [REQ-15] When a move is rejected, the unit remains selected (not deselected).

## Success Criteria

### M3 (retained)

- [x] [SC-1] `unit_materials_created_for_all_types` test
- [x] [SC-2] `unit_mesh_resource_exists` test
- [x] [SC-3] `place_unit_creates_entity` test
- [x] [SC-4] `place_unit_skipped_in_select_mode` test
- [x] [SC-5] `select_unit_sets_selected` test
- [x] [SC-6] `move_unit_updates_position` test
- [x] [SC-7] `move_unit_respects_grid_bounds` test
- [x] [SC-8] `sync_unit_visuals_updates_material` test
- [x] [SC-9] `sync_unit_materials_adds_new_type` test

### M4 (new)

- [ ] [SC-10] `move_rejected_when_blocked` test — unit does not move to a position not in
      ValidMoveSet
- [ ] [SC-11] `move_allowed_when_valid` test — unit moves to a position in ValidMoveSet
- [ ] [SC-12] `free_movement_when_no_constraints` test — all grid positions are valid when ontology
      is empty
- [ ] [SC-13] `unit_stays_selected_on_rejection` test — SelectedUnit is not cleared when move fails
- [ ] [SC-BUILD] `cargo build` succeeds with this plugin registered
- [ ] [SC-CLIPPY] `cargo clippy --all-targets` passes
- [ ] [SC-TEST] `cargo test` passes
- [ ] [SC-BOUNDARY] No imports from other features' internals

## Constraints

- Unit entities share HexPosition with hex tiles but are separate entities (not children of tiles)
- Multiple units can occupy the same hex (no stacking rules in M4)
- Unit selection is position-based (check HexPosition match), not raycast-based
- Clicking the same hex as the selected unit deselects it
- When ValidMoveSet.for_entity doesn't match the selected unit, treat all positions as valid
  (graceful degradation)

## Open Questions

- None
