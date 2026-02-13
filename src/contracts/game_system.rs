//! Shared Game System types. See `.specs/contracts/game_system.md`.
//!
//! Contains the Game System container, the entity-agnostic property system,
//! and the unified entity type system. M4 replaces separate `CellType`/`UnitType`
//! with `EntityType` + `EntityRole`.

use std::collections::HashMap;

use bevy::prelude::*;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Identity
// ---------------------------------------------------------------------------

/// Unique identifier for types within the Game System (entity types,
/// enum definitions, property definitions, concepts, relations,
/// constraints, etc.). Uses UUID v4 for stability across future
/// serialization.
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

/// The root design artifact. All definitions (entity types, property schemas,
/// enum definitions, concepts, relations, constraints) belong to a Game System.
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
/// Property definitions are reusable across entity types regardless of role.
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
// Entity Types (M4 — unified, replaces CellType and UnitType)
// ---------------------------------------------------------------------------

/// The role an entity type plays in the game system.
/// Determines how instances interact with the grid and other entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityRole {
    /// Occupies a hex position on the board (replaces `CellType`).
    /// Each hex tile has exactly one `BoardPosition` entity type.
    BoardPosition,
    /// A movable game piece placed on hex tiles (replaces `UnitType`).
    /// Multiple tokens may occupy the same hex position.
    Token,
}

/// A unified entity type definition. Replaces both `CellType` and `UnitType`.
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
/// Replaces `CellTypeRegistry` and `UnitTypeRegistry`.
#[derive(Resource, Debug, Default)]
pub struct EntityTypeRegistry {
    pub types: Vec<EntityType>,
    pub enum_definitions: Vec<EnumDefinition>,
}

impl EntityTypeRegistry {
    /// Look up an entity type by its ID.
    pub fn get(&self, id: TypeId) -> Option<&EntityType> {
        self.types.iter().find(|t| t.id == id)
    }

    /// Look up an enum definition by its ID.
    pub fn get_enum(&self, id: TypeId) -> Option<&EnumDefinition> {
        self.enum_definitions.iter().find(|e| e.id == id)
    }

    /// Returns all entity types with the given role.
    pub fn types_by_role(&self, role: EntityRole) -> Vec<&EntityType> {
        self.types.iter().filter(|t| t.role == role).collect()
    }

    /// Returns the first entity type with the given role, if any.
    pub fn first_by_role(&self, role: EntityRole) -> Option<&EntityType> {
        self.types.iter().find(|t| t.role == role)
    }
}

/// Component attached to any entity on the hex grid (tiles and tokens).
/// Stores the entity type and per-instance property values.
/// Replaces both `CellData` and `UnitData`.
#[derive(Component, Debug, Clone)]
pub struct EntityData {
    pub entity_type_id: TypeId,
    /// Per-instance property values, keyed by `PropertyDefinition` ID.
    /// Entities get default values from their type; users can
    /// override individual values via the inspector.
    pub properties: HashMap<TypeId, PropertyValue>,
}

/// Marker component for token entities on the hex grid.
/// Used to distinguish tokens from tiles in queries.
#[derive(Component, Debug)]
pub struct UnitInstance;

/// Tracks which `BoardPosition` entity type the user is currently painting with.
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
    pub position: super::hex_grid::HexPosition,
    pub entity_type_id: TypeId,
}
