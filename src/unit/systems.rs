//! Systems for the unit plugin.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::contracts::editor_ui::EditorTool;
use crate::contracts::game_system::{
    ActiveTokenType, EntityData, EntityRole, EntityTypeRegistry, PropertyValue, SelectedUnit,
    UnitInstance, UnitPlacedEvent,
};
use crate::contracts::hex_grid::{HexGridConfig, HexMoveEvent, HexPosition, HexSelectedEvent};
use crate::contracts::undo_redo::{PlaceUnitCommand, UndoStack};
use crate::contracts::validation::ValidMoveSet;

use super::components::{UnitMaterials, UnitMesh};

/// Height offset for unit tokens above the hex tile surface.
const UNIT_Y_OFFSET: f32 = 0.25;

// ---------------------------------------------------------------------------
// Startup
// ---------------------------------------------------------------------------

/// Creates materials for all registered Token entity types and a shared cylinder mesh.
pub fn setup_unit_visuals(
    mut commands: Commands,
    registry: Res<EntityTypeRegistry>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut unit_materials = HashMap::new();
    for et in registry.types_by_role(EntityRole::Token) {
        let handle = materials.add(StandardMaterial {
            base_color: et.color,
            ..default()
        });
        unit_materials.insert(et.id, handle);
    }
    commands.insert_resource(UnitMaterials {
        materials: unit_materials,
    });

    let mesh_handle = meshes.add(Cylinder::new(0.3, 0.4));
    commands.insert_resource(UnitMesh {
        handle: mesh_handle,
    });
}

// ---------------------------------------------------------------------------
// Observers
// ---------------------------------------------------------------------------

/// Places a unit on the clicked hex tile when in Place mode.
/// Records a `PlaceUnitCommand` on the undo stack for reversibility.
#[allow(clippy::too_many_arguments)]
pub fn handle_unit_placement(
    trigger: On<HexSelectedEvent>,
    tool: Res<EditorTool>,
    active_unit: Res<ActiveTokenType>,
    registry: Res<EntityTypeRegistry>,
    config: Res<HexGridConfig>,
    unit_materials: Res<UnitMaterials>,
    unit_mesh: Res<UnitMesh>,
    mut undo_stack: ResMut<UndoStack>,
    mut commands: Commands,
) {
    if *tool != EditorTool::Place {
        return;
    }

    let Some(active_id) = active_unit.entity_type_id else {
        return;
    };

    let Some(entity_type) = registry.get(active_id) else {
        return;
    };

    let event = trigger.event();
    let pos = event.position;

    // Verify position is within grid bounds.
    let hex = pos.to_hex();
    if hex.unsigned_distance_to(hexx::Hex::ZERO) > config.map_radius {
        return;
    }

    // Compute world position from hex coordinates.
    let world_pos = config.layout.hex_to_world_pos(hex);

    // Build default properties for this entity type.
    let default_properties: HashMap<_, _> = entity_type
        .properties
        .iter()
        .map(|pd| (pd.id, PropertyValue::default_for(&pd.property_type)))
        .collect();

    let Some(material) = unit_materials.get(active_id) else {
        return;
    };

    let transform = Transform::from_xyz(world_pos.x, UNIT_Y_OFFSET, world_pos.y);
    let entity_data = EntityData {
        entity_type_id: active_id,
        properties: default_properties,
    };

    // Spawn unit entity.
    let entity = commands
        .spawn((
            UnitInstance,
            HexPosition::new(pos.q, pos.r),
            entity_data.clone(),
            Mesh3d(unit_mesh.handle.clone()),
            MeshMaterial3d(material.clone()),
            transform,
        ))
        .id();

    // Record for undo.
    let label = format!("Place {} at ({}, {})", entity_type.name, pos.q, pos.r);
    undo_stack.record(Box::new(PlaceUnitCommand {
        entity: Some(entity),
        position: pos,
        entity_data,
        mesh: unit_mesh.handle.clone(),
        material: material.clone(),
        transform,
        label,
    }));

    commands.trigger(UnitPlacedEvent {
        entity,
        position: pos,
        entity_type_id: active_id,
    });
}

/// Handles unit selection and movement in Select mode.
///
/// - Click hex with unit → select it
/// - Click same hex as selected unit → deselect
/// - Click different hex while unit selected → move unit there
///
/// If `ValidMoveSet` has valid positions, only allows movement to those
/// positions. If `ValidMoveSet` is empty (no constraints), all in-bounds
/// positions are allowed (backward compatible with 0.3.0).
#[allow(clippy::too_many_arguments)]
pub fn handle_unit_interaction(
    trigger: On<HexSelectedEvent>,
    tool: Res<EditorTool>,
    mut selected_unit: ResMut<SelectedUnit>,
    valid_moves: Option<Res<ValidMoveSet>>,
    config: Res<HexGridConfig>,
    mut units: Query<(Entity, &mut HexPosition, &mut Transform), With<UnitInstance>>,
    mut commands: Commands,
) {
    if *tool != EditorTool::Select {
        return;
    }

    let event = trigger.event();
    let clicked_pos = event.position;

    // Check if there's a unit at the clicked position.
    let unit_at_pos = units
        .iter()
        .find(|(_, pos, _)| **pos == clicked_pos)
        .map(|(e, _, _)| e);

    if let Some(entity) = unit_at_pos {
        if selected_unit.entity == Some(entity) {
            // Clicked same unit → deselect.
            selected_unit.entity = None;
        } else {
            // Clicked a different unit → select it.
            selected_unit.entity = Some(entity);
        }
    } else if let Some(selected_entity) = selected_unit.entity {
        // Clicked empty tile while a unit is selected → move the unit.
        let hex = clicked_pos.to_hex();
        if hex.unsigned_distance_to(hexx::Hex::ZERO) > config.map_radius {
            return;
        }

        // If ValidMoveSet has computed valid positions for this entity,
        // only allow movement to those positions.
        if let Some(moves) = &valid_moves
            && moves.for_entity == Some(selected_entity)
            && !moves.valid_positions.is_empty()
            && !moves.valid_positions.contains(&clicked_pos)
        {
            return;
        }

        let Ok((_, mut pos, mut transform)) = units.get_mut(selected_entity) else {
            // Entity no longer exists — clear selection.
            selected_unit.entity = None;
            return;
        };

        let from = *pos;
        *pos = clicked_pos;

        let world_pos = config.layout.hex_to_world_pos(hex);
        transform.translation = Vec3::new(world_pos.x, UNIT_Y_OFFSET, world_pos.y);

        commands.trigger(HexMoveEvent {
            entity: selected_entity,
            from,
            to: clicked_pos,
        });

        selected_unit.entity = None;
    }
}

// ---------------------------------------------------------------------------
// Update systems
// ---------------------------------------------------------------------------

/// Clears the `SelectedUnit` resource if the selected entity no longer exists.
pub fn delete_selected_unit(
    mut selected_unit: ResMut<SelectedUnit>,
    units: Query<Entity, With<UnitInstance>>,
) {
    if let Some(entity) = selected_unit.entity
        && units.get(entity).is_err()
    {
        selected_unit.entity = None;
    }
}

/// Attaches mesh and material to `UnitInstance` entities that lack them.
/// This covers units spawned by `apply_pending_board_load` which only
/// adds core ECS components (no visuals).
#[allow(clippy::type_complexity)]
pub fn assign_unit_visuals(
    mut commands: Commands,
    unit_mesh: Res<UnitMesh>,
    unit_materials: Res<UnitMaterials>,
    units: Query<(Entity, &EntityData), (With<UnitInstance>, Without<Mesh3d>)>,
) {
    for (entity, entity_data) in &units {
        let Some(material) = unit_materials.get(entity_data.entity_type_id) else {
            continue;
        };
        commands.entity(entity).insert((
            Mesh3d(unit_mesh.handle.clone()),
            MeshMaterial3d(material.clone()),
        ));
    }
}

/// Updates material colors when the `EntityTypeRegistry` changes.
pub fn sync_unit_materials(
    registry: Res<EntityTypeRegistry>,
    mut unit_materials: ResMut<UnitMaterials>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !registry.is_changed() {
        return;
    }

    let token_types = registry.types_by_role(EntityRole::Token);

    for et in &token_types {
        if let Some(handle) = unit_materials.materials.get(&et.id) {
            if let Some(mat) = materials.get_mut(handle) {
                mat.base_color = et.color;
            }
        } else {
            let handle = materials.add(StandardMaterial {
                base_color: et.color,
                ..default()
            });
            unit_materials.materials.insert(et.id, handle);
        }
    }

    let valid_ids: HashSet<_> = token_types.iter().map(|et| et.id).collect();
    unit_materials
        .materials
        .retain(|id, _| valid_ids.contains(id));
}

/// Syncs unit material when `EntityData` changes (change detection).
#[allow(clippy::type_complexity)]
pub fn sync_unit_visuals(
    unit_materials: Res<UnitMaterials>,
    mut units: Query<
        (&EntityData, &mut MeshMaterial3d<StandardMaterial>),
        (With<UnitInstance>, Changed<EntityData>),
    >,
) {
    for (entity_data, mut material) in &mut units {
        if let Some(handle) = unit_materials.get(entity_data.entity_type_id) {
            material.0 = handle.clone();
        }
    }
}
