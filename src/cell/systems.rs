//! Systems for the cell feature plugin.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::contracts::editor_ui::{EditorTool, PaintPreview};
use crate::contracts::game_system::{ActiveCellType, CellData, CellTypeRegistry, PropertyValue};
use crate::contracts::hex_grid::{HexPosition, HexSelectedEvent, HexTile, TileBaseMaterial};

use super::components::CellMaterials;

/// Creates material handles for each cell type in the registry and stores
/// them in the `CellMaterials` resource.
pub fn setup_cell_materials(
    mut commands: Commands,
    registry: Res<CellTypeRegistry>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut cell_materials = HashMap::new();

    for vt in &registry.types {
        let handle = materials.add(StandardMaterial {
            base_color: vt.color,
            unlit: true,
            ..default()
        });
        cell_materials.insert(vt.id, handle);
    }

    commands.insert_resource(CellMaterials {
        materials: cell_materials,
    });
}

/// Attaches a default `CellData` component to all hex tiles that do not
/// already have one. Uses the first cell type in the registry.
pub fn assign_default_cell_data(
    mut commands: Commands,
    registry: Res<CellTypeRegistry>,
    tiles: Query<Entity, (With<HexTile>, Without<CellData>)>,
) {
    let Some(first_type) = registry.first() else {
        return;
    };

    let default_properties: HashMap<_, _> = first_type
        .properties
        .iter()
        .map(|pd| (pd.id, PropertyValue::default_for(&pd.property_type)))
        .collect();

    for entity in &tiles {
        commands.entity(entity).insert(CellData {
            cell_type_id: first_type.id,
            properties: default_properties.clone(),
        });
    }
}

/// Observer callback: when a hex tile is selected (clicked), paint the active
/// cell type onto that tile if the editor is in Paint mode.
pub fn paint_cell(
    trigger: On<HexSelectedEvent>,
    tool: Res<EditorTool>,
    active: Res<ActiveCellType>,
    registry: Res<CellTypeRegistry>,
    mut tiles: Query<(&HexPosition, &mut CellData), With<HexTile>>,
) {
    if *tool != EditorTool::Paint {
        return;
    }

    let Some(active_id) = active.cell_type_id else {
        return;
    };

    let Some(cell_type) = registry.get(active_id) else {
        return;
    };

    let default_properties: HashMap<_, _> = cell_type
        .properties
        .iter()
        .map(|pd| (pd.id, PropertyValue::default_for(&pd.property_type)))
        .collect();

    let event = trigger.event();
    for (pos, mut cell_data) in &mut tiles {
        if *pos == event.position {
            cell_data.cell_type_id = active_id;
            cell_data.properties.clone_from(&default_properties);
        }
    }
}

/// Syncs the visual material of hex tiles to match their current cell type.
/// Updates both the rendered material and `TileBaseMaterial` so that
/// hover/selection highlighting can restore the correct cell color.
/// Uses change detection to only update tiles whose `CellData` has changed.
#[allow(clippy::type_complexity)]
pub fn sync_cell_visuals(
    cell_materials: Res<CellMaterials>,
    mut tiles: Query<
        (
            &CellData,
            &mut MeshMaterial3d<StandardMaterial>,
            &mut TileBaseMaterial,
        ),
        (With<HexTile>, Changed<CellData>),
    >,
) {
    for (cell_data, mut material, mut base) in &mut tiles {
        if let Some(handle) = cell_materials.get(cell_data.cell_type_id) {
            material.0 = handle.clone();
            base.0 = handle.clone();
        }
    }
}

/// Updates the `CellMaterials` resource when cell types are added, removed,
/// or have their colors changed in the registry. Only runs when the registry
/// resource has been mutated.
pub fn sync_cell_materials(
    registry: Res<CellTypeRegistry>,
    mut cell_materials: ResMut<CellMaterials>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !registry.is_changed() {
        return;
    }

    for vt in &registry.types {
        if let Some(handle) = cell_materials.materials.get(&vt.id) {
            // Update color in place if the type already has a material.
            if let Some(mat) = materials.get_mut(handle) {
                mat.base_color = vt.color;
            }
        } else {
            // New type — create a material for it.
            let handle = materials.add(StandardMaterial {
                base_color: vt.color,
                unlit: true,
                ..default()
            });
            cell_materials.materials.insert(vt.id, handle);
        }
    }

    // Remove materials for cell types that no longer exist.
    let valid_ids: std::collections::HashSet<_> = registry.types.iter().map(|vt| vt.id).collect();
    cell_materials
        .materials
        .retain(|id, _| valid_ids.contains(id));
}

/// Keeps the `PaintPreview` resource in sync with the currently active cell type.
/// Runs whenever `ActiveCellType` or `CellMaterials` changes.
pub fn update_paint_preview(
    active: Res<ActiveCellType>,
    cell_materials: Res<CellMaterials>,
    mut preview: ResMut<PaintPreview>,
) {
    if !active.is_changed() && !cell_materials.is_changed() {
        return;
    }

    preview.material = active
        .cell_type_id
        .and_then(|id| cell_materials.get(id).cloned());
}
