# Plugin Log: rules_engine

## 2026-02-16 — 0.9.0 Core Mechanics kickoff (#77)

**Scope**: Turn structure definition, CRT editor, combat execution, modifier system. Big Batch pitch
touching rules_engine, game_system, unit, and editor_ui. New `mechanics` contract.

**Research consumed**: Hex Wargame Mechanics Survey (wiki) — Section 1.3 (Combat Resolution Systems)
covers 5 CRT types, common result codes, modifier categories. Section 1.4 (Turn Structure) covers
IGOUGO, alternating activation, chit-pull, impulse, simultaneous. The survey validates the pitch's
scoping: phase-based turns only (IGOUGO/simultaneous), card-driven/chit-pull deferred.

**Design decisions from pitch Q&A** (see `docs/plans/2026-02-15-core-mechanics-design.md`):

- Q1: Resource-only turn structure (not Bevy States) — designer-defined phases can't be compile-time
- Q2: `AppScreen::Play` added — toggle between Editor and Play via toolbar
- Q3: Per-column CRT type (mixed ratio/differential in one CRT)
- Q4: Fully custom rows (no auto-generation from dice config)
- Q5: Structured outcomes with labels (partial automation: highlight + confirm)
- Q6: Priority-ordered modifiers with optional caps
- Q7: Concept binding for combat strength (ontology integration)

**First piece**: CRT data model + resolution logic + test harness. Most novel, self-contained,
testable without UI. Surfaces column lookup edge cases, threshold ordering, mixed column types.

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
