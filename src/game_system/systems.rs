//! Systems and factory functions for the game_system feature plugin.

use bevy::prelude::*;

use crate::contracts::game_system::{
    GameSystem, TypeId, CellType, CellTypeRegistry, UnitType, UnitTypeRegistry,
};

/// Creates a new `GameSystem` resource with a fresh UUID and default version.
pub fn create_game_system() -> GameSystem {
    GameSystem {
        id: uuid::Uuid::new_v4().to_string(),
        version: "0.1.0".to_string(),
    }
}

/// Creates the default `CellTypeRegistry` populated with starter cell types.
pub fn create_cell_type_registry() -> CellTypeRegistry {
    CellTypeRegistry {
        types: vec![
            CellType {
                id: TypeId::new(),
                name: "Plains".to_string(),
                color: Color::srgb(0.6, 0.8, 0.4),
                properties: Vec::new(),
            },
            CellType {
                id: TypeId::new(),
                name: "Forest".to_string(),
                color: Color::srgb(0.2, 0.5, 0.2),
                properties: Vec::new(),
            },
            CellType {
                id: TypeId::new(),
                name: "Water".to_string(),
                color: Color::srgb(0.2, 0.4, 0.8),
                properties: Vec::new(),
            },
            CellType {
                id: TypeId::new(),
                name: "Mountain".to_string(),
                color: Color::srgb(0.5, 0.4, 0.3),
                properties: Vec::new(),
            },
            CellType {
                id: TypeId::new(),
                name: "Road".to_string(),
                color: Color::srgb(0.7, 0.6, 0.4),
                properties: Vec::new(),
            },
        ],
        enum_definitions: Vec::new(),
    }
}

/// Creates the default `UnitTypeRegistry` populated with starter unit types.
pub fn create_unit_type_registry() -> UnitTypeRegistry {
    UnitTypeRegistry {
        types: vec![
            UnitType {
                id: TypeId::new(),
                name: "Infantry".to_string(),
                color: Color::srgb(0.2, 0.4, 0.7),
                properties: Vec::new(),
            },
            UnitType {
                id: TypeId::new(),
                name: "Cavalry".to_string(),
                color: Color::srgb(0.7, 0.3, 0.2),
                properties: Vec::new(),
            },
            UnitType {
                id: TypeId::new(),
                name: "Artillery".to_string(),
                color: Color::srgb(0.6, 0.6, 0.2),
                properties: Vec::new(),
            },
        ],
        enum_definitions: Vec::new(),
    }
}
