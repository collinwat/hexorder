# Feature: rules_engine

## Summary

Evaluates the ontology's constraints against the current board state. Computes which hex positions a
selected unit can reach (ValidMoveSet) using BFS with constraint evaluation at each step. Produces
human-readable explanations for constraint violations. Also runs schema validation on the game
system definition.

## Plugin

- Module: `src/rules_engine/`
- Plugin struct: `RulesEnginePlugin`
- Schedule: Startup (resource initialization), Update (schema validation, valid move computation)

## Dependencies

- **Contracts consumed**: game_system (EntityTypeRegistry, EntityType, EntityData, EntityRole,
  TypeId, PropertyDefinition, PropertyValue, SelectedUnit, UnitInstance), ontology (ConceptRegistry,
  RelationRegistry, ConstraintRegistry, Concept, ConceptRole, ConceptBinding, Relation, Constraint,
  ConstraintExpr, RelationTrigger, RelationEffect, CompareOp), hex_grid (HexPosition, HexGridConfig,
  HexTile), validation (SchemaValidation, SchemaError, SchemaErrorCategory, ValidMoveSet,
  ValidationResult)
- **Contracts produced**: validation (SchemaValidation, ValidMoveSet)
- **Crate dependencies**: none new (hexx already available via hex_grid contract)

## Requirements

1. [REQ-1] Inserts SchemaValidation and ValidMoveSet resources at Startup (both default/empty)
2. [REQ-2] Schema validation system runs when ontology registries or EntityTypeRegistry change.
   Produces SchemaValidation resource with all detected errors.
3. [REQ-3] Schema validation checks all invariants listed in the ontology contract:
    - Dangling references (concept bindings, relations, constraints)
    - Role mismatches (entity role vs concept role's allowed_entity_roles)
    - Property mismatches (property bindings referencing non-existent properties)
    - Missing bindings (concept roles with no bound entity types — warning level)
    - Invalid expressions (constraint expressions referencing invalid roles or properties)
4. [REQ-4] Valid move computation runs when SelectedUnit changes, when EntityData changes on tiles,
   or when ontology registries change
5. [REQ-5] When a unit is selected, computes reachable positions via BFS:
    - Start from the unit's current HexPosition
    - At each neighbor, evaluate all applicable constraints (relations with OnEnter trigger)
    - PathBudget constraints accumulate cost along the path
    - BFS depth is limited to the unit's maximum budget value (performance guard)
    - Only positions within grid bounds (map_radius) are considered
6. [REQ-6] ValidMoveSet.valid_positions contains all reachable hex positions
7. [REQ-7] ValidMoveSet.blocked_explanations contains human-readable reasons for each blocked
   position within range
8. [REQ-8] When no unit is selected, ValidMoveSet is empty (for_entity = None)
9. [REQ-9] Human-readable explanations follow template patterns:
    - Block: "{entity_type} cannot enter {target_type}: {relation_name} blocks entry"
    - Budget exceeded: "{entity_type} cannot reach ({q}, {r}): path cost {cost} exceeds
      {budget_property} of {budget}"
    - Property violation: "{constraint_name}: {property_name} is {actual}, must be {op} {expected}"

## Success Criteria

- [ ] [SC-1] `schema_validation_resource_exists` test — SchemaValidation exists after Startup
- [ ] [SC-2] `valid_move_set_resource_exists` test — ValidMoveSet exists after Startup
- [ ] [SC-3] `valid_moves_empty_when_no_selection` test — ValidMoveSet is empty when no unit
      selected
- [ ] [SC-4] `valid_moves_computed_on_selection` test — selecting a unit populates ValidMoveSet
- [ ] [SC-5] `blocked_positions_have_explanations` test — blocked hexes have non-empty explanation
      strings
- [ ] [SC-6] `path_budget_limits_range` test — unit with budget N cannot reach positions costing
      more than N
- [ ] [SC-7] `block_relation_prevents_entry` test — Block relation on a concept role prevents
      movement to bound entity types
- [ ] [SC-8] `schema_errors_detected` test — invalid ontology produces non-empty SchemaValidation
      errors
- [ ] [SC-9] `valid_moves_respect_grid_bounds` test — positions outside map_radius are never in
      valid_positions
- [ ] [SC-BUILD] `cargo build` succeeds with this plugin registered
- [ ] [SC-CLIPPY] `cargo clippy --all-targets` passes
- [ ] [SC-TEST] `cargo test` passes
- [ ] [SC-BOUNDARY] No imports from other features' internals — all cross-feature types come from
      `crate::contracts::`

## Constraints

- The rules_engine does NOT modify ontology registries — it only reads them
- The rules_engine does NOT execute moves — it only computes validity. The unit plugin executes
  moves and checks ValidMoveSet before doing so.
- BFS must be bounded to prevent performance issues on large grids. Maximum depth = max budget value
  of the selected unit's concept bindings.
- If no constraints exist in the ontology, all positions within grid bounds are valid (free
  movement, same as M3 behavior)
- Schema validation is advisory — invalid schemas do not prevent the tool from functioning

## Open Questions

- None
