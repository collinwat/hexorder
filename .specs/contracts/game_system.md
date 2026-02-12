# Contract: game_system

## Purpose

Defines the Game System container, the entity-agnostic property system, and the unified entity type
system. The Game System is the root design artifact that holds all user-defined definitions.

M4 unifies CellType and UnitType into a single EntityType with a designer-assigned role. This
eliminates code duplication and enables the ontology framework to work across all entity categories.

## Types

### Identity

```rust
/// Unique identifier for entity types, enum definitions, property definitions,
/// concepts, relations, constraints, etc.
/// Uses UUID for stability across serialization (M5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub uuid::Uuid);
```

### Game System Container

```rust
/// The root design artifact. Holds all definitions for a game system.
#[derive(Resource, Debug)]
pub struct GameSystem {
    /// Unique identifier for this game system.
    pub id: String,
    /// Version string (e.g., "0.1.0").
    pub version: String,
}
```

### Property Types

```rust
/// The data type of a property definition. Extensible for future milestones.
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyType {
    Bool,
    Int,
    Float,
    String,
    Color,
    Enum(TypeId),  // references an EnumDefinition
}

/// A concrete value for a property instance.
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Color(bevy::color::Color),
    Enum(String),  // the selected option name
}

/// A property schema entry: name, type, and default value.
#[derive(Debug, Clone)]
pub struct PropertyDefinition {
    pub id: TypeId,
    pub name: String,
    pub property_type: PropertyType,
    pub default_value: PropertyValue,
}

/// A named set of string options for Enum-type properties.
#[derive(Debug, Clone)]
pub struct EnumDefinition {
    pub id: TypeId,
    pub name: String,
    pub options: Vec<String>,
}
```

### Entity Types (M4 — replaces Cell Types and Unit Types)

```rust
/// The role an entity type plays in the game system.
/// Determines how instances interact with the grid and other entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityRole {
    /// Occupies a hex position on the board (replaces CellType).
    /// Each hex tile has exactly one BoardPosition entity type.
    BoardPosition,
    /// A movable game piece placed on hex tiles (replaces UnitType).
    /// Multiple tokens may occupy the same hex position.
    Token,
}

/// A unified entity type definition. Replaces both CellType and UnitType.
/// The designer classifies each type by role.
#[derive(Debug, Clone)]
pub struct EntityType {
    pub id: TypeId,
    pub name: String,
    pub role: EntityRole,
    pub color: bevy::color::Color,
    pub properties: Vec<PropertyDefinition>,
}

/// Unified registry of all entity types and enum definitions.
/// Replaces CellTypeRegistry and UnitTypeRegistry.
#[derive(Resource, Debug, Default)]
pub struct EntityTypeRegistry {
    pub types: Vec<EntityType>,
    pub enum_definitions: Vec<EnumDefinition>,
}

/// Component attached to any entity on the hex grid (tiles and tokens).
/// Stores the entity type and per-instance property values.
/// Replaces both CellData and UnitData.
#[derive(Component, Debug, Clone)]
pub struct EntityData {
    pub entity_type_id: TypeId,
    /// Per-instance property values, keyed by PropertyDefinition ID.
    pub properties: HashMap<TypeId, PropertyValue>,
}

/// Marker component for token entities on the hex grid.
/// Used to distinguish tokens from tiles in queries.
#[derive(Component, Debug)]
pub struct UnitInstance;

/// Tracks which BoardPosition entity type the user is currently painting with.
#[derive(Resource, Debug, Default)]
pub struct ActiveBoardType {
    pub entity_type_id: Option<TypeId>,
}

/// Tracks which Token entity type the user is currently placing.
#[derive(Resource, Debug, Default)]
pub struct ActiveTokenType {
    pub entity_type_id: Option<TypeId>,
}

/// Tracks the currently selected unit entity, if any.
#[derive(Resource, Debug, Default)]
pub struct SelectedUnit {
    pub entity: Option<Entity>,
}

/// Fired when a token entity is placed on the grid.
#[derive(Event, Debug)]
pub struct UnitPlacedEvent {
    pub entity: Entity,
    pub position: HexPosition,
    pub entity_type_id: TypeId,
}
```

### Removed Types (M4)

The following M3 types are removed in M4, replaced by the unified EntityType system:

| Removed Type       | Replaced By          | Notes                                  |
| ------------------ | -------------------- | -------------------------------------- |
| `CellType`         | `EntityType`         | role = BoardPosition                   |
| `CellTypeId`       | `TypeId`             | No longer a separate alias             |
| `CellTypeRegistry` | `EntityTypeRegistry` | Filter by role for role-specific views |
| `CellData`         | `EntityData`         | Same structure, unified name           |
| `ActiveCellType`   | `ActiveBoardType`    | Role-specific active selection         |
| `UnitType`         | `EntityType`         | role = Token                           |
| `UnitTypeId`       | `TypeId`             | No longer a separate alias             |
| `UnitTypeRegistry` | `EntityTypeRegistry` | Filter by role for role-specific views |
| `UnitData`         | `EntityData`         | Same structure, unified name           |
| `ActiveUnitType`   | `ActiveTokenType`    | Role-specific active selection         |

## Consumers

- game_system (owns the GameSystem resource, EntityTypeRegistry, startup logic)
- cell (reads EntityTypeRegistry filtered by BoardPosition, EntityData)
- unit (reads EntityTypeRegistry filtered by Token, EntityData, SelectedUnit)
- ontology (reads EntityTypeRegistry for concept bindings and schema validation)
- rules_engine (reads EntityTypeRegistry for constraint evaluation)
- editor_ui (reads/writes GameSystem, EntityTypeRegistry, ActiveBoardType, ActiveTokenType,
  SelectedUnit, PropertyDefinition, PropertyValue)

## Producers

- game_system (inserts GameSystem, EntityTypeRegistry, ActiveBoardType, ActiveTokenType,
  SelectedUnit resources at startup)

## Invariants

- `GameSystem` is inserted during `Startup` and available for the lifetime of the app
- `EntityTypeRegistry` is inserted during `Startup`; may be empty or contain starter types
- `ActiveBoardType` is inserted during `Startup`; defaults to the first BoardPosition type
- `ActiveTokenType` is inserted during `Startup`; defaults to the first Token type
- `SelectedUnit` is inserted during `Startup`; defaults to None
- `EntityData.entity_type_id` must reference a valid entry in `EntityTypeRegistry`
- `PropertyValue` variant must match the corresponding `PropertyType` variant
- `PropertyValue::Enum` value must be one of the options in the referenced `EnumDefinition`
- `TypeId` values are globally unique (UUID-based)
- `EntityTypeRegistry` contains all entity types regardless of role; use `types_by_role()` for
  filtered views

## Changelog

| Date       | Change                           | Reason                                                                                                      |
| ---------- | -------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| 2026-02-08 | Initial definition               | M2 Game System container and property system                                                                |
| 2026-02-09 | Renamed Vertex->Cell terminology | Cell is mathematically correct for N-dimensional grid elements; Vertex means hex corner in grid terminology |
| 2026-02-09 | Added unit types section         | M3 — units on the hex grid                                                                                  |
| 2026-02-11 | Unified EntityType               | M4 — replace CellType/UnitType with EntityType + EntityRole                                                 |
