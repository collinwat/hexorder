//! Shared Game System types. See `.specs/contracts/game_system.md`.
//!
//! Contains the Game System container, the entity-agnostic property system,
//! and cell type definitions. Cell types are Game System definitions
//! and live here rather than in a separate contract.

use std::collections::HashMap;

use bevy::prelude::*;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Identity
// ---------------------------------------------------------------------------

/// Unique identifier for types within the Game System (cell types,
/// enum definitions, property definitions, etc.). Uses UUID v4 for
/// stability across future serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub Uuid);

impl TypeId {
    /// Generate a new random `TypeId`.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TypeId {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Game System Container
// ---------------------------------------------------------------------------

/// The root design artifact. All definitions (cell types, property schemas,
/// enum definitions, and in future milestones unit types and rules) belong to
/// a Game System.
#[derive(Resource, Debug)]
pub struct GameSystem {
    /// Unique identifier for this game system.
    pub id: String,
    /// Semantic version string (e.g., "0.1.0").
    pub version: String,
}

// ---------------------------------------------------------------------------
// Property System (entity-agnostic)
// ---------------------------------------------------------------------------

/// The data type of a property definition.
/// Extensible — future milestones will add `EntityRef`, List, Map, Struct, etc.
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyType {
    Bool,
    Int,
    Float,
    String,
    Color,
    /// References an `EnumDefinition` by its `TypeId`.
    Enum(TypeId),
}

/// A concrete value for a property instance.
/// The variant must match the corresponding `PropertyType`.
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Color(bevy::color::Color),
    /// The selected option name from the referenced `EnumDefinition`.
    Enum(String),
}

impl PropertyValue {
    /// Returns a default value for the given property type.
    pub fn default_for(property_type: &PropertyType) -> Self {
        match property_type {
            PropertyType::Bool => PropertyValue::Bool(false),
            PropertyType::Int => PropertyValue::Int(0),
            PropertyType::Float => PropertyValue::Float(0.0),
            PropertyType::String => PropertyValue::String(String::new()),
            PropertyType::Color => PropertyValue::Color(bevy::color::Color::WHITE),
            PropertyType::Enum(_) => PropertyValue::Enum(String::new()),
        }
    }
}

/// A property schema entry defining a named, typed property with a default value.
/// Property definitions are reusable across entity types (cell types, future
/// unit types, etc.).
#[derive(Debug, Clone)]
pub struct PropertyDefinition {
    pub id: TypeId,
    pub name: String,
    pub property_type: PropertyType,
    pub default_value: PropertyValue,
}

/// A named set of string options for Enum-type properties.
/// For example: "Movement Mode" with options `["Foot", "Wheeled", "Tracked"]`.
#[derive(Debug, Clone)]
pub struct EnumDefinition {
    pub id: TypeId,
    pub name: String,
    pub options: Vec<String>,
}

// ---------------------------------------------------------------------------
// Cell Types (Game System definitions for board positions)
// ---------------------------------------------------------------------------

/// Alias for clarity — cell types are identified by the same `TypeId`.
pub type CellTypeId = TypeId;

/// A cell type definition: describes what a board position can be.
/// Defined by the user within a Game System.
#[derive(Debug, Clone)]
pub struct CellType {
    pub id: CellTypeId,
    pub name: String,
    pub color: bevy::color::Color,
    pub properties: Vec<PropertyDefinition>,
}

/// Registry of all defined cell types and enum definitions in the
/// current Game System. This is a resource managed by the `game_system` plugin.
#[derive(Resource, Debug, Default)]
pub struct CellTypeRegistry {
    pub types: Vec<CellType>,
    pub enum_definitions: Vec<EnumDefinition>,
}

impl CellTypeRegistry {
    /// Look up a cell type by its ID.
    pub fn get(&self, id: CellTypeId) -> Option<&CellType> {
        self.types.iter().find(|t| t.id == id)
    }

    /// Look up an enum definition by its ID.
    pub fn get_enum(&self, id: TypeId) -> Option<&EnumDefinition> {
        self.enum_definitions.iter().find(|e| e.id == id)
    }

    /// Returns the first registered cell type, if any.
    pub fn first(&self) -> Option<&CellType> {
        self.types.first()
    }
}

/// Component attached to hex tile entities. Stores which cell type this
/// tile is and its per-instance property values.
#[derive(Component, Debug, Clone)]
pub struct CellData {
    pub cell_type_id: CellTypeId,
    /// Per-instance property values, keyed by `PropertyDefinition` ID.
    /// Painted tiles get default values from the cell type; users can
    /// override individual values via the inspector.
    pub properties: HashMap<TypeId, PropertyValue>,
}

/// Tracks which cell type the user is currently painting with.
#[derive(Resource, Debug, Default)]
pub struct ActiveCellType {
    pub cell_type_id: Option<CellTypeId>,
}

// ---------------------------------------------------------------------------
// Unit Types (Game System definitions for entities on the board)
// ---------------------------------------------------------------------------

/// Alias for clarity — unit types are identified by the same `TypeId`.
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

/// Registry of all defined unit types and their enum definitions in the
/// current Game System. This is a resource managed by the `game_system` plugin.
#[derive(Resource, Debug, Default)]
pub struct UnitTypeRegistry {
    pub types: Vec<UnitType>,
    pub enum_definitions: Vec<EnumDefinition>,
}

impl UnitTypeRegistry {
    /// Look up a unit type by its ID.
    pub fn get(&self, id: UnitTypeId) -> Option<&UnitType> {
        self.types.iter().find(|t| t.id == id)
    }

    /// Look up an enum definition by its ID.
    pub fn get_enum(&self, id: TypeId) -> Option<&EnumDefinition> {
        self.enum_definitions.iter().find(|e| e.id == id)
    }

    /// Returns the first registered unit type, if any.
    pub fn first(&self) -> Option<&UnitType> {
        self.types.first()
    }
}

/// Component attached to entities representing units on the hex grid.
/// Stores which unit type this entity is and its per-instance property values.
#[derive(Component, Debug, Clone)]
pub struct UnitData {
    pub unit_type_id: UnitTypeId,
    /// Per-instance property values, keyed by `PropertyDefinition` ID.
    /// Placed units get default values from the unit type; users can
    /// override individual values via the inspector.
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
    pub position: super::hex_grid::HexPosition,
    pub unit_type_id: UnitTypeId,
}
