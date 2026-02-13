//! Systems for the cell feature plugin.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::contracts::editor_ui::{EditorTool, PaintPreview};
use crate::contracts::game_system::{
    ActiveBoardType, EntityData, EntityRole, EntityTypeRegistry, PropertyValue,
};
use crate::contracts::hex_grid::{HexPosition, HexSelectedEvent, HexTile, TileBaseMaterial};

use super::components::CellMaterials;

/// Creates material handles for each `BoardPosition` entity type in the registry
/// and stores them in the `CellMaterials` resource.
pub fn setup_cell_materials(
    mut commands: Commands,
    registry: Res<EntityTypeRegistry>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut cell_materials = HashMap::new();

    for et in registry.types_by_role(EntityRole::BoardPosition) {
        let handle = materials.add(StandardMaterial {
            base_color: et.color,
            unlit: true,
            ..default()
        });
        cell_materials.insert(et.id, handle);
    }

    commands.insert_resource(CellMaterials {
        materials: cell_materials,
    });
}

/// Attaches a default `EntityData` component to all hex tiles that do not
/// already have one. Uses the first `BoardPosition` entity type in the registry.
pub fn assign_default_cell_data(
    mut commands: Commands,
    registry: Res<EntityTypeRegistry>,
    tiles: Query<Entity, (With<HexTile>, Without<EntityData>)>,
) {
    let Some(first_type) = registry.first_by_role(EntityRole::BoardPosition) else {
        return;
    };

    let default_properties: HashMap<_, _> = first_type
        .properties
        .iter()
        .map(|pd| (pd.id, PropertyValue::default_for(&pd.property_type)))
        .collect();

    for entity in &tiles {
        commands.entity(entity).insert(EntityData {
            entity_type_id: first_type.id,
            properties: default_properties.clone(),
        });
    }
}

/// Observer callback: when a hex tile is selected (clicked), paint the active
/// board type onto that tile if the editor is in Paint mode.
pub fn paint_cell(
    trigger: On<HexSelectedEvent>,
    tool: Res<EditorTool>,
    active: Res<ActiveBoardType>,
    registry: Res<EntityTypeRegistry>,
    mut tiles: Query<(&HexPosition, &mut EntityData), With<HexTile>>,
) {
    if *tool != EditorTool::Paint {
        return;
    }

    let Some(active_id) = active.entity_type_id else {
        return;
    };

    let Some(entity_type) = registry.get(active_id) else {
        return;
    };

    let default_properties: HashMap<_, _> = entity_type
        .properties
        .iter()
        .map(|pd| (pd.id, PropertyValue::default_for(&pd.property_type)))
        .collect();

    let event = trigger.event();
    for (pos, mut entity_data) in &mut tiles {
        if *pos == event.position {
            entity_data.entity_type_id = active_id;
            entity_data.properties.clone_from(&default_properties);
        }
    }
}

/// Syncs the visual material of hex tiles to match their current entity type.
/// Updates both the rendered material and `TileBaseMaterial` so that
/// hover/selection highlighting can restore the correct cell color.
/// Uses change detection to only update tiles whose `EntityData` has changed.
#[allow(clippy::type_complexity)]
pub fn sync_cell_visuals(
    cell_materials: Res<CellMaterials>,
    mut tiles: Query<
        (
            &EntityData,
            &mut MeshMaterial3d<StandardMaterial>,
            &mut TileBaseMaterial,
        ),
        (With<HexTile>, Changed<EntityData>),
    >,
) {
    for (entity_data, mut material, mut base) in &mut tiles {
        if let Some(handle) = cell_materials.get(entity_data.entity_type_id) {
            material.0 = handle.clone();
            base.0 = handle.clone();
        }
    }
}

/// Updates the `CellMaterials` resource when `BoardPosition` entity types are
/// added, removed, or have their colors changed in the registry. Only runs
/// when the registry resource has been mutated.
pub fn sync_cell_materials(
    registry: Res<EntityTypeRegistry>,
    mut cell_materials: ResMut<CellMaterials>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !registry.is_changed() {
        return;
    }

    let board_types = registry.types_by_role(EntityRole::BoardPosition);

    for et in &board_types {
        if let Some(handle) = cell_materials.materials.get(&et.id) {
            // Update color in place if the type already has a material.
            if let Some(mat) = materials.get_mut(handle) {
                mat.base_color = et.color;
            }
        } else {
            // New type — create a material for it.
            let handle = materials.add(StandardMaterial {
                base_color: et.color,
                unlit: true,
                ..default()
            });
            cell_materials.materials.insert(et.id, handle);
        }
    }

    // Remove materials for entity types that no longer exist as BoardPosition.
    let valid_ids: std::collections::HashSet<_> = board_types.iter().map(|et| et.id).collect();
    cell_materials
        .materials
        .retain(|id, _| valid_ids.contains(id));
}

/// Keeps the `PaintPreview` resource in sync with the currently active board type.
/// Runs whenever `ActiveBoardType` or `CellMaterials` changes.
pub fn update_paint_preview(
    active: Res<ActiveBoardType>,
    cell_materials: Res<CellMaterials>,
    mut preview: ResMut<PaintPreview>,
) {
    if !active.is_changed() && !cell_materials.is_changed() {
        return;
    }

    preview.material = active
        .entity_type_id
        .and_then(|id| cell_materials.get(id).cloned());
}
