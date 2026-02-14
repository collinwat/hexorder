//! Systems and factory functions for the `game_system` plugin.

use bevy::prelude::*;

use crate::contracts::game_system::{
    EntityRole, EntityType, EntityTypeRegistry, GameSystem, PropertyDefinition, PropertyType,
    PropertyValue, TypeId,
};

/// Creates a new `GameSystem` resource with a fresh UUID and default version.
pub fn create_game_system() -> GameSystem {
    GameSystem {
        id: uuid::Uuid::new_v4().to_string(),
        version: "0.1.0".to_string(),
    }
}

/// Creates the default `EntityTypeRegistry` populated with starter entity types.
/// Includes 5 `BoardPosition` types and 3 `Token` types.
pub fn create_entity_type_registry() -> EntityTypeRegistry {
    EntityTypeRegistry {
        types: vec![
            // -- BoardPosition types (terrain) --
            EntityType {
                id: TypeId::new(),
                name: "Plains".to_string(),
                role: EntityRole::BoardPosition,
                color: Color::srgb(0.6, 0.8, 0.4),
                properties: Vec::new(),
            },
            EntityType {
                id: TypeId::new(),
                name: "Forest".to_string(),
                role: EntityRole::BoardPosition,
                color: Color::srgb(0.2, 0.5, 0.2),
                properties: Vec::new(),
            },
            EntityType {
                id: TypeId::new(),
                name: "Water".to_string(),
                role: EntityRole::BoardPosition,
                color: Color::srgb(0.2, 0.4, 0.8),
                properties: Vec::new(),
            },
            EntityType {
                id: TypeId::new(),
                name: "Mountain".to_string(),
                role: EntityRole::BoardPosition,
                color: Color::srgb(0.5, 0.4, 0.3),
                properties: vec![PropertyDefinition {
                    id: TypeId::new(),
                    name: "Movement Cost".to_string(),
                    property_type: PropertyType::Int,
                    default_value: PropertyValue::Int(3),
                }],
            },
            EntityType {
                id: TypeId::new(),
                name: "Road".to_string(),
                role: EntityRole::BoardPosition,
                color: Color::srgb(0.7, 0.6, 0.4),
                properties: Vec::new(),
            },
            // -- Token types (units) --
            EntityType {
                id: TypeId::new(),
                name: "Infantry".to_string(),
                role: EntityRole::Token,
                color: Color::srgb(0.2, 0.4, 0.7),
                properties: vec![PropertyDefinition {
                    id: TypeId::new(),
                    name: "Movement Points".to_string(),
                    property_type: PropertyType::Int,
                    default_value: PropertyValue::Int(4),
                }],
            },
            EntityType {
                id: TypeId::new(),
                name: "Cavalry".to_string(),
                role: EntityRole::Token,
                color: Color::srgb(0.7, 0.3, 0.2),
                properties: Vec::new(),
            },
            EntityType {
                id: TypeId::new(),
                name: "Artillery".to_string(),
                role: EntityRole::Token,
                color: Color::srgb(0.6, 0.6, 0.2),
                properties: Vec::new(),
            },
        ],
        enum_definitions: Vec::new(),
    }
}
