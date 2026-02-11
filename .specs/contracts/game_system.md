# Contract: game_system

## Purpose
Defines the Game System container, the entity-agnostic property system, and the types needed to describe cell type definitions and unit type definitions. The Game System is the root design artifact that holds all user-defined definitions.

## Types

### Identity
```rust
/// Unique identifier for cell types, enum definitions, property definitions, etc.
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

### Cell Types
```rust
/// Unique identifier for a cell type.
pub type CellTypeId = TypeId;

/// A cell type definition: describes what a board position can be.
#[derive(Debug, Clone)]
pub struct CellType {
    pub id: CellTypeId,
    pub name: String,
    pub color: bevy::color::Color,
    pub properties: Vec<PropertyDefinition>,
}

/// Registry of all defined cell types in the current Game System.
#[derive(Resource, Debug)]
pub struct CellTypeRegistry {
    pub types: Vec<CellType>,
    pub enum_definitions: Vec<EnumDefinition>,
}

/// Component attached to hex tile entities. Stores the cell type
/// and per-instance property values.
#[derive(Component, Debug, Clone)]
pub struct CellData {
    pub cell_type_id: CellTypeId,
    pub properties: HashMap<TypeId, PropertyValue>,
}

/// Tracks which cell type the user is currently painting with.
#[derive(Resource, Debug)]
pub struct ActiveCellType {
    pub cell_type_id: Option<CellTypeId>,
}
```

### Unit Types
```rust
/// Unique identifier for a unit type.
pub type UnitTypeId = TypeId;

/// A unit type definition: describes what a game entity on the board can be.
/// Defined by the user within a Game System.
#[derive(Debug, Clone)]
pub struct UnitType {
    pub id: UnitTypeId,
    pub name: String,
    pub color: bevy::color::Color,
    pub properties: Vec<PropertyDefinition>,
}

/// Registry of all defined unit types in the current Game System.
#[derive(Resource, Debug, Default)]
pub struct UnitTypeRegistry {
    pub types: Vec<UnitType>,
    pub enum_definitions: Vec<EnumDefinition>,
}

/// Component attached to entities representing units on the hex grid.
/// Stores which unit type this entity is and its per-instance property values.
#[derive(Component, Debug, Clone)]
pub struct UnitData {
    pub unit_type_id: UnitTypeId,
    pub properties: HashMap<TypeId, PropertyValue>,
}

/// Marker component for unit entities on the hex grid.
#[derive(Component, Debug)]
pub struct UnitInstance;

/// Tracks which unit type the user is currently placing.
#[derive(Resource, Debug, Default)]
pub struct ActiveUnitType {
    pub unit_type_id: Option<UnitTypeId>,
}

/// Tracks the currently selected unit entity, if any.
#[derive(Resource, Debug, Default)]
pub struct SelectedUnit {
    pub entity: Option<Entity>,
}

/// Fired when a unit is placed on the grid.
#[derive(Event, Debug)]
pub struct UnitPlacedEvent {
    pub entity: Entity,
    pub position: HexPosition,
    pub unit_type_id: UnitTypeId,
}
```

## Consumers
- game_system (owns the GameSystem resource, cell type registry, unit type registry, startup logic)
- cell (reads CellTypeRegistry, CellType, CellData, ActiveCellType)
- unit (reads UnitTypeRegistry, UnitType, UnitData, ActiveUnitType, SelectedUnit)
- editor_ui (reads/writes GameSystem, CellTypeRegistry, UnitTypeRegistry, ActiveCellType, ActiveUnitType, SelectedUnit, PropertyDefinition, PropertyValue)

## Producers
- game_system (inserts GameSystem, CellTypeRegistry, ActiveCellType, UnitTypeRegistry, ActiveUnitType, SelectedUnit resources at startup)

## Invariants
- `GameSystem` is inserted during `Startup` and available for the lifetime of the app
- `CellTypeRegistry` is inserted during `Startup`; may be empty or contain starter types
- `ActiveCellType` is inserted during `Startup`; defaults to the first registered cell type (or None if empty)
- `CellData.cell_type_id` must reference a valid entry in `CellTypeRegistry` (or be handled gracefully if the type was deleted)
- `UnitTypeRegistry` is inserted during `Startup`; may be empty or contain starter types
- `ActiveUnitType` is inserted during `Startup`; defaults to the first registered unit type (or None if empty)
- `SelectedUnit` is inserted during `Startup`; defaults to None
- `UnitData.unit_type_id` must reference a valid entry in `UnitTypeRegistry` (or be handled gracefully if the type was deleted)
- `PropertyValue` variant must match the corresponding `PropertyType` variant
- `PropertyValue::Enum` value must be one of the options in the referenced `EnumDefinition`
- `TypeId` values are globally unique (UUID-based)
- Enum definitions are duplicated in both CellTypeRegistry and UnitTypeRegistry (future consolidation planned)

## Changelog
| Date | Change | Reason |
|------|--------|--------|
| 2026-02-08 | Initial definition | M2 Game System container and property system |
| 2026-02-09 | Renamed Vertex→Cell terminology | Cell is mathematically correct for N-dimensional grid elements; Vertex means hex corner in grid terminology |
| 2026-02-09 | Added unit types section | M3 — units on the hex grid |
