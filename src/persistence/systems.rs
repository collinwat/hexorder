//! Systems for the persistence feature plugin.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::contracts::game_system::{
    EntityData, EntityTypeRegistry, GameSystem, SelectedUnit, UnitInstance,
};
use crate::contracts::hex_grid::{HexGridConfig, HexPosition, HexTile};
use crate::contracts::ontology::{ConceptRegistry, ConstraintRegistry, RelationRegistry};
use crate::contracts::persistence::{
    AppScreen, CurrentFilePath, FORMAT_VERSION, GameSystemFile, LoadRequestEvent, NewProjectEvent,
    PendingBoardLoad, SaveRequestEvent, TileSaveData, UnitSaveData,
};
use crate::contracts::validation::SchemaValidation;

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
    concepts: Res<ConceptRegistry>,
    relations: Res<RelationRegistry>,
    constraints: Res<ConstraintRegistry>,
    config: Res<HexGridConfig>,
    tiles: Query<(&HexPosition, &EntityData), With<HexTile>>,
    units: Query<(&HexPosition, &EntityData), With<UnitInstance>>,
    mut file_path: ResMut<CurrentFilePath>,
) {
    let event = trigger.event();

    // Determine target path.
    let path = if event.save_as || file_path.path.is_none() {
        // Show save dialog.
        let dialog = rfd::FileDialog::new()
            .add_filter("Hexorder", &["hexorder"])
            .set_file_name("untitled.hexorder");
        match dialog.save_file() {
            Some(p) => p,
            None => return, // User cancelled.
        }
    } else {
        file_path.path.clone().expect("checked is_some above")
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
        game_system: game_system.clone(),
        entity_types: entity_types.clone(),
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
            file_path.path = Some(path);
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
    mut concepts: ResMut<ConceptRegistry>,
    mut relations: ResMut<RelationRegistry>,
    mut constraints: ResMut<ConstraintRegistry>,
    mut schema: ResMut<SchemaValidation>,
    mut file_path: ResMut<CurrentFilePath>,
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
    *concepts = file.concepts;
    *relations = file.relations;
    *constraints = file.constraints;

    // Reset derived state.
    *schema = SchemaValidation::default();

    // Store file path.
    file_path.path = Some(path);

    // Insert pending board load for deferred application.
    commands.insert_resource(PendingBoardLoad {
        tiles: file.tiles,
        units: file.units,
    });

    // Transition to editor (may already be in editor if loading from editor).
    next_state.set(AppScreen::Editor);

    info!("Loaded game system: {}", game_system.id);
}

/// Handles new project requests. Resets all registries to defaults and
/// transitions to the editor.
#[allow(clippy::too_many_arguments)]
pub fn handle_new_project(
    _trigger: On<NewProjectEvent>,
    mut game_system: ResMut<GameSystem>,
    mut entity_types: ResMut<EntityTypeRegistry>,
    mut concepts: ResMut<ConceptRegistry>,
    mut relations: ResMut<RelationRegistry>,
    mut constraints: ResMut<ConstraintRegistry>,
    mut schema: ResMut<SchemaValidation>,
    mut selected_unit: ResMut<SelectedUnit>,
    mut file_path: ResMut<CurrentFilePath>,
    mut next_state: ResMut<NextState<AppScreen>>,
) {
    // Reset to factory defaults (reuse game_system plugin's factory functions).
    *game_system = crate::game_system::create_game_system();
    let registry = crate::game_system::create_entity_type_registry();
    *entity_types = registry;

    // Reset ontology.
    *concepts = ConceptRegistry::default();
    *relations = RelationRegistry::default();
    *constraints = ConstraintRegistry::default();

    // Reset derived state.
    *schema = SchemaValidation::default();
    selected_unit.entity = None;
    file_path.path = None;

    next_state.set(AppScreen::Editor);
}

// ---------------------------------------------------------------------------
// Update Systems
// ---------------------------------------------------------------------------

/// Keyboard shortcuts for file operations.
/// - `Cmd+S`: Save (to current path, or Save As if no path)
/// - `Cmd+Shift+S`: Save As (always show dialog)
/// - `Cmd+O`: Open
/// - `Cmd+N`: New project
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
        commands.trigger(NewProjectEvent);
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
