# Plugin Log: rules_engine

## 2026-02-11 — Initial spec

- Created feature spec for M4
- Rules engine evaluates constraints against board state
- Computes ValidMoveSet via BFS with constraint evaluation
- Produces SchemaValidation for the game system definition
- Key design: if no constraints exist, all moves are valid (backward compatible with M3)

## 2026-02-12 — Implementation complete

### Decisions

- **Schema validation not duplicated**: The OntologyPlugin already initializes SchemaValidation and
  runs schema validation (run_schema_validation system). The RulesEnginePlugin only initializes
  ValidMoveSet and runs compute_valid_moves. SC-1 and SC-8 are satisfied by OntologyPlugin.
- **BFS approach**: Used a VecDeque-based BFS with budget tracking. Each position tracks the best
  remaining budget seen. A position is only re-explored if a new path arrives with a strictly better
  budget, preventing infinite loops and unnecessary recomputation.
- **StepContext struct**: Introduced to bundle parameters for evaluate_step, avoiding clippy
  too_many_arguments warnings on helper functions while keeping the main system function annotated
  with allow.
- **Change detection**: System only recomputes when SelectedUnit, ConceptRegistry, RelationRegistry,
  or ConstraintRegistry changes. EntityData changes on tiles are not tracked for now (would require
  query-level change detection).
- **Free movement fallback**: When no OnEnter relations and no constraints exist, the system uses a
  simple BFS (no budget, no cost) that reaches all in-bounds positions. This preserves M3 behavior.
- **Block condition evaluation**: Supports IsType, IsNotType, All, Any, Not expressions. Other
  expression types conservatively default to blocked.
- **Budget determination**: Searches OnEnter Subtract relations to find the target_property
  concept-local name, then resolves it through concept bindings to the actual property value on the
  unit. Falls back to a "budget" named property, then to a generous default.
- **Grid bounds check**: Uses `max(|q|, |r|, |q+r|) <= map_radius` for axial coordinate bounds.
- **Neighbor computation**: Uses hexx::Hex::all_neighbors() which returns the 6 adjacent hex
  positions in axial coordinates.

### Test results

All 8 rules_engine tests pass:

- valid_move_set_resource_exists
- valid_moves_empty_when_no_selection
- valid_moves_computed_on_selection
- blocked_positions_have_explanations
- path_budget_limits_range
- block_relation_prevents_entry
- valid_moves_respect_grid_bounds
- free_movement_when_no_constraints

Full test suite: 86 tests pass (78 existing + 8 new). cargo clippy --all-targets -- -D warnings:
clean. cargo build: clean. cargo doc --no-deps --quiet: clean.
