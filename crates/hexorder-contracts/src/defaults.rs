//! Factory functions for creating default/starter instances of contract types.
//!
//! These are used by both `game_system` (at startup) and `persistence`
//! (when resetting to a new project). They produce opinionated starter
//! data — not empty defaults (use `Default::default()` for those).

use bevy::prelude::*;

use crate::game_system::{
    EntityRole, EntityType, EntityTypeRegistry, EnumDefinition, EnumRegistry, GameSystem,
    PropertyDefinition, PropertyType, PropertyValue, TypeId,
};
use crate::mechanics::{
    CombatOutcome, CombatResultsTable, OutcomeEffect, Phase, PhaseType, PlayerOrder, TurnStructure,
};
use crate::simulation::{ColumnType, ResolutionTable, TableColumn, TableRow};

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
    }
}

/// Creates the default `EnumRegistry` with starter enum definitions.
pub fn create_enum_registry() -> EnumRegistry {
    let mut reg = EnumRegistry::default();

    let terrain_id = TypeId::new();
    reg.insert(EnumDefinition {
        id: terrain_id,
        name: "Terrain Type".to_string(),
        options: vec![
            "Open".to_string(),
            "Rough".to_string(),
            "Impassable".to_string(),
        ],
    });

    let movement_id = TypeId::new();
    reg.insert(EnumDefinition {
        id: movement_id,
        name: "Movement Mode".to_string(),
        options: vec![
            "Foot".to_string(),
            "Wheeled".to_string(),
            "Tracked".to_string(),
        ],
    });

    reg
}

/// Creates a default 5-phase turn structure for new game systems.
pub fn create_default_turn_structure() -> TurnStructure {
    TurnStructure {
        phases: vec![
            Phase {
                id: TypeId::new(),
                name: "Reinforcement Phase".to_string(),
                phase_type: PhaseType::Admin,
                description: "Place reinforcements and replacements.".to_string(),
            },
            Phase {
                id: TypeId::new(),
                name: "Movement Phase".to_string(),
                phase_type: PhaseType::Movement,
                description: "Move units within their movement allowance.".to_string(),
            },
            Phase {
                id: TypeId::new(),
                name: "Combat Phase".to_string(),
                phase_type: PhaseType::Combat,
                description: "Declare and resolve attacks.".to_string(),
            },
            Phase {
                id: TypeId::new(),
                name: "Supply Phase".to_string(),
                phase_type: PhaseType::Admin,
                description: "Check supply lines and attrition.".to_string(),
            },
            Phase {
                id: TypeId::new(),
                name: "Victory Check Phase".to_string(),
                phase_type: PhaseType::Admin,
                description: "Evaluate victory conditions.".to_string(),
            },
        ],
        player_order: PlayerOrder::Alternating,
    }
}

/// Creates a default CRT with standard odds-ratio columns and 6 rows (1d6).
pub fn create_default_crt() -> CombatResultsTable {
    let columns = vec![
        TableColumn {
            label: "1:2".to_string(),
            column_type: ColumnType::Ratio,
            threshold: 0.5,
        },
        TableColumn {
            label: "1:1".to_string(),
            column_type: ColumnType::Ratio,
            threshold: 1.0,
        },
        TableColumn {
            label: "2:1".to_string(),
            column_type: ColumnType::Ratio,
            threshold: 2.0,
        },
        TableColumn {
            label: "3:1".to_string(),
            column_type: ColumnType::Ratio,
            threshold: 3.0,
        },
        TableColumn {
            label: "4:1".to_string(),
            column_type: ColumnType::Ratio,
            threshold: 4.0,
        },
        TableColumn {
            label: "5:1".to_string(),
            column_type: ColumnType::Ratio,
            threshold: 5.0,
        },
        TableColumn {
            label: "6:1".to_string(),
            column_type: ColumnType::Ratio,
            threshold: 6.0,
        },
    ];

    let rows: Vec<TableRow> = (1..=6)
        .map(|i| TableRow {
            label: i.to_string(),
            value_min: i,
            value_max: i,
        })
        .collect();

    let outcomes = vec![
        outcome_row(&["AE", "AE", "AR", "EX", "DR", "DS", "DE"]),
        outcome_row(&["AE", "AR", "EX", "DR", "DS", "DE", "DE"]),
        outcome_row(&["AR", "EX", "DR", "DR", "DS", "DE", "DE"]),
        outcome_row(&["AR", "NE", "DR", "DS", "DE", "DE", "DE"]),
        outcome_row(&["NE", "DR", "DS", "DS", "DE", "DE", "DE"]),
        outcome_row(&["DR", "DR", "DS", "DE", "DE", "DE", "DE"]),
    ];

    CombatResultsTable {
        id: TypeId::new(),
        name: "Combat Results Table".to_string(),
        table: ResolutionTable {
            id: TypeId::new(),
            name: "CRT Lookup".to_string(),
            columns,
            rows,
            outcomes: Vec::new(),
        },
        outcomes,
        combat_concept_id: None,
    }
}

/// Helper: create a row of combat outcomes from label strings.
fn outcome_row(labels: &[&str]) -> Vec<CombatOutcome> {
    labels
        .iter()
        .map(|label| {
            let effect = match *label {
                "AE" => Some(OutcomeEffect::AttackerEliminated),
                "DE" => Some(OutcomeEffect::DefenderEliminated),
                "AR" => Some(OutcomeEffect::AttackerStepLoss { steps: 1 }),
                "DS" => Some(OutcomeEffect::StepLoss { steps: 1 }),
                "DR" => Some(OutcomeEffect::Retreat { hexes: 1 }),
                "EX" => Some(OutcomeEffect::Exchange {
                    attacker_steps: 1,
                    defender_steps: 1,
                }),
                "NE" => Some(OutcomeEffect::NoEffect),
                _ => None,
            };
            CombatOutcome {
                label: (*label).to_string(),
                effect,
            }
        })
        .collect()
}
