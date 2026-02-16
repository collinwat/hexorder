//! Unit tests for the `rules_engine` plugin.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::contracts::game_system::{
    EntityData, EntityRole, EntityType, EntityTypeRegistry, PropertyDefinition, PropertyType,
    PropertyValue, SelectedUnit, TypeId, UnitInstance,
};
use crate::contracts::hex_grid::{HexGridConfig, HexPosition, HexTile};
use crate::contracts::ontology::{
    Concept, ConceptBinding, ConceptRegistry, ConceptRole, ConstraintExpr, ConstraintRegistry,
    ModifyOperation, PropertyBinding, Relation, RelationEffect, RelationRegistry, RelationTrigger,
};
use crate::contracts::persistence::AppScreen;
use crate::contracts::validation::ValidMoveSet;

/// Creates a minimal headless test app with the `RulesEnginePlugin`.
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.insert_state(AppScreen::Editor);
    app.insert_resource(HexGridConfig {
        layout: hexx::HexLayout {
            orientation: hexx::HexOrientation::Pointy,
            scale: bevy::math::Vec2::splat(1.0),
            origin: bevy::math::Vec2::ZERO,
        },
        map_radius: 3,
    });
    app.init_resource::<EntityTypeRegistry>();
    app.init_resource::<SelectedUnit>();
    app.init_resource::<ConceptRegistry>();
    app.init_resource::<RelationRegistry>();
    app.init_resource::<ConstraintRegistry>();
    app.add_plugins(super::RulesEnginePlugin);
    app
}

/// Spawns hex tiles in a grid of the given radius with the given entity type ID.
fn spawn_hex_grid(app: &mut App, radius: u32, tile_type_id: TypeId) {
    let radius_i = radius as i32;
    for q in -radius_i..=radius_i {
        for r in -radius_i..=radius_i {
            if (q + r).unsigned_abs() <= radius {
                app.world_mut().spawn((
                    HexTile,
                    HexPosition::new(q, r),
                    EntityData {
                        entity_type_id: tile_type_id,
                        properties: HashMap::new(),
                    },
                ));
            }
        }
    }
}

/// Spawns hex tiles with a custom property on each tile.
fn spawn_hex_grid_with_properties(
    app: &mut App,
    radius: u32,
    tile_type_id: TypeId,
    cost_prop_id: TypeId,
    cost_value: i64,
) {
    let radius_i = radius as i32;
    for q in -radius_i..=radius_i {
        for r in -radius_i..=radius_i {
            if (q + r).unsigned_abs() <= radius {
                let mut properties = HashMap::new();
                properties.insert(cost_prop_id, PropertyValue::Int(cost_value));
                app.world_mut().spawn((
                    HexTile,
                    HexPosition::new(q, r),
                    EntityData {
                        entity_type_id: tile_type_id,
                        properties,
                    },
                ));
            }
        }
    }
}

/// Spawns a unit entity at the given position with the given entity data.
fn spawn_unit(app: &mut App, q: i32, r: i32, entity_data: EntityData) -> Entity {
    app.world_mut()
        .spawn((UnitInstance, HexPosition::new(q, r), entity_data))
        .id()
}

/// Creates a basic Motion concept with traveler (`Token`) and terrain
/// (`BoardPosition`) roles.
struct MotionSetup {
    concept_id: TypeId,
    traveler_role_id: TypeId,
    terrain_role_id: TypeId,
    unit_type_id: TypeId,
    tile_type_id: TypeId,
    budget_prop_id: TypeId,
    cost_prop_id: TypeId,
}

fn setup_motion_ontology(app: &mut App, budget: i64, cost: i64) -> MotionSetup {
    let concept_id = TypeId::new();
    let traveler_role_id = TypeId::new();
    let terrain_role_id = TypeId::new();
    let unit_type_id = TypeId::new();
    let tile_type_id = TypeId::new();
    let budget_prop_id = TypeId::new();
    let cost_prop_id = TypeId::new();

    // Register entity types.
    let mut registry = EntityTypeRegistry::default();
    registry.types.push(EntityType {
        id: unit_type_id,
        name: "Infantry".to_string(),
        role: EntityRole::Token,
        color: bevy::color::Color::WHITE,
        properties: vec![PropertyDefinition {
            id: budget_prop_id,
            name: "movement_points".to_string(),
            property_type: PropertyType::Int,
            default_value: PropertyValue::Int(budget),
        }],
    });
    registry.types.push(EntityType {
        id: tile_type_id,
        name: "Plains".to_string(),
        role: EntityRole::BoardPosition,
        color: bevy::color::Color::srgb(0.3, 0.6, 0.2),
        properties: vec![PropertyDefinition {
            id: cost_prop_id,
            name: "terrain_cost".to_string(),
            property_type: PropertyType::Int,
            default_value: PropertyValue::Int(cost),
        }],
    });
    app.insert_resource(registry);

    // Set up concept.
    let concept = Concept {
        id: concept_id,
        name: "Motion".to_string(),
        description: "Movement across terrain".to_string(),
        role_labels: vec![
            ConceptRole {
                id: traveler_role_id,
                name: "traveler".to_string(),
                allowed_entity_roles: vec![EntityRole::Token],
            },
            ConceptRole {
                id: terrain_role_id,
                name: "terrain".to_string(),
                allowed_entity_roles: vec![EntityRole::BoardPosition],
            },
        ],
    };

    // Concept bindings.
    let unit_binding = ConceptBinding {
        id: TypeId::new(),
        entity_type_id: unit_type_id,
        concept_id,
        concept_role_id: traveler_role_id,
        property_bindings: vec![PropertyBinding {
            property_id: budget_prop_id,
            concept_local_name: "budget".to_string(),
        }],
    };

    let tile_binding = ConceptBinding {
        id: TypeId::new(),
        entity_type_id: tile_type_id,
        concept_id,
        concept_role_id: terrain_role_id,
        property_bindings: vec![PropertyBinding {
            property_id: cost_prop_id,
            concept_local_name: "cost".to_string(),
        }],
    };

    app.insert_resource(ConceptRegistry {
        concepts: vec![concept],
        bindings: vec![unit_binding, tile_binding],
    });

    // Relation: terrain cost subtracts from movement budget.
    let relation = Relation {
        id: TypeId::new(),
        name: "Terrain Movement Cost".to_string(),
        concept_id,
        subject_role_id: traveler_role_id,
        object_role_id: terrain_role_id,
        trigger: RelationTrigger::OnEnter,
        effect: RelationEffect::ModifyProperty {
            target_property: "budget".to_string(),
            source_property: "cost".to_string(),
            operation: ModifyOperation::Subtract,
        },
    };
    app.insert_resource(RelationRegistry {
        relations: vec![relation],
    });

    MotionSetup {
        concept_id,
        traveler_role_id,
        terrain_role_id,
        unit_type_id,
        tile_type_id,
        budget_prop_id,
        cost_prop_id,
    }
}

// -------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------

/// SC-2: `ValidMoveSet` exists after Startup.
#[test]
fn valid_move_set_resource_exists() {
    let mut app = test_app();
    app.update();

    let valid_moves = app.world().get_resource::<ValidMoveSet>();
    assert!(
        valid_moves.is_some(),
        "ValidMoveSet should exist after first update"
    );
}

/// SC-3: With no unit selected, `ValidMoveSet` is empty.
#[test]
fn valid_moves_empty_when_no_selection() {
    let mut app = test_app();
    app.update(); // First update: system runs because resources are "changed".
    app.update(); // Second update: system skips (no changes).

    let valid_moves = app.world().resource::<ValidMoveSet>();
    assert!(
        valid_moves.valid_positions.is_empty(),
        "Valid positions should be empty when no unit is selected"
    );
    assert!(
        valid_moves.for_entity.is_none(),
        "for_entity should be None when no unit is selected"
    );
}

/// SC-4: Selecting a unit populates `ValidMoveSet`.
#[test]
fn valid_moves_computed_on_selection() {
    let mut app = test_app();
    let setup = setup_motion_ontology(&mut app, 3, 1);

    // Spawn tiles with cost property.
    spawn_hex_grid_with_properties(&mut app, 3, setup.tile_type_id, setup.cost_prop_id, 1);

    // Spawn a unit at origin with budget = 3.
    let mut unit_props = HashMap::new();
    unit_props.insert(setup.budget_prop_id, PropertyValue::Int(3));
    let unit_entity = spawn_unit(
        &mut app,
        0,
        0,
        EntityData {
            entity_type_id: setup.unit_type_id,
            properties: unit_props,
        },
    );

    // Select the unit.
    app.world_mut().resource_mut::<SelectedUnit>().entity = Some(unit_entity);

    // Run systems.
    app.update();

    let valid_moves = app.world().resource::<ValidMoveSet>();
    assert!(
        !valid_moves.valid_positions.is_empty(),
        "Valid positions should be populated after selecting a unit"
    );
    assert_eq!(
        valid_moves.for_entity,
        Some(unit_entity),
        "for_entity should match selected unit"
    );
}

/// SC-5: Blocked positions have non-empty explanation strings.
#[test]
fn blocked_positions_have_explanations() {
    let mut app = test_app();
    let setup = setup_motion_ontology(&mut app, 2, 1);

    // Add a water tile type that blocks entry.
    let water_type_id = TypeId::new();
    {
        let mut registry = app.world_mut().resource_mut::<EntityTypeRegistry>();
        registry.types.push(EntityType {
            id: water_type_id,
            name: "Water".to_string(),
            role: EntityRole::BoardPosition,
            color: bevy::color::Color::srgb(0.1, 0.3, 0.8),
            properties: vec![],
        });
    }

    // Bind Water to the terrain role.
    {
        let mut concepts = app.world_mut().resource_mut::<ConceptRegistry>();
        concepts.bindings.push(ConceptBinding {
            id: TypeId::new(),
            entity_type_id: water_type_id,
            concept_id: setup.concept_id,
            concept_role_id: setup.terrain_role_id,
            property_bindings: vec![],
        });
    }

    // Add a Block relation on Water terrain.
    let block_relation_id = TypeId::new();
    {
        let mut relations = app.world_mut().resource_mut::<RelationRegistry>();
        relations.relations.push(Relation {
            id: block_relation_id,
            name: "Water Blocks Entry".to_string(),
            concept_id: setup.concept_id,
            subject_role_id: setup.traveler_role_id,
            object_role_id: setup.terrain_role_id,
            trigger: RelationTrigger::OnEnter,
            effect: RelationEffect::Block {
                condition: Some(ConstraintExpr::IsType {
                    role_id: setup.terrain_role_id,
                    entity_type_id: water_type_id,
                }),
            },
        });
    }

    // Spawn normal tiles, but put Water tiles at specific positions.
    let radius: u32 = 2;
    let radius_i = radius as i32;
    for q in -radius_i..=radius_i {
        for r in -radius_i..=radius_i {
            if (q + r).unsigned_abs() > radius {
                continue;
            }
            // Place Water at (1, 0) and (0, 1).
            let is_water = (q == 1 && r == 0) || (q == 0 && r == 1);
            let type_id = if is_water {
                water_type_id
            } else {
                setup.tile_type_id
            };
            let mut properties = HashMap::new();
            if !is_water {
                properties.insert(setup.cost_prop_id, PropertyValue::Int(1));
            }
            app.world_mut().spawn((
                HexTile,
                HexPosition::new(q, r),
                EntityData {
                    entity_type_id: type_id,
                    properties,
                },
            ));
        }
    }

    // Spawn unit at origin with budget = 4.
    let mut unit_props = HashMap::new();
    unit_props.insert(setup.budget_prop_id, PropertyValue::Int(4));
    let unit_entity = spawn_unit(
        &mut app,
        0,
        0,
        EntityData {
            entity_type_id: setup.unit_type_id,
            properties: unit_props,
        },
    );

    app.world_mut().resource_mut::<SelectedUnit>().entity = Some(unit_entity);
    app.update();

    let valid_moves = app.world().resource::<ValidMoveSet>();

    // Water positions should be blocked.
    let water_pos_1 = HexPosition::new(1, 0);
    let water_pos_2 = HexPosition::new(0, 1);

    assert!(
        !valid_moves.valid_positions.contains(&water_pos_1),
        "Water tile at (1,0) should not be in valid_positions"
    );
    assert!(
        !valid_moves.valid_positions.contains(&water_pos_2),
        "Water tile at (0,1) should not be in valid_positions"
    );

    // At least one blocked explanation should exist.
    let has_explanation = valid_moves.blocked_explanations.values().any(|reasons| {
        reasons
            .iter()
            .any(|r| !r.explanation.is_empty() && !r.satisfied)
    });
    assert!(
        has_explanation,
        "Blocked positions should have non-empty explanation strings"
    );
}

/// SC-6: Unit with budget N cannot reach positions costing more than N.
#[test]
fn path_budget_limits_range() {
    let mut app = test_app();
    let setup = setup_motion_ontology(&mut app, 2, 1);

    // Spawn tiles with cost = 1.
    spawn_hex_grid_with_properties(&mut app, 3, setup.tile_type_id, setup.cost_prop_id, 1);

    // Spawn a unit at origin with budget = 2.
    let mut unit_props = HashMap::new();
    unit_props.insert(setup.budget_prop_id, PropertyValue::Int(2));
    let unit_entity = spawn_unit(
        &mut app,
        0,
        0,
        EntityData {
            entity_type_id: setup.unit_type_id,
            properties: unit_props,
        },
    );

    app.world_mut().resource_mut::<SelectedUnit>().entity = Some(unit_entity);
    app.update();

    let valid_moves = app.world().resource::<ValidMoveSet>();

    // Positions at hex distance > 2 should NOT be reachable.
    // For example, (3, 0) is at distance 3 from origin.
    let far_pos = HexPosition::new(3, 0);
    assert!(
        !valid_moves.valid_positions.contains(&far_pos),
        "Position at distance 3 should not be reachable with budget 2"
    );

    // Positions at distance 1 should be reachable.
    let near_pos = HexPosition::new(1, 0);
    assert!(
        valid_moves.valid_positions.contains(&near_pos),
        "Position at distance 1 should be reachable with budget 2"
    );

    // Positions at distance 2 should be reachable.
    let mid_pos = HexPosition::new(2, 0);
    assert!(
        valid_moves.valid_positions.contains(&mid_pos),
        "Position at distance 2 should be reachable with budget 2"
    );
}

/// SC-7: Block relation on a specific entity type prevents movement.
#[test]
fn block_relation_prevents_entry() {
    let mut app = test_app();
    let setup = setup_motion_ontology(&mut app, 4, 1);

    // Add a Mountain tile type that blocks entry unconditionally.
    let mountain_type_id = TypeId::new();
    {
        let mut registry = app.world_mut().resource_mut::<EntityTypeRegistry>();
        registry.types.push(EntityType {
            id: mountain_type_id,
            name: "Mountain".to_string(),
            role: EntityRole::BoardPosition,
            color: bevy::color::Color::srgb(0.5, 0.5, 0.5),
            properties: vec![],
        });
    }

    // Bind Mountain to the terrain role.
    {
        let mut concepts = app.world_mut().resource_mut::<ConceptRegistry>();
        concepts.bindings.push(ConceptBinding {
            id: TypeId::new(),
            entity_type_id: mountain_type_id,
            concept_id: setup.concept_id,
            concept_role_id: setup.terrain_role_id,
            property_bindings: vec![],
        });
    }

    // Add unconditional Block relation.
    {
        let mut relations = app.world_mut().resource_mut::<RelationRegistry>();
        relations.relations.push(Relation {
            id: TypeId::new(),
            name: "Mountain Impassable".to_string(),
            concept_id: setup.concept_id,
            subject_role_id: setup.traveler_role_id,
            object_role_id: setup.terrain_role_id,
            trigger: RelationTrigger::OnEnter,
            effect: RelationEffect::Block {
                condition: Some(ConstraintExpr::IsType {
                    role_id: setup.terrain_role_id,
                    entity_type_id: mountain_type_id,
                }),
            },
        });
    }

    // Spawn tiles: mostly Plains, but Mountain at (1, 0).
    let radius: u32 = 2;
    let radius_i = radius as i32;
    for q in -radius_i..=radius_i {
        for r in -radius_i..=radius_i {
            if (q + r).unsigned_abs() > radius {
                continue;
            }
            let is_mountain = q == 1 && r == 0;
            let type_id = if is_mountain {
                mountain_type_id
            } else {
                setup.tile_type_id
            };
            let mut properties = HashMap::new();
            if !is_mountain {
                properties.insert(setup.cost_prop_id, PropertyValue::Int(1));
            }
            app.world_mut().spawn((
                HexTile,
                HexPosition::new(q, r),
                EntityData {
                    entity_type_id: type_id,
                    properties,
                },
            ));
        }
    }

    // Spawn unit at origin with budget = 4.
    let mut unit_props = HashMap::new();
    unit_props.insert(setup.budget_prop_id, PropertyValue::Int(4));
    let unit_entity = spawn_unit(
        &mut app,
        0,
        0,
        EntityData {
            entity_type_id: setup.unit_type_id,
            properties: unit_props,
        },
    );

    app.world_mut().resource_mut::<SelectedUnit>().entity = Some(unit_entity);
    app.update();

    let valid_moves = app.world().resource::<ValidMoveSet>();

    // Mountain at (1, 0) should NOT be in valid positions.
    let mountain_pos = HexPosition::new(1, 0);
    assert!(
        !valid_moves.valid_positions.contains(&mountain_pos),
        "Mountain tile should not be in valid_positions"
    );

    // (1, 0) should appear in blocked_explanations.
    assert!(
        valid_moves.blocked_explanations.contains_key(&mountain_pos),
        "Mountain position should have blocked explanations"
    );
}

/// SC-9: Positions outside `map_radius` are never in `valid_positions`.
#[test]
fn valid_moves_respect_grid_bounds() {
    let mut app = test_app();
    // map_radius is 3 (set in test_app).

    // No ontology constraints: free movement.
    spawn_hex_grid(&mut app, 3, TypeId::new());

    let unit_type_id = TypeId::new();
    let unit_entity = spawn_unit(
        &mut app,
        0,
        0,
        EntityData {
            entity_type_id: unit_type_id,
            properties: HashMap::new(),
        },
    );

    app.world_mut().resource_mut::<SelectedUnit>().entity = Some(unit_entity);
    app.update();

    let valid_moves = app.world().resource::<ValidMoveSet>();
    let map_radius = 3u32;

    for pos in &valid_moves.valid_positions {
        let q = pos.q.unsigned_abs();
        let r = pos.r.unsigned_abs();
        let s = (pos.q + pos.r).unsigned_abs();
        assert!(
            q.max(r).max(s) <= map_radius,
            "Position ({}, {}) is outside map bounds (radius {})",
            pos.q,
            pos.r,
            map_radius
        );
    }
}

/// When no ontology constraints exist, all positions within grid bounds
/// should be valid (0.3.0 backward compat).
#[test]
fn free_movement_when_no_constraints() {
    let mut app = test_app();
    // Default: empty concept/relation/constraint registries.

    spawn_hex_grid(&mut app, 3, TypeId::new());

    let unit_type_id = TypeId::new();
    let unit_entity = spawn_unit(
        &mut app,
        0,
        0,
        EntityData {
            entity_type_id: unit_type_id,
            properties: HashMap::new(),
        },
    );

    app.world_mut().resource_mut::<SelectedUnit>().entity = Some(unit_entity);
    app.update();

    let valid_moves = app.world().resource::<ValidMoveSet>();

    // Count expected tiles: 3-radius hex grid = 1 + 6*(1+2+3) = 37 tiles.
    // The unit's own position is not in valid_positions (already there),
    // so expected = 37 - 1 = 36.
    // Actually count all in-bounds positions minus the origin.
    let mut expected_count: usize = 0;
    for q in -3i32..=3 {
        for r in -3i32..=3 {
            if (q + r).unsigned_abs() <= 3 && !(q == 0 && r == 0) {
                expected_count += 1;
            }
        }
    }

    assert_eq!(
        valid_moves.valid_positions.len(),
        expected_count,
        "All in-bounds positions (except unit's own) should be valid with no constraints"
    );
}

// =========================================================================
// CRT Resolution Tests (0.9.0)
// =========================================================================

use crate::contracts::mechanics::{
    CombatModifierDefinition, CombatOutcome, CombatResultsTable, CrtColumn, CrtColumnType, CrtRow,
    ModifierSource, OutcomeEffect,
};
use crate::contracts::mechanics::{
    apply_column_shift, calculate_differential, calculate_odds_ratio,
    evaluate_modifiers_prioritized, find_crt_column, find_crt_row, resolve_crt,
};

/// Helper: create a standard odds-ratio CRT for testing.
fn test_odds_crt() -> CombatResultsTable {
    CombatResultsTable {
        id: TypeId::new(),
        name: "Test CRT".to_string(),
        columns: vec![
            CrtColumn {
                label: "1:2".to_string(),
                column_type: CrtColumnType::OddsRatio,
                threshold: 0.5,
            },
            CrtColumn {
                label: "1:1".to_string(),
                column_type: CrtColumnType::OddsRatio,
                threshold: 1.0,
            },
            CrtColumn {
                label: "2:1".to_string(),
                column_type: CrtColumnType::OddsRatio,
                threshold: 2.0,
            },
            CrtColumn {
                label: "3:1".to_string(),
                column_type: CrtColumnType::OddsRatio,
                threshold: 3.0,
            },
            CrtColumn {
                label: "4:1".to_string(),
                column_type: CrtColumnType::OddsRatio,
                threshold: 4.0,
            },
        ],
        rows: vec![
            CrtRow {
                label: "1".to_string(),
                die_value_min: 1,
                die_value_max: 1,
            },
            CrtRow {
                label: "2".to_string(),
                die_value_min: 2,
                die_value_max: 2,
            },
            CrtRow {
                label: "3".to_string(),
                die_value_min: 3,
                die_value_max: 3,
            },
            CrtRow {
                label: "4".to_string(),
                die_value_min: 4,
                die_value_max: 4,
            },
            CrtRow {
                label: "5".to_string(),
                die_value_min: 5,
                die_value_max: 5,
            },
            CrtRow {
                label: "6".to_string(),
                die_value_min: 6,
                die_value_max: 6,
            },
        ],
        outcomes: vec![
            // Row 1 (die=1): worst outcomes for attacker
            vec![
                CombatOutcome {
                    label: "AE".to_string(),
                    effect: Some(OutcomeEffect::AttackerEliminated),
                },
                CombatOutcome {
                    label: "AE".to_string(),
                    effect: Some(OutcomeEffect::AttackerEliminated),
                },
                CombatOutcome {
                    label: "AR".to_string(),
                    effect: Some(OutcomeEffect::AttackerStepLoss { steps: 1 }),
                },
                CombatOutcome {
                    label: "EX".to_string(),
                    effect: Some(OutcomeEffect::Exchange {
                        attacker_steps: 1,
                        defender_steps: 1,
                    }),
                },
                CombatOutcome {
                    label: "DR".to_string(),
                    effect: Some(OutcomeEffect::Retreat { hexes: 1 }),
                },
            ],
            // Row 2 (die=2)
            vec![
                CombatOutcome {
                    label: "AE".to_string(),
                    effect: Some(OutcomeEffect::AttackerEliminated),
                },
                CombatOutcome {
                    label: "AR".to_string(),
                    effect: Some(OutcomeEffect::AttackerStepLoss { steps: 1 }),
                },
                CombatOutcome {
                    label: "EX".to_string(),
                    effect: Some(OutcomeEffect::Exchange {
                        attacker_steps: 1,
                        defender_steps: 1,
                    }),
                },
                CombatOutcome {
                    label: "DR".to_string(),
                    effect: Some(OutcomeEffect::Retreat { hexes: 1 }),
                },
                CombatOutcome {
                    label: "DE".to_string(),
                    effect: Some(OutcomeEffect::DefenderEliminated),
                },
            ],
            // Row 3 (die=3)
            vec![
                CombatOutcome {
                    label: "AR".to_string(),
                    effect: None,
                },
                CombatOutcome {
                    label: "EX".to_string(),
                    effect: None,
                },
                CombatOutcome {
                    label: "DR".to_string(),
                    effect: None,
                },
                CombatOutcome {
                    label: "DR".to_string(),
                    effect: None,
                },
                CombatOutcome {
                    label: "DE".to_string(),
                    effect: None,
                },
            ],
            // Row 4 (die=4)
            vec![
                CombatOutcome {
                    label: "EX".to_string(),
                    effect: None,
                },
                CombatOutcome {
                    label: "NE".to_string(),
                    effect: Some(OutcomeEffect::NoEffect),
                },
                CombatOutcome {
                    label: "DR".to_string(),
                    effect: None,
                },
                CombatOutcome {
                    label: "DE".to_string(),
                    effect: None,
                },
                CombatOutcome {
                    label: "DE".to_string(),
                    effect: None,
                },
            ],
            // Row 5 (die=5)
            vec![
                CombatOutcome {
                    label: "NE".to_string(),
                    effect: Some(OutcomeEffect::NoEffect),
                },
                CombatOutcome {
                    label: "DR".to_string(),
                    effect: None,
                },
                CombatOutcome {
                    label: "DR".to_string(),
                    effect: None,
                },
                CombatOutcome {
                    label: "DE".to_string(),
                    effect: None,
                },
                CombatOutcome {
                    label: "DE".to_string(),
                    effect: None,
                },
            ],
            // Row 6 (die=6): best outcomes for attacker
            vec![
                CombatOutcome {
                    label: "DR".to_string(),
                    effect: Some(OutcomeEffect::Retreat { hexes: 1 }),
                },
                CombatOutcome {
                    label: "DR".to_string(),
                    effect: Some(OutcomeEffect::Retreat { hexes: 2 }),
                },
                CombatOutcome {
                    label: "DE".to_string(),
                    effect: Some(OutcomeEffect::DefenderEliminated),
                },
                CombatOutcome {
                    label: "DE".to_string(),
                    effect: Some(OutcomeEffect::DefenderEliminated),
                },
                CombatOutcome {
                    label: "DE".to_string(),
                    effect: Some(OutcomeEffect::DefenderEliminated),
                },
            ],
        ],
        combat_concept_id: None,
    }
}

// -- Odds Ratio Calculation --

#[test]
fn odds_ratio_basic() {
    let ratio = calculate_odds_ratio(6.0, 2.0);
    assert!((ratio - 3.0).abs() < f64::EPSILON, "6:2 should be 3.0");
}

#[test]
fn odds_ratio_defender_advantage() {
    let ratio = calculate_odds_ratio(2.0, 4.0);
    assert!((ratio - 0.5).abs() < f64::EPSILON, "2:4 should be 0.5");
}

#[test]
fn odds_ratio_zero_defender() {
    let ratio = calculate_odds_ratio(5.0, 0.0);
    assert!(ratio.is_infinite(), "0 defender should produce infinity");
}

#[test]
fn odds_ratio_equal_strength() {
    let ratio = calculate_odds_ratio(4.0, 4.0);
    assert!((ratio - 1.0).abs() < f64::EPSILON, "4:4 should be 1.0");
}

// -- Differential Calculation --

#[test]
fn differential_basic() {
    let diff = calculate_differential(8.0, 3.0);
    assert!((diff - 5.0).abs() < f64::EPSILON, "8-3 should be 5.0");
}

#[test]
fn differential_negative() {
    let diff = calculate_differential(2.0, 7.0);
    assert!((diff - (-5.0)).abs() < f64::EPSILON, "2-7 should be -5.0");
}

// -- Column Lookup --

#[test]
fn column_lookup_exact_match() {
    let crt = test_odds_crt();
    // Attacker 6, defender 2 = ratio 3.0 -> "3:1" column (index 3)
    let col = find_crt_column(6.0, 2.0, &crt.columns);
    assert_eq!(col, Some(3), "3:1 ratio should match column index 3");
}

#[test]
fn column_lookup_between_columns() {
    let crt = test_odds_crt();
    // Attacker 5, defender 2 = ratio 2.5 -> between "2:1" (2.0) and "3:1" (3.0)
    // Should match "2:1" (the highest column whose threshold is met)
    let col = find_crt_column(5.0, 2.0, &crt.columns);
    assert_eq!(col, Some(2), "2.5 ratio should match 2:1 column (index 2)");
}

#[test]
fn column_lookup_below_minimum() {
    let crt = test_odds_crt();
    // Attacker 1, defender 10 = ratio 0.1 -> below 1:2 threshold (0.5)
    let col = find_crt_column(1.0, 10.0, &crt.columns);
    assert_eq!(col, None, "0.1 ratio should match no column");
}

#[test]
fn column_lookup_above_maximum() {
    let crt = test_odds_crt();
    // Attacker 50, defender 1 = ratio 50.0 -> exceeds all thresholds
    let col = find_crt_column(50.0, 1.0, &crt.columns);
    assert_eq!(
        col,
        Some(4),
        "50:1 ratio should match the last column (index 4)"
    );
}

#[test]
fn column_lookup_empty_columns() {
    let col = find_crt_column(5.0, 1.0, &[]);
    assert_eq!(col, None, "Empty columns should return None");
}

#[test]
fn column_lookup_single_column() {
    let columns = vec![CrtColumn {
        label: "1:1".to_string(),
        column_type: CrtColumnType::OddsRatio,
        threshold: 1.0,
    }];
    let col = find_crt_column(3.0, 1.0, &columns);
    assert_eq!(col, Some(0), "Single column should match if threshold met");

    let col = find_crt_column(0.5, 1.0, &columns);
    assert_eq!(
        col, None,
        "Single column should not match if below threshold"
    );
}

#[test]
fn column_lookup_mixed_types() {
    // A CRT with both ratio and differential columns
    let columns = vec![
        CrtColumn {
            label: "1:2".to_string(),
            column_type: CrtColumnType::OddsRatio,
            threshold: 0.5,
        },
        CrtColumn {
            label: "1:1".to_string(),
            column_type: CrtColumnType::OddsRatio,
            threshold: 1.0,
        },
        CrtColumn {
            label: "+2".to_string(),
            column_type: CrtColumnType::Differential,
            threshold: 2.0,
        },
        CrtColumn {
            label: "+5".to_string(),
            column_type: CrtColumnType::Differential,
            threshold: 5.0,
        },
    ];

    // Attacker 6, defender 3: ratio=2.0 (meets 1:2 and 1:1), diff=3.0 (meets +2)
    let col = find_crt_column(6.0, 3.0, &columns);
    assert_eq!(
        col,
        Some(2),
        "6 vs 3: ratio 2.0 meets cols 0,1; diff 3.0 meets col 2; best is 2"
    );

    // Attacker 10, defender 3: ratio=3.33 (meets 0,1), diff=7.0 (meets 2,3)
    let col = find_crt_column(10.0, 3.0, &columns);
    assert_eq!(
        col,
        Some(3),
        "10 vs 3: diff 7.0 meets +5 threshold at col 3"
    );
}

// -- Row Lookup --

#[test]
fn row_lookup_exact_value() {
    let crt = test_odds_crt();
    let row = find_crt_row(3, &crt.rows);
    assert_eq!(row, Some(2), "Die roll 3 should match row index 2");
}

#[test]
fn row_lookup_range() {
    let rows = vec![
        CrtRow {
            label: "1-2".to_string(),
            die_value_min: 1,
            die_value_max: 2,
        },
        CrtRow {
            label: "3-4".to_string(),
            die_value_min: 3,
            die_value_max: 4,
        },
        CrtRow {
            label: "5-6".to_string(),
            die_value_min: 5,
            die_value_max: 6,
        },
    ];
    assert_eq!(find_crt_row(1, &rows), Some(0));
    assert_eq!(find_crt_row(2, &rows), Some(0));
    assert_eq!(find_crt_row(3, &rows), Some(1));
    assert_eq!(find_crt_row(5, &rows), Some(2));
    assert_eq!(find_crt_row(6, &rows), Some(2));
}

#[test]
fn row_lookup_no_match() {
    let crt = test_odds_crt();
    let row = find_crt_row(7, &crt.rows);
    assert_eq!(row, None, "Die roll 7 should not match any 1d6 row");
}

#[test]
fn row_lookup_empty() {
    let row = find_crt_row(1, &[]);
    assert_eq!(row, None, "Empty rows should return None");
}

// -- Full CRT Resolution --

#[test]
fn resolve_crt_basic() {
    let crt = test_odds_crt();
    // 6 vs 2 = 3:1 (col 3), die roll 6 (row 5) -> "DE"
    let result = resolve_crt(&crt, 6.0, 2.0, 6);
    assert!(result.is_some(), "Should resolve successfully");
    let res = result.expect("resolve");
    assert_eq!(res.column_index, 3);
    assert_eq!(res.row_index, 5);
    assert_eq!(res.column_label, "3:1");
    assert_eq!(res.outcome.label, "DE");
}

#[test]
fn resolve_crt_attacker_eliminated() {
    let crt = test_odds_crt();
    // 1 vs 2 = 0.5 (col 0, "1:2"), die roll 1 (row 0) -> "AE"
    let result = resolve_crt(&crt, 1.0, 2.0, 1);
    let res = result.expect("resolve");
    assert_eq!(res.outcome.label, "AE");
    assert!(matches!(
        res.outcome.effect,
        Some(OutcomeEffect::AttackerEliminated)
    ));
}

#[test]
fn resolve_crt_no_match_column() {
    let crt = test_odds_crt();
    // ratio too low to match any column
    let result = resolve_crt(&crt, 1.0, 10.0, 3);
    assert!(result.is_none(), "Should fail when no column matches");
}

#[test]
fn resolve_crt_no_match_row() {
    let crt = test_odds_crt();
    // valid column but invalid die roll
    let result = resolve_crt(&crt, 6.0, 2.0, 7);
    assert!(result.is_none(), "Should fail when no row matches");
}

#[test]
fn resolve_crt_empty() {
    let crt = CombatResultsTable::default();
    let result = resolve_crt(&crt, 6.0, 2.0, 3);
    assert!(result.is_none(), "Empty CRT should return None");
}

// -- Modifier Evaluation --

#[test]
fn modifiers_priority_order() {
    let modifiers = vec![
        CombatModifierDefinition {
            id: TypeId::new(),
            name: "Forest".to_string(),
            source: ModifierSource::DefenderTerrain,
            column_shift: -2,
            priority: 10,
            cap: None,
            terrain_type_filter: None,
        },
        CombatModifierDefinition {
            id: TypeId::new(),
            name: "Combined Arms".to_string(),
            source: ModifierSource::Custom("combined".to_string()),
            column_shift: 1,
            priority: 5,
            cap: None,
            terrain_type_filter: None,
        },
    ];

    let (total, display) = evaluate_modifiers_prioritized(&modifiers, 7);
    assert_eq!(total, -1, "Forest(-2) + Combined Arms(+1) = -1");
    assert_eq!(display[0].0, "Forest", "Higher priority evaluated first");
    assert_eq!(
        display[1].0, "Combined Arms",
        "Lower priority evaluated second"
    );
}

#[test]
fn modifiers_cap_limits_total() {
    let modifiers = vec![
        CombatModifierDefinition {
            id: TypeId::new(),
            name: "Terrain Cap".to_string(),
            source: ModifierSource::DefenderTerrain,
            column_shift: -3,
            priority: 20,
            cap: Some(2), // cap total to [-2, +2]
            terrain_type_filter: None,
        },
        CombatModifierDefinition {
            id: TypeId::new(),
            name: "Extra Shift".to_string(),
            source: ModifierSource::Custom("extra".to_string()),
            column_shift: -1,
            priority: 10,
            cap: None,
            terrain_type_filter: None,
        },
    ];

    let (total, _) = evaluate_modifiers_prioritized(&modifiers, 7);
    // Terrain Cap: shift -3, then cap to [-2, +2] -> total = -2
    // Extra Shift: shift -1 -> total = -3
    // No further cap -> -3
    // Clamp to column bounds: -3 clamped to [-6, 6] -> -3
    assert_eq!(
        total, -3,
        "Cap applied after first modifier, then second shifts further"
    );
}

#[test]
fn modifiers_clamp_to_column_bounds() {
    let modifiers = vec![CombatModifierDefinition {
        id: TypeId::new(),
        name: "Massive Shift".to_string(),
        source: ModifierSource::Custom("big".to_string()),
        column_shift: -100,
        priority: 1,
        cap: None,
        terrain_type_filter: None,
    }];

    let (total, _) = evaluate_modifiers_prioritized(&modifiers, 5);
    assert_eq!(
        total, -4,
        "Shift should be clamped to column bounds (5 columns -> max shift 4)"
    );
}

#[test]
fn modifiers_empty_list() {
    let (total, display) = evaluate_modifiers_prioritized(&[], 5);
    assert_eq!(total, 0, "No modifiers should produce zero shift");
    assert!(display.is_empty());
}

// -- Column Shift Application --

#[test]
fn apply_shift_positive() {
    let result = apply_column_shift(2, 1, 5);
    assert_eq!(result, 3);
}

#[test]
fn apply_shift_negative() {
    let result = apply_column_shift(2, -1, 5);
    assert_eq!(result, 1);
}

#[test]
fn apply_shift_clamps_high() {
    let result = apply_column_shift(3, 5, 5);
    assert_eq!(result, 4, "Should clamp to max column index");
}

#[test]
fn apply_shift_clamps_low() {
    let result = apply_column_shift(1, -5, 5);
    assert_eq!(result, 0, "Should clamp to zero");
}

#[test]
fn apply_shift_zero_columns() {
    let result = apply_column_shift(0, 1, 0);
    assert_eq!(result, 0, "Zero columns should return 0");
}

// =========================================================================
// Phase Advancement Tests (0.9.0)
// =========================================================================

use crate::contracts::mechanics::PlayerOrder;
use crate::contracts::mechanics::{Phase, PhaseType, TurnState, TurnStructure};
use crate::rules_engine::systems::advance_phase;
use crate::rules_engine::systems::start_turn_sequence;

fn test_turn_structure() -> TurnStructure {
    TurnStructure {
        phases: vec![
            Phase {
                id: crate::contracts::game_system::TypeId::new(),
                name: "Movement".to_string(),
                phase_type: PhaseType::Movement,
                description: String::new(),
            },
            Phase {
                id: crate::contracts::game_system::TypeId::new(),
                name: "Combat".to_string(),
                phase_type: PhaseType::Combat,
                description: String::new(),
            },
            Phase {
                id: crate::contracts::game_system::TypeId::new(),
                name: "Supply".to_string(),
                phase_type: PhaseType::Admin,
                description: String::new(),
            },
        ],
        player_order: PlayerOrder::Alternating,
    }
}

#[test]
fn start_turn_initializes_to_first_phase() {
    let mut state = TurnState::default();
    let structure = test_turn_structure();

    let event = start_turn_sequence(&mut state, &structure);

    assert!(event.is_some());
    let event = event.expect("should have event");
    assert_eq!(event.turn_number, 1);
    assert_eq!(event.phase_index, 0);
    assert_eq!(event.phase_name, "Movement");
    assert_eq!(event.phase_type, PhaseType::Movement);
    assert!(state.is_active);
    assert_eq!(state.turn_number, 1);
    assert_eq!(state.current_phase_index, 0);
}

#[test]
fn start_turn_empty_structure_returns_none() {
    let mut state = TurnState::default();
    let structure = TurnStructure {
        phases: Vec::new(),
        player_order: PlayerOrder::Alternating,
    };

    let event = start_turn_sequence(&mut state, &structure);
    assert!(event.is_none());
}

#[test]
fn advance_phase_increments_index() {
    let mut state = TurnState {
        turn_number: 1,
        current_phase_index: 0,
        is_active: true,
    };
    let structure = test_turn_structure();

    let event = advance_phase(&mut state, &structure);

    assert!(event.is_some());
    let event = event.expect("should have event");
    assert_eq!(event.phase_index, 1);
    assert_eq!(event.phase_name, "Combat");
    assert_eq!(event.phase_type, PhaseType::Combat);
    assert_eq!(state.turn_number, 1);
}

#[test]
fn advance_phase_wraps_to_next_turn() {
    let mut state = TurnState {
        turn_number: 1,
        current_phase_index: 2, // Last phase (Supply, index 2 of 3)
        is_active: true,
    };
    let structure = test_turn_structure();

    let event = advance_phase(&mut state, &structure);

    assert!(event.is_some());
    let event = event.expect("should have event");
    assert_eq!(event.turn_number, 2);
    assert_eq!(event.phase_index, 0);
    assert_eq!(event.phase_name, "Movement");
    assert_eq!(state.turn_number, 2);
    assert_eq!(state.current_phase_index, 0);
}

#[test]
fn advance_phase_multiple_turns() {
    let mut state = TurnState {
        turn_number: 1,
        current_phase_index: 0,
        is_active: true,
    };
    let structure = test_turn_structure();

    // Advance through all 3 phases + into turn 2
    advance_phase(&mut state, &structure); // -> phase 1 (Combat)
    advance_phase(&mut state, &structure); // -> phase 2 (Supply)
    advance_phase(&mut state, &structure); // -> turn 2, phase 0 (Movement)
    let event = advance_phase(&mut state, &structure); // -> turn 2, phase 1 (Combat)

    let event = event.expect("should have event");
    assert_eq!(event.turn_number, 2);
    assert_eq!(event.phase_index, 1);
    assert_eq!(event.phase_name, "Combat");
}

#[test]
fn advance_phase_empty_structure_returns_none() {
    let mut state = TurnState {
        turn_number: 1,
        current_phase_index: 0,
        is_active: true,
    };
    let structure = TurnStructure {
        phases: Vec::new(),
        player_order: PlayerOrder::Alternating,
    };

    let event = advance_phase(&mut state, &structure);
    assert!(event.is_none());
}

#[test]
fn advance_phase_single_phase_wraps_every_advance() {
    let structure = TurnStructure {
        phases: vec![Phase {
            id: crate::contracts::game_system::TypeId::new(),
            name: "Only Phase".to_string(),
            phase_type: PhaseType::Combat,
            description: String::new(),
        }],
        player_order: PlayerOrder::Alternating,
    };
    let mut state = TurnState {
        turn_number: 1,
        current_phase_index: 0,
        is_active: true,
    };

    let event = advance_phase(&mut state, &structure);
    let event = event.expect("should have event");
    assert_eq!(event.turn_number, 2, "Should wrap to turn 2");
    assert_eq!(event.phase_index, 0);

    let event = advance_phase(&mut state, &structure);
    let event = event.expect("should have event");
    assert_eq!(event.turn_number, 3, "Should wrap to turn 3");
}
