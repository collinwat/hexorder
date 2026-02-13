# Feature Log: ontology

## 2026-02-12 — Implementation complete

- Implemented `OntologyPlugin` in `src/ontology/` with two chained systems:
    - `auto_generate_constraints`: uses Bevy change detection on `RelationRegistry` and
      `ConstraintRegistry`; creates/removes/regenerates companion constraints for Subtract relations
    - `run_schema_validation`: uses change detection on all four registries; validates bindings,
      roles, properties, relations, constraints, and missing bindings
- Plugin initializes `ConceptRegistry`, `RelationRegistry`, `ConstraintRegistry`, and
  `SchemaValidation` as resources at build time (immediately available, no deferred startup)
- Registered plugin in `main.rs` after `GameSystemPlugin`, before `CellPlugin` (per coordination.md
  load order)
- Fixed clippy doc_markdown issue in `src/contracts/ontology.rs` (`movement_points` needed
  backticks)
- All 7 ontology tests pass, 78 total tests pass, clippy clean, no boundary violations
- Decisions:
    - `SchemaValidation` is initialized by `OntologyPlugin` (not `rules_engine`) because the
      ontology plugin produces schema validation results. `rules_engine` will read it or produce
      state-level validation separately.
    - Auto-generated constraints use `TypeId::new()` for fresh UUIDs, so regeneration produces new
      IDs (no stable ID preservation across regeneration). This is acceptable since auto-generated
      constraints are ephemeral and not referenced by other definitions.
    - When a constraint's `auto_generated` flag is `false` (designer modified it), the retain logic
      preserves it even if the source relation is deleted. This satisfies REQ-5.

## 2026-02-11 — Initial spec

- Created feature spec for M4
- Ontology manages concepts, relations, and constraints as Bevy resources
- Auto-generates companion constraints for Subtract relations
- Schema validation checks internal consistency of the game system definition
