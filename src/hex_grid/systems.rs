//! Systems for the `hex_grid` plugin.

use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use hexx::shapes;

use crate::contracts::editor_ui::{EditorTool, PaintPreview, Selection};
use crate::contracts::game_system::{SelectedUnit, UnitInstance};
use crate::contracts::hex_grid::{
    HexGridConfig, HexPosition, HexSelectedEvent, HexTile, MoveOverlay, MoveOverlayState,
    SelectedHex, TileBaseMaterial,
};
use crate::contracts::validation::ValidMoveSet;

use super::algorithms;
use super::components::{
    HexMaterials, HoverIndicator, HoveredHex, IndicatorMaterials, MultiSelectIndicator,
    OverlayMaterials, SelectIndicator,
};

/// Creates the hex grid configuration resource with default settings.
pub fn setup_grid_config(mut commands: Commands) {
    let layout = hexx::HexLayout {
        orientation: hexx::HexOrientation::Pointy,
        ..hexx::HexLayout::default()
    }
    .with_hex_size(1.0);

    commands.insert_resource(HexGridConfig {
        layout,
        map_radius: 10,
    });

    commands.insert_resource(SelectedHex::default());
    commands.insert_resource(HoveredHex::default());
}

/// Creates the shared default material handle for hex tile rendering.
pub fn setup_materials(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let hex_materials = HexMaterials {
        default: materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.8, 0.8),
            unlit: true,
            ..default()
        }),
    };
    commands.insert_resource(hex_materials);
}

/// Spawns all hex tile entities for the configured grid radius.
pub fn spawn_grid(
    mut commands: Commands,
    config: Res<HexGridConfig>,
    hex_materials: Res<HexMaterials>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Use Bevy's built-in RegularPolygon (6 sides = hexagon) which generates
    // all required mesh attributes. The mesh is created in the XY plane, so we
    // rotate each tile -90 degrees around X to lay it flat on the XZ ground plane.
    let hex_size = config.layout.scale.x.max(config.layout.scale.y);
    // Shrink slightly (0.95) to leave a thin gap between adjacent tiles.
    let mesh_handle = meshes.add(RegularPolygon::new(hex_size * 0.95, 6));

    // Rotation to lay the XY-plane polygon flat on the XZ ground plane.
    let flat_rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);

    for hex in shapes::hexagon(hexx::Hex::ZERO, config.map_radius) {
        let world_pos = config.layout.hex_to_world_pos(hex);

        commands.spawn((
            HexTile,
            HexPosition::from_hex(hex),
            Mesh3d(mesh_handle.clone()),
            MeshMaterial3d(hex_materials.default.clone()),
            TileBaseMaterial(hex_materials.default.clone()),
            Transform::from_xyz(world_pos.x, 0.0, world_pos.y).with_rotation(flat_rotation),
        ));
    }
}

/// Converts screen-space mouse position to world-space XZ coordinates
/// using the camera projection.
fn screen_to_ground(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    cursor_position: Vec2,
) -> Option<Vec2> {
    // Cast a ray from the camera through the cursor position.
    let ray = camera
        .viewport_to_world(camera_transform, cursor_position)
        .ok()?;

    // Find where the ray intersects the Y=0 ground plane.
    // Ray: P = origin + t * direction
    // Ground plane: y = 0
    // origin.y + t * direction.y = 0
    // t = -origin.y / direction.y
    let direction_y = ray.direction.y;
    if direction_y.abs() < 1e-6 {
        // Ray is parallel to ground plane, no intersection.
        return None;
    }

    let t = -ray.origin.y / direction_y;
    if t < 0.0 {
        // Intersection is behind the camera.
        return None;
    }

    let hit = ray.origin + t * *ray.direction;
    Some(Vec2::new(hit.x, hit.z))
}

/// Updates the hovered hex based on the current mouse position.
pub fn update_hover(
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    config: Res<HexGridConfig>,
    mut hovered: ResMut<HoveredHex>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        // Mouse is outside the window.
        hovered.position = None;
        return;
    };

    let Some(world_pos) = screen_to_ground(camera, camera_transform, cursor_pos) else {
        hovered.position = None;
        return;
    };

    // Convert world XZ position to hex coordinates.
    let hex = config.layout.world_pos_to_hex(world_pos);

    // Only consider hexes within the grid radius.
    let distance = hex.unsigned_distance_to(hexx::Hex::ZERO);
    if distance <= config.map_radius {
        let pos = HexPosition::from_hex(hex);
        hovered.position = Some(pos);
    } else {
        hovered.position = None;
    }
}

/// Pixel distance threshold to distinguish a click from a drag.
const DRAG_THRESHOLD: f32 = 5.0;

/// Handles mouse click to select a hex tile and fire `HexSelectedEvent`.
///
/// Fires on button *release* rather than press so that left-click drags
/// (used for panning) are not mistaken for tile selections.
///
/// **Shift+click** toggles the tile in/out of the multi-selection set
/// without changing the primary `SelectedHex`. Normal click clears the
/// multi-selection and sets `SelectedHex` as before.
#[allow(clippy::too_many_arguments)]
pub fn handle_click(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    keyboard: Res<ButtonInput<KeyCode>>,
    hovered: Res<HoveredHex>,
    mut selected: ResMut<SelectedHex>,
    mut selection: ResMut<Selection>,
    tile_query: Query<(Entity, &HexPosition), With<HexTile>>,
    mut commands: Commands,
    mut drag_acc: Local<f32>,
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        *drag_acc = 0.0;
    }

    if mouse_buttons.pressed(MouseButton::Left) {
        *drag_acc += mouse_motion.delta.length();
    }

    if !mouse_buttons.just_released(MouseButton::Left) {
        return;
    }

    // If the mouse moved more than the threshold, this was a drag, not a click.
    if *drag_acc > DRAG_THRESHOLD {
        return;
    }

    let Some(pos) = hovered.position else {
        return;
    };

    let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    if shift_held {
        // Shift+click: toggle tile entity in/out of multi-selection.
        if let Some((entity, _)) = tile_query.iter().find(|(_, p)| **p == pos)
            && !selection.entities.remove(&entity)
        {
            selection.entities.insert(entity);
        }
    } else {
        // Normal click: clear multi-selection, toggle primary selection.
        selection.entities.clear();
        if selected.position == Some(pos) {
            selected.position = None;
        } else {
            selected.position = Some(pos);
            commands.trigger(HexSelectedEvent { position: pos });
        }
    }
}

/// Observer: handles commands dispatched via the shortcut registry.
pub fn handle_hex_grid_command(
    trigger: On<crate::contracts::shortcuts::CommandExecutedEvent>,
    selected: Option<ResMut<SelectedHex>>,
) {
    if trigger.event().command_id.0 == "edit.deselect"
        && let Some(mut sel) = selected
    {
        sel.position = None;
    }
}

/// Builds a hexagonal ring (hollow hexagon) mesh in the XY plane.
/// `outer_radius` is the outer edge radius, `inner_radius` is the inner edge.
fn build_hex_ring_mesh(outer_radius: f32, inner_radius: f32) -> Mesh {
    use bevy::asset::RenderAssetUsages;
    use bevy::mesh::{Indices, PrimitiveTopology};

    const SIDES: usize = 6;
    let mut positions = Vec::with_capacity(SIDES * 2);
    let mut normals = Vec::with_capacity(SIDES * 2);
    let mut uvs = Vec::with_capacity(SIDES * 2);
    let mut indices = Vec::with_capacity(SIDES * 6);

    let inner_ratio = inner_radius / outer_radius;

    for i in 0..SIDES {
        // Start at π/2 to match Bevy's RegularPolygon vertex placement.
        let angle = std::f32::consts::FRAC_PI_2 + std::f32::consts::TAU * i as f32 / SIDES as f32;
        let (sin, cos) = angle.sin_cos();

        // Outer vertex
        positions.push([outer_radius * cos, outer_radius * sin, 0.0]);
        normals.push([0.0, 0.0, 1.0]);
        uvs.push([0.5 + 0.5 * cos, 0.5 + 0.5 * sin]);

        // Inner vertex
        positions.push([inner_radius * cos, inner_radius * sin, 0.0]);
        normals.push([0.0, 0.0, 1.0]);
        uvs.push([0.5 + 0.5 * inner_ratio * cos, 0.5 + 0.5 * inner_ratio * sin]);
    }

    for i in 0..SIDES {
        let next = (i + 1) % SIDES;
        let o_i = (i * 2) as u32;
        let i_i = (i * 2 + 1) as u32;
        let o_next = (next * 2) as u32;
        let i_next = (next * 2 + 1) as u32;

        indices.extend_from_slice(&[o_i, o_next, i_i]);
        indices.extend_from_slice(&[i_i, o_next, i_next]);
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

/// Spawns hover and selection ring overlay entities with indicator materials.
pub fn setup_indicators(
    mut commands: Commands,
    config: Res<HexGridConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let hex_size = config.layout.scale.x.max(config.layout.scale.y);
    let ring_mesh = meshes.add(build_hex_ring_mesh(hex_size * 0.93, hex_size * 0.82));
    let flat_rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);

    let hover_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.6),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let select_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 1.0),
        unlit: true,
        ..default()
    });

    let multi_select_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 1.0),
        unlit: true,
        ..default()
    });
    commands.insert_resource(IndicatorMaterials {
        hover: hover_material.clone(),
        multi_select: multi_select_material,
        ring_mesh: ring_mesh.clone(),
        flat_rotation,
    });

    commands.spawn((
        HoverIndicator,
        Mesh3d(ring_mesh.clone()),
        MeshMaterial3d(hover_material),
        Transform::from_xyz(0.0, 0.01, 0.0).with_rotation(flat_rotation),
        Visibility::Hidden,
    ));

    commands.spawn((
        SelectIndicator,
        Mesh3d(ring_mesh.clone()),
        MeshMaterial3d(select_material),
        Transform::from_xyz(0.0, 0.02, 0.0).with_rotation(flat_rotation),
        Visibility::Hidden,
    ));

    // Move overlay materials (0.4.0).
    let valid_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.2, 0.8, 0.2, 0.4),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let blocked_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.8, 0.2, 0.2, 0.4),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    commands.insert_resource(OverlayMaterials {
        valid: valid_material,
        blocked: blocked_material,
        ring_mesh,
    });
}

/// Positions hover and selection ring overlays at the appropriate hex tiles.
/// Tiles always keep their real cell type color — only the border ring changes.
///
/// In **Paint mode**, the hover ring uses the paint preview color so the user
/// can see which color will be applied before clicking.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn update_indicators(
    hovered: Res<HoveredHex>,
    selected: Res<SelectedHex>,
    tool: Res<EditorTool>,
    paint_preview: Option<Res<PaintPreview>>,
    config: Res<HexGridConfig>,
    indicator_materials: Res<IndicatorMaterials>,
    mut hover_q: Query<
        (
            &mut Transform,
            &mut Visibility,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        (With<HoverIndicator>, Without<SelectIndicator>),
    >,
    mut select_q: Query<
        (&mut Transform, &mut Visibility),
        (With<SelectIndicator>, Without<HoverIndicator>),
    >,
) {
    let paint_changed = paint_preview
        .as_ref()
        .is_some_and(bevy::prelude::DetectChanges::is_changed);
    if !hovered.is_changed() && !selected.is_changed() && !tool.is_changed() && !paint_changed {
        return;
    }

    if let Ok((mut transform, mut vis, mut mat)) = hover_q.single_mut() {
        match hovered.position {
            Some(pos) if selected.position != Some(pos) => {
                let wp = config.layout.hex_to_world_pos(pos.to_hex());
                transform.translation.x = wp.x;
                transform.translation.z = wp.y;
                *vis = Visibility::Visible;

                let paint_mat = if *tool == EditorTool::Paint {
                    paint_preview.as_ref().and_then(|p| p.material.clone())
                } else {
                    None
                };
                mat.0 = paint_mat.unwrap_or_else(|| indicator_materials.hover.clone());
            }
            _ => {
                *vis = Visibility::Hidden;
            }
        }
    }

    if let Ok((mut transform, mut vis)) = select_q.single_mut() {
        match selected.position {
            Some(pos) => {
                let wp = config.layout.hex_to_world_pos(pos.to_hex());
                transform.translation.x = wp.x;
                transform.translation.z = wp.y;
                *vis = Visibility::Visible;
            }
            None => {
                *vis = Visibility::Hidden;
            }
        }
    }
}

/// Spawns and despawns multi-selection ring indicators to match `Selection`.
pub fn sync_multi_select_indicators(
    selection: Res<Selection>,
    config: Res<HexGridConfig>,
    indicator_materials: Res<IndicatorMaterials>,
    tile_positions: Query<&HexPosition, With<HexTile>>,
    existing: Query<(Entity, &MultiSelectIndicator)>,
    mut commands: Commands,
) {
    if !selection.is_changed() {
        return;
    }

    // Despawn indicators for entities no longer in selection.
    for (indicator_entity, indicator) in &existing {
        if !selection.entities.contains(&indicator.tile_entity) {
            commands.entity(indicator_entity).despawn();
        }
    }

    // Collect tile entities that already have indicators.
    let has_indicator: std::collections::HashSet<Entity> =
        existing.iter().map(|(_, ind)| ind.tile_entity).collect();

    // Spawn indicators for newly selected tile entities.
    for &tile_entity in &selection.entities {
        if has_indicator.contains(&tile_entity) {
            continue;
        }
        let Ok(pos) = tile_positions.get(tile_entity) else {
            continue;
        };
        let wp = config.layout.hex_to_world_pos(pos.to_hex());
        commands.spawn((
            MultiSelectIndicator { tile_entity },
            Mesh3d(indicator_materials.ring_mesh.clone()),
            MeshMaterial3d(indicator_materials.multi_select.clone()),
            Transform::from_xyz(wp.x, 0.025, wp.y).with_rotation(indicator_materials.flat_rotation),
        ));
    }
}

/// Spawns, updates, and despawns move overlay entities based on `ValidMoveSet`.
///
/// - When `ValidMoveSet` has a selected entity and positions, overlays are
///   spawned (or updated) above tiles at y=0.015.
/// - When `ValidMoveSet` is empty (no unit selected), all overlays are despawned.
/// - Uses change detection to avoid work when the move set hasn't changed.
pub fn sync_move_overlays(
    valid_moves: Res<ValidMoveSet>,
    overlay_materials: Res<OverlayMaterials>,
    config: Res<HexGridConfig>,
    existing_overlays: Query<(Entity, &MoveOverlay)>,
    mut commands: Commands,
) {
    if !valid_moves.is_changed() {
        return;
    }

    let flat_rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);

    // Despawn all existing overlays first (simple pool strategy).
    for (entity, _) in &existing_overlays {
        commands.entity(entity).despawn();
    }

    // If no unit is selected, we're done — overlays are cleared.
    if valid_moves.for_entity.is_none() {
        return;
    }

    // Spawn valid overlays.
    for &pos in &valid_moves.valid_positions {
        let wp = config.layout.hex_to_world_pos(pos.to_hex());
        commands.spawn((
            MoveOverlay {
                state: MoveOverlayState::Valid,
                position: pos,
            },
            Mesh3d(overlay_materials.ring_mesh.clone()),
            MeshMaterial3d(overlay_materials.valid.clone()),
            Transform::from_xyz(wp.x, 0.015, wp.y).with_rotation(flat_rotation),
        ));
    }

    // Spawn blocked overlays.
    for &pos in valid_moves.blocked_explanations.keys() {
        let wp = config.layout.hex_to_world_pos(pos.to_hex());
        commands.spawn((
            MoveOverlay {
                state: MoveOverlayState::Blocked,
                position: pos,
            },
            Mesh3d(overlay_materials.ring_mesh.clone()),
            MeshMaterial3d(overlay_materials.blocked.clone()),
            Transform::from_xyz(wp.x, 0.015, wp.y).with_rotation(flat_rotation),
        ));
    }
}

/// Draws a LOS ray from the selected unit to the hovered hex using gizmos.
///
/// Green line if line of sight is clear, red if blocked. Only active when
/// a unit is selected and the mouse hovers a different hex.
pub fn draw_los_ray(
    selected_unit: Res<SelectedUnit>,
    hovered: Res<HoveredHex>,
    config: Res<HexGridConfig>,
    unit_positions: Query<&HexPosition, With<UnitInstance>>,
    mut gizmos: Gizmos,
) {
    let Some(unit_entity) = selected_unit.entity else {
        return;
    };
    let Ok(&unit_pos) = unit_positions.get(unit_entity) else {
        return;
    };
    let Some(hover_pos) = hovered.position else {
        return;
    };
    if unit_pos == hover_pos {
        return;
    }

    // Placeholder: nothing blocks LOS until property system (#81) ships.
    let result = algorithms::line_of_sight(unit_pos, hover_pos, |_| false);

    let color = if result.clear {
        Color::srgb(0.2, 0.9, 0.2)
    } else {
        Color::srgb(0.9, 0.2, 0.2)
    };

    for window in result.path.windows(2) {
        let a = config.layout.hex_to_world_pos(window[0].to_hex());
        let b = config.layout.hex_to_world_pos(window[1].to_hex());
        gizmos.line(Vec3::new(a.x, 0.03, a.y), Vec3::new(b.x, 0.03, b.y), color);
    }
}

#[cfg(test)]
pub fn tile_count_for_radius(radius: u32) -> usize {
    // A hex grid of radius r has 3*r*(r+1) + 1 tiles.
    if radius == 0 {
        1
    } else {
        (3 * radius * (radius + 1) + 1) as usize
    }
}

/// Despawns plugin-internal entities on editor exit.
/// Contract-level entities (`HexTile`, `MoveOverlay`) are cleaned up by
/// `persistence::cleanup_editor_entities`. This handles the rest.
pub fn cleanup_internal_entities(
    mut commands: Commands,
    hover: Query<Entity, With<HoverIndicator>>,
    select: Query<Entity, With<SelectIndicator>>,
) {
    for entity in hover.iter().chain(select.iter()) {
        commands.entity(entity).despawn();
    }
}
