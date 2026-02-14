# Plugin: game_system

## Summary

Provides the Game System container and the entity-agnostic property system. The Game System is the
root design artifact — it holds all definitions. The property system defines schema types that any
entity can use.

0.4.0 unifies CellType and UnitType into a single EntityType with a designer-assigned role
(BoardPosition or Token). The plugin produces a single EntityTypeRegistry.

## Plugin

- Module: `src/game_system/`
- Plugin struct: `GameSystemPlugin`
- Schedule: `Startup` (initialize GameSystem resource, EntityTypeRegistry, starter entity types)

## Dependencies

- **Contracts consumed**: none
- **Contracts produced**: `game_system` (GameSystem, TypeId, PropertyType, PropertyDefinition,
  PropertyValue, EnumDefinition, EntityType, EntityRole, EntityTypeRegistry, EntityData,
  ActiveBoardType, ActiveTokenType, UnitInstance, SelectedUnit, UnitPlacedEvent)
- **Crate dependencies**: `uuid` (for generating type IDs)

## Requirements

### 0.2.0 (retained, evolved for 0.4.0)

1. [REQ-CONTAINER] Insert a `GameSystem` resource at startup with a generated `id` (UUID) and
   `version` string ("0.1.0" default).
2. [REQ-PROP-TYPES] Define a `PropertyType` enum with 6 variants: Bool, Int, Float, String, Color,
   Enum(TypeId).
3. [REQ-PROP-DEF] Define `PropertyDefinition` — a schema entry with id, name, property_type, and
   default_value.
4. [REQ-PROP-VALUE] Define `PropertyValue` — a concrete value matching a PropertyType.
5. [REQ-ENUM-DEF] Define `EnumDefinition` — a named set of string options.

### 0.4.0 (new — EntityType unification)

6. [REQ-ENTITY-ROLE] Define `EntityRole` enum: BoardPosition, Token.
7. [REQ-ENTITY-TYPE] Define `EntityType` — a unified type definition with id, name, role, color, and
   properties. Replaces CellType and UnitType.
8. [REQ-ENTITY-REGISTRY] Insert an `EntityTypeRegistry` resource at startup. Provides methods:
   `get(id)`, `get_enum(id)`, `types_by_role(role)`, `first_by_role(role)`, iteration.
9. [REQ-ENTITY-DATA] Define `EntityData` component — attached to hex tile and unit entities. Holds
   entity_type_id and per-instance property values. Replaces CellData and UnitData.
10. [REQ-ACTIVE-TYPES] Insert `ActiveBoardType` and `ActiveTokenType` resources at startup,
    defaulting to the first entity type of each role.
11. [REQ-STARTERS] On startup, create starter entity types:
    - BoardPosition role: Plains, Forest, Water, Mountain, Road (5 types, with colors)
    - Token role: Infantry, Cavalry, Artillery (3 types, with colors)
    - Starter types should include properties that demonstrate the ontology. E.g., Infantry gets a
      "Movement Points" (Int, default 4) property; Mountain gets a "Movement Cost" (Int, default 3)
      property.

### Removed (0.4.0)

- CellType, CellTypeId, CellTypeRegistry, CellData, ActiveCellType — replaced by EntityType system
- UnitType, UnitTypeId, UnitTypeRegistry, UnitData, ActiveUnitType — replaced by EntityType system

## Success Criteria

### 0.2.0 (retained)

- [x] [SC-1] GameSystem resource exists after Startup with a non-empty id and version
- [x] [SC-2] All 6 PropertyType variants can be constructed and are distinct
- [x] [SC-3] PropertyDefinition can be created for each PropertyType with a default value
- [x] [SC-4] PropertyValue round-trips correctly for each type
- [x] [SC-5] EnumDefinition can hold options and be referenced by PropertyType::Enum

### 0.4.0 (new)

- [ ] [SC-6] EntityTypeRegistry exists after Startup with starter types
- [ ] [SC-7] `types_by_role(BoardPosition)` returns exactly the 5 board types
- [ ] [SC-8] `types_by_role(Token)` returns exactly the 3 token types
- [ ] [SC-9] ActiveBoardType defaults to the first BoardPosition type
- [ ] [SC-10] ActiveTokenType defaults to the first Token type
- [ ] [SC-11] EntityData component can store an entity_type_id and property values
- [ ] [SC-12] Starter Infantry type has "Movement Points" property (Int, default 4)
- [ ] [SC-BUILD] `cargo build` succeeds with this plugin registered
- [ ] [SC-CLIPPY] `cargo clippy --all-targets` passes
- [ ] [SC-TEST] `cargo test` passes
- [ ] [SC-BOUNDARY] No imports from other features' internals

## Constraints

- PropertyType must be extensible — future releases will add EntityRef, List, Map, Struct, etc.
- TypeId uses UUID for stability across future serialization (0.6.0)
- The property system is entity-agnostic — not coupled to any specific EntityRole
- GameSystem is a singleton (one game system per app session)
- EntityTypeRegistry contains ALL types; use types_by_role() for filtered views

## Open Questions

- None (EntityType unification design decided during 0.4.0 planning)
