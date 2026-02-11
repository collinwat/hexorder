# Feature: game_system

## Summary

Provides the Game System container and the entity-agnostic property system. The Game System is the
root design artifact — it holds all definitions (cell types, and later unit types, rules, etc.). The
property system defines schema types that any entity can use.

## Plugin

- Module: `src/game_system/`
- Plugin struct: `GameSystemPlugin`
- Schedule: `Startup` (initialize GameSystem resource, default cell types)

## Dependencies

- **Contracts consumed**: none
- **Contracts produced**: `game_system` (GameSystem, PropertyType, PropertyDefinition,
  PropertyValue, EnumDefinition), `cell` (CellTypeId, CellType, CellTypeRegistry, CellData,
  ActiveCellType)
- **Crate dependencies**: `uuid` (for generating cell type IDs)

## Requirements

1. [REQ-CONTAINER] Insert a `GameSystem` resource at startup with a generated `id` (UUID) and
   `version` string ("0.1.0" default). The Game System is the root container for all design data.
2. [REQ-PROP-TYPES] Define a `PropertyType` enum with 6 variants: `Bool`, `Int`, `Float`, `String`,
   `Color`, `Enum(EnumDefinitionId)`. These are the primitive data types for M2.
3. [REQ-PROP-DEF] Define `PropertyDefinition` — a schema entry with `id`, `name`, `property_type`,
   and `default_value`. Property definitions are reusable across entity types.
4. [REQ-PROP-VALUE] Define `PropertyValue` — a concrete value matching a `PropertyType`. Used for
   per-instance property storage.
5. [REQ-ENUM-DEF] Define `EnumDefinition` — a named set of string options (e.g., "Movement Mode"
   with options ["Foot", "Wheeled", "Tracked"]). Enum property types reference an EnumDefinition by
   ID.
6. [REQ-CELL-TYPE] Define `CellType` — a named template with `id`, `name`, `color`, and a list of
   `PropertyDefinition`s. This is what a Game System uses to describe a kind of board position.
7. [REQ-CELL-REGISTRY] Insert a `CellTypeRegistry` resource that holds all defined cell types.
   Provide methods for add, remove, get by ID, and iteration.
8. [REQ-CELL-DATA] Define `CellData` component — attached to hex tile entities. Holds a `CellTypeId`
   and a map of property values. Replaces the M1 `Terrain` component.
9. [REQ-ACTIVE-CELL] Insert an `ActiveCellType` resource that tracks which cell type the user is
   currently painting with.
10. [REQ-DEFAULTS] On startup, if the registry is empty, create a set of starter cell types (e.g.,
    "Plains", "Forest", "Water") with a color and no custom properties. This gives the user
    something to start with.

## Success Criteria

- [x] [SC-1] GameSystem resource exists after Startup with a non-empty id and version
- [x] [SC-2] All 6 PropertyType variants can be constructed and are distinct
- [x] [SC-3] PropertyDefinition can be created for each PropertyType with a default value
- [x] [SC-4] PropertyValue round-trips correctly for each type
- [x] [SC-5] EnumDefinition can hold options and be referenced by PropertyType::Enum
- [x] [SC-6] CellType can be created with properties and a color
- [x] [SC-7] CellTypeRegistry supports add, remove, get, and iterate
- [x] [SC-8] CellData component can store a cell type ID and property values
- [x] [SC-9] ActiveCellType resource defaults to the first registered cell type
- [x] [SC-10] Starter cell types are created if registry is empty at startup
- [x] [SC-BUILD] `cargo build` succeeds with this plugin registered
- [x] [SC-CLIPPY] `cargo clippy -- -D warnings` passes
- [x] [SC-TEST] `cargo test` passes
- [x] [SC-BOUNDARY] No imports from other features' internals

## Decomposition

Solo feature — no parallel decomposition needed. The contract types should be defined first, then
the plugin systems.

## Constraints

- PropertyType must be extensible — M3+ will add EntityRef, List, Map, Struct, Formula, etc.
- CellTypeId should be a stable identifier (UUID or similar) that survives serialization (M5)
- The property system is entity-agnostic — do NOT couple it to cells specifically. Unit types in M3
  will reuse the same PropertyDefinition and PropertyValue types.
- GameSystem is a singleton for M2 (one game system per app session). Multi-system support is future
  scope.

## Open Questions

- Should PropertyDefinition carry validation hints (min/max for Int/Float)? (Suggest: defer to a
  follow-up, keep M2 simple)
- Should cell types support inheritance/prototypes? (Suggest: defer, VASSAL-style prototypes are
  useful but add complexity)
