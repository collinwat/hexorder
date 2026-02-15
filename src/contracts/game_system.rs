//! Shared Game System types. See `docs/contracts/game-system.md`.
//!
//! Contains the Game System container, the entity-agnostic property system,
//! and the unified entity type system. 0.4.0 replaces separate `CellType`/`UnitType`
//! with `EntityType` + `EntityRole`.

use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Identity
// ---------------------------------------------------------------------------

/// Unique identifier for types within the Game System (entity types,
/// enum definitions, property definitions, concepts, relations,
/// constraints, etc.). Uses UUID v4 for stability across future
/// serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
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
#[derive(Resource, Debug, Clone, Reflect, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(opaque)]
pub enum PropertyType {
    Bool,
    Int,
    Float,
    String,
    Color,
    /// References an `EnumDefinition` by its `TypeId`.
    Enum(TypeId),
    /// Reference to an entity type, optionally filtered by role.
    EntityRef(Option<EntityRole>),
    /// Ordered collection of a single inner property type.
    List(Box<PropertyType>),
    /// Map with enum keys (by `TypeId`) and typed values.
    Map(TypeId, Box<PropertyType>),
    /// Named composite referencing a `StructDefinition` by `TypeId`.
    Struct(TypeId),
    /// Bounded integer with min/max validation.
    IntRange {
        min: i64,
        max: i64,
    },
    /// Bounded float with min/max validation.
    FloatRange {
        min: f64,
        max: f64,
    },
}

/// A concrete value for a property instance.
/// The variant must match the corresponding `PropertyType`.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(opaque)]
pub enum PropertyValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Color(bevy::color::Color),
    /// The selected option name from the referenced `EnumDefinition`.
    Enum(String),
    /// Reference to an entity type, or None if unset.
    EntityRef(Option<TypeId>),
    /// Ordered collection of values.
    List(Vec<PropertyValue>),
    /// Enum key name to value pairs (preserves insertion order for display).
    Map(Vec<(String, PropertyValue)>),
    /// Field values keyed by `PropertyDefinition` ID from the `StructDefinition`.
    Struct(HashMap<TypeId, PropertyValue>),
    /// Bounded integer value.
    IntRange(i64),
    /// Bounded float value.
    FloatRange(f64),
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
            PropertyType::EntityRef(_) => PropertyValue::EntityRef(None),
            PropertyType::List(_) => PropertyValue::List(Vec::new()),
            PropertyType::Map(_, _) => PropertyValue::Map(Vec::new()),
            PropertyType::Struct(_) => PropertyValue::Struct(HashMap::new()),
            PropertyType::IntRange { min, .. } => PropertyValue::IntRange(*min),
            PropertyType::FloatRange { min, .. } => PropertyValue::FloatRange(*min),
        }
    }
}

/// A property schema entry defining a named, typed property with a default value.
/// Property definitions are reusable across entity types regardless of role.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct PropertyDefinition {
    pub id: TypeId,
    pub name: String,
    pub property_type: PropertyType,
    pub default_value: PropertyValue,
}

/// A named set of string options for Enum-type properties.
/// For example: "Movement Mode" with options `["Foot", "Wheeled", "Tracked"]`.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct EnumDefinition {
    pub id: TypeId,
    pub name: String,
    pub options: Vec<String>,
}

/// Standalone registry of all designer-defined enum definitions.
/// Replaces `EntityTypeRegistry.enum_definitions` (0.7.0).
#[derive(Resource, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
pub struct EnumRegistry {
    pub definitions: HashMap<TypeId, EnumDefinition>,
}

impl EnumRegistry {
    /// Look up an enum definition by its ID.
    pub fn get(&self, id: TypeId) -> Option<&EnumDefinition> {
        self.definitions.get(&id)
    }

    /// Look up a mutable enum definition by its ID.
    pub fn get_mut(&mut self, id: TypeId) -> Option<&mut EnumDefinition> {
        self.definitions.get_mut(&id)
    }

    /// Insert or replace an enum definition.
    pub fn insert(&mut self, def: EnumDefinition) {
        self.definitions.insert(def.id, def);
    }

    /// Remove an enum definition by ID. Returns the removed definition.
    pub fn remove(&mut self, id: TypeId) -> Option<EnumDefinition> {
        self.definitions.remove(&id)
    }
}

/// A named composite type — a list of typed, named fields.
/// Registered centrally so multiple entity types can reference the same struct schema.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct StructDefinition {
    pub id: TypeId,
    pub name: String,
    pub fields: Vec<PropertyDefinition>,
}

/// Standalone registry of all designer-defined struct definitions (0.7.0).
#[derive(Resource, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
pub struct StructRegistry {
    pub definitions: HashMap<TypeId, StructDefinition>,
}

impl StructRegistry {
    /// Look up a struct definition by its ID.
    pub fn get(&self, id: TypeId) -> Option<&StructDefinition> {
        self.definitions.get(&id)
    }

    /// Look up a mutable struct definition by its ID.
    pub fn get_mut(&mut self, id: TypeId) -> Option<&mut StructDefinition> {
        self.definitions.get_mut(&id)
    }

    /// Insert or replace a struct definition.
    pub fn insert(&mut self, def: StructDefinition) {
        self.definitions.insert(def.id, def);
    }

    /// Remove a struct definition by ID. Returns the removed definition.
    pub fn remove(&mut self, id: TypeId) -> Option<StructDefinition> {
        self.definitions.remove(&id)
    }
}

// ---------------------------------------------------------------------------
// Entity Types (0.4.0 — unified, replaces CellType and UnitType)
// ---------------------------------------------------------------------------

/// The role an entity type plays in the game system.
/// Determines how instances interact with the grid and other entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct EntityType {
    pub id: TypeId,
    pub name: String,
    pub role: EntityRole,
    pub color: bevy::color::Color,
    pub properties: Vec<PropertyDefinition>,
}

/// Unified registry of all entity types.
/// Replaces `CellTypeRegistry` and `UnitTypeRegistry`.
#[derive(Resource, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
pub struct EntityTypeRegistry {
    pub types: Vec<EntityType>,
}

impl EntityTypeRegistry {
    /// Look up an entity type by its ID.
    pub fn get(&self, id: TypeId) -> Option<&EntityType> {
        self.types.iter().find(|t| t.id == id)
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
#[derive(Component, Debug, Clone, Reflect)]
pub struct EntityData {
    pub entity_type_id: TypeId,
    /// Per-instance property values, keyed by `PropertyDefinition` ID.
    /// Entities get default values from their type; users can
    /// override individual values via the inspector.
    pub properties: HashMap<TypeId, PropertyValue>,
}

/// Marker component for token entities on the hex grid.
/// Used to distinguish tokens from tiles in queries.
#[derive(Component, Debug, Reflect)]
pub struct UnitInstance;

/// Tracks which `BoardPosition` entity type the user is currently painting with.
#[derive(Resource, Debug, Default, Reflect)]
pub struct ActiveBoardType {
    pub entity_type_id: Option<TypeId>,
}

/// Tracks which Token entity type the user is currently placing.
#[derive(Resource, Debug, Default, Reflect)]
pub struct ActiveTokenType {
    pub entity_type_id: Option<TypeId>,
}

/// Tracks the currently selected unit entity, if any.
#[derive(Resource, Debug, Default, Reflect)]
pub struct SelectedUnit {
    pub entity: Option<Entity>,
}

/// Fired when a token entity is placed on the grid.
#[derive(Event, Debug, Reflect)]
pub struct UnitPlacedEvent {
    pub entity: Entity,
    pub position: super::hex_grid::HexPosition,
    pub entity_type_id: TypeId,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Round-trip: serialize `EntityTypeRegistry` to RON, deserialize, verify.
    #[test]
    fn entity_type_registry_ron_round_trip() {
        let registry = EntityTypeRegistry {
            types: vec![
                EntityType {
                    id: TypeId::new(),
                    name: "Plains".to_string(),
                    role: EntityRole::BoardPosition,
                    color: bevy::color::Color::srgb(0.6, 0.8, 0.4),
                    properties: vec![PropertyDefinition {
                        id: TypeId::new(),
                        name: "Movement Cost".to_string(),
                        property_type: PropertyType::Int,
                        default_value: PropertyValue::Int(1),
                    }],
                },
                EntityType {
                    id: TypeId::new(),
                    name: "Infantry".to_string(),
                    role: EntityRole::Token,
                    color: bevy::color::Color::srgb(0.2, 0.4, 0.7),
                    properties: Vec::new(),
                },
            ],
        };

        let ron_str = ron::to_string(&registry).expect("serialize");
        let deserialized: EntityTypeRegistry = ron::from_str(&ron_str).expect("deserialize");

        assert_eq!(deserialized.types.len(), 2);
        assert_eq!(deserialized.types[0].name, "Plains");
        assert_eq!(deserialized.types[0].role, EntityRole::BoardPosition);
        assert_eq!(deserialized.types[0].properties.len(), 1);
        assert_eq!(deserialized.types[1].name, "Infantry");
        assert_eq!(deserialized.types[1].role, EntityRole::Token);
    }

    #[test]
    fn enum_registry_insert_and_get() {
        let mut reg = EnumRegistry::default();
        let id = TypeId::new();
        let def = EnumDefinition {
            id,
            name: "Terrain".to_string(),
            options: vec!["Grass".to_string(), "Sand".to_string()],
        };
        reg.definitions.insert(id, def);
        assert_eq!(reg.definitions.len(), 1);
        assert_eq!(reg.get(id).expect("should find").name, "Terrain");
        assert!(reg.get(TypeId::new()).is_none());
    }

    #[test]
    fn enum_registry_ron_round_trip() {
        let mut reg = EnumRegistry::default();
        let id = TypeId::new();
        reg.definitions.insert(
            id,
            EnumDefinition {
                id,
                name: "Side".to_string(),
                options: vec!["Axis".to_string(), "Allied".to_string()],
            },
        );
        let ron_str = ron::to_string(&reg).expect("serialize");
        let loaded: EnumRegistry = ron::from_str(&ron_str).expect("deserialize");
        assert_eq!(loaded.definitions.len(), 1);
        assert_eq!(loaded.get(id).expect("should find").options.len(), 2);
    }

    #[test]
    fn struct_registry_insert_and_get() {
        let mut reg = StructRegistry::default();
        let id = TypeId::new();
        let def = StructDefinition {
            id,
            name: "CombatProfile".to_string(),
            fields: vec![
                PropertyDefinition {
                    id: TypeId::new(),
                    name: "attack".to_string(),
                    property_type: PropertyType::Int,
                    default_value: PropertyValue::Int(0),
                },
                PropertyDefinition {
                    id: TypeId::new(),
                    name: "defense".to_string(),
                    property_type: PropertyType::Int,
                    default_value: PropertyValue::Int(0),
                },
            ],
        };
        reg.definitions.insert(id, def);
        assert_eq!(reg.definitions.len(), 1);
        assert_eq!(reg.get(id).expect("should find").name, "CombatProfile");
        assert_eq!(reg.get(id).expect("should find").fields.len(), 2);
        assert!(reg.get(TypeId::new()).is_none());
    }

    #[test]
    fn struct_registry_ron_round_trip() {
        let mut reg = StructRegistry::default();
        let id = TypeId::new();
        reg.definitions.insert(
            id,
            StructDefinition {
                id,
                name: "Stats".to_string(),
                fields: vec![PropertyDefinition {
                    id: TypeId::new(),
                    name: "hp".to_string(),
                    property_type: PropertyType::Int,
                    default_value: PropertyValue::Int(10),
                }],
            },
        );
        let ron_str = ron::to_string(&reg).expect("serialize");
        let loaded: StructRegistry = ron::from_str(&ron_str).expect("deserialize");
        assert_eq!(loaded.definitions.len(), 1);
        assert_eq!(loaded.get(id).expect("should find").fields.len(), 1);
    }

    #[test]
    fn compound_property_value_ron_round_trip() {
        use std::collections::HashMap;

        let values: Vec<PropertyValue> = vec![
            PropertyValue::EntityRef(Some(TypeId::new())),
            PropertyValue::EntityRef(None),
            PropertyValue::List(vec![PropertyValue::Int(1), PropertyValue::Int(2)]),
            PropertyValue::Map(vec![
                ("Grass".to_string(), PropertyValue::Int(1)),
                ("Sand".to_string(), PropertyValue::Int(2)),
            ]),
            PropertyValue::Struct({
                let mut m = HashMap::new();
                m.insert(TypeId::new(), PropertyValue::Int(5));
                m.insert(TypeId::new(), PropertyValue::String("test".to_string()));
                m
            }),
            PropertyValue::IntRange(7),
            PropertyValue::FloatRange(0.5),
        ];

        for value in &values {
            let ron_str = ron::to_string(value).expect("serialize");
            let loaded: PropertyValue = ron::from_str(&ron_str).expect("deserialize");
            assert_eq!(&loaded, value, "Round-trip failed for {value:?}");
        }
    }

    #[test]
    fn nested_compound_type_ron_round_trip() {
        let field_id = TypeId::new();
        let inner = PropertyValue::Struct({
            let mut m = std::collections::HashMap::new();
            m.insert(
                field_id,
                PropertyValue::List(vec![PropertyValue::Int(1), PropertyValue::Int(2)]),
            );
            m
        });
        let map_val = PropertyValue::Map(vec![("Key".to_string(), inner)]);

        let ron_str = ron::to_string(&map_val).expect("serialize");
        let loaded: PropertyValue = ron::from_str(&ron_str).expect("deserialize");
        assert_eq!(loaded, map_val);
    }

    #[test]
    fn property_type_new_variants_are_distinct() {
        let enum_id = TypeId::new();
        let struct_id = TypeId::new();
        let variants: Vec<PropertyType> = vec![
            PropertyType::EntityRef(None),
            PropertyType::EntityRef(Some(EntityRole::Token)),
            PropertyType::List(Box::new(PropertyType::Int)),
            PropertyType::Map(enum_id, Box::new(PropertyType::Int)),
            PropertyType::Struct(struct_id),
            PropertyType::IntRange { min: 0, max: 10 },
            PropertyType::FloatRange { min: 0.0, max: 1.0 },
        ];
        for (i, a) in variants.iter().enumerate() {
            for (j, b) in variants.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "Variants {i} and {j} should differ");
                }
            }
        }
    }
}
