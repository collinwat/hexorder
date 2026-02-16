//! Systems for the persistence plugin.

use std::collections::HashMap;
use std::path::PathBuf;

use bevy::prelude::*;

use crate::contracts::game_system::{
    EntityData, EntityTypeRegistry, EnumRegistry, GameSystem, SelectedUnit, StructRegistry,
    UnitInstance,
};
use crate::contracts::hex_grid::{HexGridConfig, HexPosition, HexTile, MoveOverlay};
use crate::contracts::ontology::{ConceptRegistry, ConstraintRegistry, RelationRegistry};
use crate::contracts::persistence::{
    AppScreen, CloseProjectEvent, FORMAT_VERSION, GameSystemFile, LoadRequestEvent,
    NewProjectEvent, PendingBoardLoad, SaveRequestEvent, TileSaveData, UnitSaveData, Workspace,
};
use crate::contracts::validation::SchemaValidation;

// ---------------------------------------------------------------------------
// Shared Helpers
// ---------------------------------------------------------------------------

/// Reset all registries and derived state to factory defaults.
/// Used by both `handle_new_project` and `handle_close_project`.
#[allow(clippy::too_many_arguments)]
fn reset_all_registries(
    game_system: &mut GameSystem,
    entity_types: &mut EntityTypeRegistry,
    enum_registry: &mut EnumRegistry,
    struct_registry: &mut StructRegistry,
    concepts: &mut ConceptRegistry,
    relations: &mut RelationRegistry,
    constraints: &mut ConstraintRegistry,
    schema: &mut SchemaValidation,
    selected_unit: &mut SelectedUnit,
) {
    *game_system = crate::game_system::create_game_system();
    *entity_types = crate::game_system::create_entity_type_registry();
    *enum_registry = crate::game_system::create_enum_registry();
    *struct_registry = StructRegistry::default();
    *concepts = ConceptRegistry::default();
    *relations = RelationRegistry::default();
    *constraints = ConstraintRegistry::default();
    *schema = SchemaValidation::default();
    selected_unit.entity = None;
}

/// Sanitize a workspace name for use as a filename.
/// Replaces disallowed characters with hyphens, trims, and falls back to "untitled".
pub(crate) fn sanitize_filename(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = sanitized.trim().to_string();
    if trimmed.is_empty() {
        "untitled".to_string()
    } else {
        trimmed
    }
}

/// Returns the default save directory for new projects: `~/Documents/Hexorder/`.
fn default_save_directory() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join("Documents").join("Hexorder"))
}

// ---------------------------------------------------------------------------
// Observer Systems
// ---------------------------------------------------------------------------

/// Handles save requests. Builds a `GameSystemFile` from current state
/// and writes it to disk via RON.
#[allow(clippy::too_many_arguments)]
pub fn handle_save_request(
    trigger: On<SaveRequestEvent>,
    game_system: Res<GameSystem>,
    entity_types: Res<EntityTypeRegistry>,
    enum_registry: Res<EnumRegistry>,
    struct_registry: Res<StructRegistry>,
    concepts: Res<ConceptRegistry>,
    relations: Res<RelationRegistry>,
    constraints: Res<ConstraintRegistry>,
    config: Res<HexGridConfig>,
    tiles: Query<(&HexPosition, &EntityData), With<HexTile>>,
    units: Query<(&HexPosition, &EntityData), With<UnitInstance>>,
    mut workspace: ResMut<Workspace>,
) {
    let event = trigger.event();

    // Determine target path.
    let path = if event.save_as || workspace.file_path.is_none() {
        // Pre-fill with sensible defaults.
        let sanitized_name = sanitize_filename(&workspace.name);
        let file_name = format!("{sanitized_name}.hexorder");

        let mut dialog = rfd::FileDialog::new()
            .add_filter("Hexorder", &["hexorder"])
            .set_file_name(&file_name);

        // Pre-fill default directory on first save.
        if let Some(ref existing) = workspace.file_path {
            if let Some(parent) = existing.parent() {
                dialog = dialog.set_directory(parent);
            }
        } else if let Some(default_dir) = default_save_directory() {
            // Attempt to create the directory; set it if successful.
            #[allow(clippy::collapsible_if)]
            if std::fs::create_dir_all(&default_dir).is_ok() {
                dialog = dialog.set_directory(&default_dir);
            }
        }

        match dialog.save_file() {
            Some(p) => p,
            None => return, // User cancelled.
        }
    } else {
        workspace.file_path.clone().expect("checked is_some above")
    };

    // Build save data.
    let tile_data: Vec<TileSaveData> = tiles
        .iter()
        .map(|(pos, data)| TileSaveData {
            position: *pos,
            entity_type_id: data.entity_type_id,
            properties: data.properties.clone(),
        })
        .collect();

    let unit_data: Vec<UnitSaveData> = units
        .iter()
        .map(|(pos, data)| UnitSaveData {
            position: *pos,
            entity_type_id: data.entity_type_id,
            properties: data.properties.clone(),
        })
        .collect();

    let file = GameSystemFile {
        format_version: FORMAT_VERSION,
        name: workspace.name.clone(),
        game_system: game_system.clone(),
        entity_types: entity_types.clone(),
        enums: enum_registry.clone(),
        structs: struct_registry.clone(),
        concepts: concepts.clone(),
        relations: relations.clone(),
        constraints: constraints.clone(),
        map_radius: config.map_radius,
        tiles: tile_data,
        units: unit_data,
    };

    match crate::contracts::persistence::save_to_file(&path, &file) {
        Ok(()) => {
            info!("Saved to {}", path.display());
            workspace.file_path = Some(path);
            workspace.dirty = false;
        }
        Err(e) => {
            error!("Failed to save: {e}");
        }
    }
}

/// Handles load requests. Opens a file dialog, loads the file, overwrites
/// registries, and inserts `PendingBoardLoad` for deferred board reconstruction.
#[allow(clippy::too_many_arguments)]
pub fn handle_load_request(
    _trigger: On<LoadRequestEvent>,
    mut game_system: ResMut<GameSystem>,
    mut entity_types: ResMut<EntityTypeRegistry>,
    mut enum_registry: ResMut<EnumRegistry>,
    mut struct_registry: ResMut<StructRegistry>,
    mut concepts: ResMut<ConceptRegistry>,
    mut relations: ResMut<RelationRegistry>,
    mut constraints: ResMut<ConstraintRegistry>,
    mut schema: ResMut<SchemaValidation>,
    mut workspace: ResMut<Workspace>,
    mut next_state: ResMut<NextState<AppScreen>>,
    mut commands: Commands,
) {
    let dialog = rfd::FileDialog::new().add_filter("Hexorder", &["hexorder"]);
    let Some(path) = dialog.pick_file() else {
        return; // User cancelled.
    };

    let file = match crate::contracts::persistence::load_from_file(&path) {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to load: {e}");
            return;
        }
    };

    // Overwrite registries.
    *game_system = file.game_system;
    *entity_types = file.entity_types;
    *enum_registry = file.enums;
    *struct_registry = file.structs;
    *concepts = file.concepts;
    *relations = file.relations;
    *constraints = file.constraints;

    // Reset derived state.
    *schema = SchemaValidation::default();

    // Derive workspace name: use file name field if present (v3+),
    // otherwise derive from filename stem (v2 backward compat).
    let name = if file.name.is_empty() {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string()
    } else {
        file.name
    };

    workspace.name = name;
    workspace.file_path = Some(path);
    workspace.dirty = false;

    // Insert pending board load for deferred application.
    commands.insert_resource(PendingBoardLoad {
        tiles: file.tiles,
        units: file.units,
    });

    // Transition to editor (may already be in editor if loading from editor).
    next_state.set(AppScreen::Editor);

    info!("Loaded game system: {}", game_system.id);
}

/// Handles new project requests. Resets all registries to defaults,
/// sets workspace name, and transitions to the editor.
#[allow(clippy::too_many_arguments)]
pub fn handle_new_project(
    trigger: On<NewProjectEvent>,
    mut game_system: ResMut<GameSystem>,
    mut entity_types: ResMut<EntityTypeRegistry>,
    mut enum_registry: ResMut<EnumRegistry>,
    mut struct_registry: ResMut<StructRegistry>,
    mut concepts: ResMut<ConceptRegistry>,
    mut relations: ResMut<RelationRegistry>,
    mut constraints: ResMut<ConstraintRegistry>,
    mut schema: ResMut<SchemaValidation>,
    mut selected_unit: ResMut<SelectedUnit>,
    mut workspace: ResMut<Workspace>,
    mut next_state: ResMut<NextState<AppScreen>>,
) {
    let event = trigger.event();

    reset_all_registries(
        &mut game_system,
        &mut entity_types,
        &mut enum_registry,
        &mut struct_registry,
        &mut concepts,
        &mut relations,
        &mut constraints,
        &mut schema,
        &mut selected_unit,
    );

    workspace.name.clone_from(&event.name);
    workspace.file_path = None;
    workspace.dirty = false;

    next_state.set(AppScreen::Editor);
}

/// Handles close project requests. Resets all state and returns to launcher.
#[allow(clippy::too_many_arguments)]
pub fn handle_close_project(
    _trigger: On<CloseProjectEvent>,
    mut workspace: ResMut<Workspace>,
    mut game_system: ResMut<GameSystem>,
    mut entity_types: ResMut<EntityTypeRegistry>,
    mut enum_registry: ResMut<EnumRegistry>,
    mut struct_registry: ResMut<StructRegistry>,
    mut concepts: ResMut<ConceptRegistry>,
    mut relations: ResMut<RelationRegistry>,
    mut constraints: ResMut<ConstraintRegistry>,
    mut schema: ResMut<SchemaValidation>,
    mut selected_unit: ResMut<SelectedUnit>,
    mut next_state: ResMut<NextState<AppScreen>>,
) {
    *workspace = Workspace::default();

    reset_all_registries(
        &mut game_system,
        &mut entity_types,
        &mut enum_registry,
        &mut struct_registry,
        &mut concepts,
        &mut relations,
        &mut constraints,
        &mut schema,
        &mut selected_unit,
    );

    next_state.set(AppScreen::Launcher);
}

// ---------------------------------------------------------------------------
// Update Systems
// ---------------------------------------------------------------------------

/// Keyboard shortcuts for file operations.
/// - `Cmd+S`: Save (to current path, or Save As if no path)
/// - `Cmd+Shift+S`: Save As (always show dialog)
/// - `Cmd+O`: Open
/// - `Cmd+N`: Close project (return to launcher)
pub fn keyboard_shortcuts(input: Option<Res<ButtonInput<KeyCode>>>, mut commands: Commands) {
    let Some(input) = input else {
        return;
    };
    let cmd = input.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]);
    let shift = input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

    if !cmd {
        return;
    }

    if input.just_pressed(KeyCode::KeyS) {
        commands.trigger(SaveRequestEvent { save_as: shift });
    } else if input.just_pressed(KeyCode::KeyO) {
        commands.trigger(LoadRequestEvent);
    } else if input.just_pressed(KeyCode::KeyN) {
        commands.trigger(CloseProjectEvent);
    }
}

/// Despawns all editor-spawned entities on `OnExit(AppScreen::Editor)`.
/// Ensures a clean slate when returning to the launcher or re-entering the editor.
pub fn cleanup_editor_entities(
    mut commands: Commands,
    tiles: Query<Entity, With<HexTile>>,
    units: Query<Entity, With<UnitInstance>>,
    cameras: Query<Entity, With<Camera3d>>,
    overlays: Query<Entity, With<MoveOverlay>>,
) {
    for entity in tiles
        .iter()
        .chain(units.iter())
        .chain(cameras.iter())
        .chain(overlays.iter())
    {
        commands.entity(entity).despawn();
    }
}

/// Applies pending board state after a load operation. Runs when
/// `PendingBoardLoad` exists, matching loaded tile data to spawned
/// tile entities by `HexPosition` and spawning unit entities.
///
/// Unit entities are spawned with core ECS components only (no mesh/material).
/// The unit plugin's `sync_unit_visuals` and `sync_unit_materials` systems
/// will attach visuals on the next frame via change detection.
pub fn apply_pending_board_load(
    pending: Option<Res<PendingBoardLoad>>,
    mut tiles: Query<(&HexPosition, &mut EntityData), With<HexTile>>,
    config: Res<HexGridConfig>,
    mut commands: Commands,
) {
    let Some(pending) = pending else {
        return;
    };

    // Build a lookup from position to tile save data.
    let tile_lookup: HashMap<HexPosition, &TileSaveData> =
        pending.tiles.iter().map(|t| (t.position, t)).collect();

    // Apply tile data to existing tile entities.
    for (pos, mut entity_data) in &mut tiles {
        if let Some(save_data) = tile_lookup.get(pos) {
            entity_data.entity_type_id = save_data.entity_type_id;
            entity_data.properties.clone_from(&save_data.properties);
        }
    }

    // Spawn unit entities with core components. The unit plugin's sync
    // systems will add mesh/material via change detection.
    for unit in &pending.units {
        let hex = unit.position.to_hex();
        let world_pos = config.layout.hex_to_world_pos(hex);

        commands.spawn((
            UnitInstance,
            HexPosition::new(unit.position.q, unit.position.r),
            EntityData {
                entity_type_id: unit.entity_type_id,
                properties: unit.properties.clone(),
            },
            Transform::from_xyz(world_pos.x, 0.25, world_pos.y),
        ));
    }

    // Remove the pending resource.
    commands.remove_resource::<PendingBoardLoad>();
}
