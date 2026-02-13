# Feature: ontology

## Summary

Manages the game ontology framework: concepts, relations, and constraints. Provides the data
registries that store designer-defined abstractions. Auto-generates companion constraints when
relations are created. Runs schema validation to check the game system definition for internal
consistency.

## Plugin

- Module: `src/ontology/`
- Plugin struct: `OntologyPlugin`
- Schedule: Startup (registry initialization), Update (auto-generation, schema validation on change)

## Dependencies

- **Contracts consumed**: game_system (EntityTypeRegistry, EntityType, EntityRole, TypeId,
  PropertyDefinition, PropertyValue), ontology (Concept, ConceptRole, ConceptBinding,
  PropertyBinding, Relation, RelationTrigger, RelationEffect, ModifyOperation, Constraint,
  ConstraintExpr, CompareOp, ConceptRegistry, RelationRegistry, ConstraintRegistry)
- **Contracts produced**: ontology (ConceptRegistry, RelationRegistry, ConstraintRegistry)
- **Crate dependencies**: none new

## Requirements

1. [REQ-1] Inserts empty ConceptRegistry, RelationRegistry, and ConstraintRegistry at Startup
2. [REQ-2] When a Relation with `ModifyProperty { operation: Subtract }` is created, auto-generates
   a companion constraint that checks the target property is >= 0 (non-negative budget). The
   constraint is marked `auto_generated = true` and references the relation ID.
3. [REQ-3] When an auto-generated constraint's source relation is deleted, the constraint is also
   deleted
4. [REQ-4] When an auto-generated constraint's source relation is modified, the constraint is
   regenerated
5. [REQ-5] Auto-generated constraints can be modified or deleted by the designer (they are not
   locked)
6. [REQ-6] Registries are Bevy Resources, mutated by the editor_ui through deferred actions and
   visible to rules_engine through `Res<T>` reads
7. [REQ-7] Schema validation runs when any ontology registry changes (ConceptRegistry,
   RelationRegistry, ConstraintRegistry) or when EntityTypeRegistry changes. Uses Bevy change
   detection.
8. [REQ-8] Schema validation checks:
    - ConceptBinding references valid entity type, concept, and concept role
    - Entity type's EntityRole matches ConceptRole's allowed_entity_roles
    - PropertyBinding references valid property on the bound entity type
    - Relation references valid concept and distinct roles within that concept
    - Constraint expression references valid roles and concept-local property names
    - Concept roles have at least one binding (warning, not error)

## Success Criteria

- [x] [SC-1] `registries_available_at_startup` test — ConceptRegistry, RelationRegistry,
      ConstraintRegistry exist as resources after Startup
- [x] [SC-2] `auto_constraint_on_subtract_relation` test — creating a Subtract relation
      auto-generates a non-negative budget constraint
- [x] [SC-3] `auto_constraint_deleted_with_relation` test — deleting the source relation removes the
      auto-generated constraint
- [x] [SC-4] `auto_constraint_regenerated_on_relation_change` test — modifying the relation
      regenerates the constraint
- [x] [SC-5] `schema_validation_catches_dangling_reference` test — binding referencing a
      non-existent entity type produces a SchemaError
- [x] [SC-6] `schema_validation_catches_role_mismatch` test — Token type bound to a
      BoardPosition-only concept role produces a SchemaError
- [x] [SC-7] `schema_validation_catches_property_mismatch` test — property binding referencing a
      non-existent property produces a SchemaError
- [x] [SC-BUILD] `cargo build` succeeds with this plugin registered
- [x] [SC-CLIPPY] `cargo clippy --all-targets` passes
- [x] [SC-TEST] `cargo test` passes
- [x] [SC-BOUNDARY] No imports from other features' internals — all cross-feature types come from
      `crate::contracts::`

## Constraints

- The ontology plugin does NOT evaluate constraints against board state — that is the rules_engine's
  responsibility
- Schema validation produces a SchemaValidation resource but does not prevent the designer from
  saving invalid configurations (it's advisory)
- Auto-generated constraints use a "[auto]" badge in the UI (editor_ui responsibility) but the
  `auto_generated` flag is maintained by this plugin
- The ontology plugin must not depend on hex_grid, cell, unit, or editor_ui contracts

## Open Questions

- None
