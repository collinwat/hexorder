//! Unit tests for the `rules_engine` feature.

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
/// should be valid (M3 backward compat).
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
