//! Systems for the unit feature plugin.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::contracts::editor_ui::EditorTool;
use crate::contracts::game_system::{
    ActiveUnitType, PropertyValue, SelectedUnit, UnitData, UnitInstance, UnitPlacedEvent,
    UnitTypeRegistry,
};
use crate::contracts::hex_grid::{HexGridConfig, HexMoveEvent, HexPosition, HexSelectedEvent};

use super::components::{UnitMaterials, UnitMesh};

/// Height offset for unit tokens above the hex tile surface.
const UNIT_Y_OFFSET: f32 = 0.25;

// ---------------------------------------------------------------------------
// Startup
// ---------------------------------------------------------------------------

/// Creates materials for all registered unit types and a shared cylinder mesh.
pub fn setup_unit_visuals(
    mut commands: Commands,
    registry: Res<UnitTypeRegistry>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut unit_materials = HashMap::new();
    for ut in &registry.types {
        let handle = materials.add(StandardMaterial {
            base_color: ut.color,
            ..default()
        });
        unit_materials.insert(ut.id, handle);
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
#[allow(clippy::too_many_arguments)]
pub fn handle_unit_placement(
    trigger: On<HexSelectedEvent>,
    tool: Res<EditorTool>,
    active_unit: Res<ActiveUnitType>,
    registry: Res<UnitTypeRegistry>,
    config: Res<HexGridConfig>,
    unit_materials: Res<UnitMaterials>,
    unit_mesh: Res<UnitMesh>,
    mut commands: Commands,
) {
    if *tool != EditorTool::Place {
        return;
    }

    let Some(active_id) = active_unit.unit_type_id else {
        return;
    };

    let Some(unit_type) = registry.get(active_id) else {
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

    // Build default properties for this unit type.
    let default_properties: HashMap<_, _> = unit_type
        .properties
        .iter()
        .map(|pd| (pd.id, PropertyValue::default_for(&pd.property_type)))
        .collect();

    let Some(material) = unit_materials.get(active_id) else {
        return;
    };

    // Spawn unit entity.
    let entity = commands
        .spawn((
            UnitInstance,
            HexPosition::new(pos.q, pos.r),
            UnitData {
                unit_type_id: active_id,
                properties: default_properties,
            },
            Mesh3d(unit_mesh.handle.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(world_pos.x, UNIT_Y_OFFSET, world_pos.y),
        ))
        .id();

    commands.trigger(UnitPlacedEvent {
        entity,
        position: pos,
        unit_type_id: active_id,
    });
}

/// Handles unit selection and movement in Select mode.
///
/// - Click hex with unit → select it
/// - Click same hex as selected unit → deselect
/// - Click different hex while unit selected → move unit there
pub fn handle_unit_interaction(
    trigger: On<HexSelectedEvent>,
    tool: Res<EditorTool>,
    mut selected_unit: ResMut<SelectedUnit>,
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

/// Deletes the selected unit when the editor UI sets
/// `SelectedUnit.entity` to a special "delete requested" signal.
///
/// The editor UI sets a `DeleteUnitRequested` resource flag; this system
/// checks it and despawns the entity.
/// Clears the `SelectedUnit` resource if the selected entity no longer exists.
/// Actual unit deletion is performed by the `editor_ui` via `commands.entity().despawn()`.
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

/// Updates material colors when the `UnitTypeRegistry` changes.
pub fn sync_unit_materials(
    registry: Res<UnitTypeRegistry>,
    mut unit_materials: ResMut<UnitMaterials>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !registry.is_changed() {
        return;
    }

    for ut in &registry.types {
        if let Some(handle) = unit_materials.materials.get(&ut.id) {
            if let Some(mat) = materials.get_mut(handle) {
                mat.base_color = ut.color;
            }
        } else {
            let handle = materials.add(StandardMaterial {
                base_color: ut.color,
                ..default()
            });
            unit_materials.materials.insert(ut.id, handle);
        }
    }

    let valid_ids: HashSet<_> = registry.types.iter().map(|ut| ut.id).collect();
    unit_materials
        .materials
        .retain(|id, _| valid_ids.contains(id));
}

/// Syncs unit material when `UnitData` changes (change detection).
#[allow(clippy::type_complexity)]
pub fn sync_unit_visuals(
    unit_materials: Res<UnitMaterials>,
    mut units: Query<
        (&UnitData, &mut MeshMaterial3d<StandardMaterial>),
        (With<UnitInstance>, Changed<UnitData>),
    >,
) {
    for (unit_data, mut material) in &mut units {
        if let Some(handle) = unit_materials.get(unit_data.unit_type_id) {
            material.0 = handle.clone();
        }
    }
}
