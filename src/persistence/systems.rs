//! Systems for the persistence plugin.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::contracts::editor_ui::{ToastEvent, ToastKind};
use crate::contracts::game_system::{
    EntityData, EntityTypeRegistry, EnumRegistry, GameSystem, SelectedUnit, StructRegistry,
    UnitInstance,
};
use crate::contracts::hex_grid::{HexGridConfig, HexPosition, HexTile, MoveOverlay};
use crate::contracts::mechanics::{
    ActiveCombat, CombatModifierRegistry, CombatResultsTable, TurnState, TurnStructure,
};
use crate::contracts::ontology::{ConceptRegistry, ConstraintRegistry, RelationRegistry};
use crate::contracts::persistence::{
    AppScreen, CloseProjectEvent, FORMAT_VERSION, GameSystemFile, LoadRequestEvent,
    NewProjectEvent, PendingBoardLoad, SaveRequestEvent, TileSaveData, UnitSaveData, Workspace,
};
use crate::contracts::storage::Storage;
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
    turn_structure: Res<TurnStructure>,
    crt: Res<CombatResultsTable>,
    combat_modifiers: Res<CombatModifierRegistry>,
    config: Res<HexGridConfig>,
    tiles: Query<(&HexPosition, &EntityData), With<HexTile>>,
    units: Query<(&HexPosition, &EntityData), With<UnitInstance>>,
    storage: Res<Storage>,
    mut workspace: ResMut<Workspace>,
    mut commands: Commands,
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
        } else {
            let base = storage.provider().base_dir();
            if std::fs::create_dir_all(base).is_ok() {
                dialog = dialog.set_directory(base);
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
        turn_structure: turn_structure.clone(),
        combat_results_table: crt.clone(),
        combat_modifiers: combat_modifiers.clone(),
        map_radius: config.map_radius,
        tiles: tile_data,
        units: unit_data,
        workspace_preset: workspace.workspace_preset.clone(),
        font_size_base: workspace.font_size_base,
    };

    match storage.provider().save_at(&path, &file) {
        Ok(()) => {
            info!("Saved to {}", path.display());
            workspace.file_path = Some(path);
            workspace.dirty = false;
            commands.trigger(ToastEvent {
                message: "Project saved".to_string(),
                kind: ToastKind::Success,
            });
        }
        Err(e) => {
            error!("Failed to save: {e}");
            commands.trigger(ToastEvent {
                message: format!("Save failed: {e}"),
                kind: ToastKind::Error,
            });
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
    mut turn_structure: ResMut<TurnStructure>,
    mut crt: ResMut<CombatResultsTable>,
    mut combat_modifiers: ResMut<CombatModifierRegistry>,
    mut schema: ResMut<SchemaValidation>,
    mut workspace: ResMut<Workspace>,
    storage: Res<Storage>,
    mut next_state: ResMut<NextState<AppScreen>>,
    mut commands: Commands,
) {
    let dialog = rfd::FileDialog::new().add_filter("Hexorder", &["hexorder"]);
    let Some(path) = dialog.pick_file() else {
        return; // User cancelled.
    };

    let file = match storage.provider().load(&path) {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to load: {e}");
            commands.trigger(ToastEvent {
                message: format!("Load failed: {e}"),
                kind: ToastKind::Error,
            });
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
    *turn_structure = file.turn_structure;
    *crt = file.combat_results_table;
    *combat_modifiers = file.combat_modifiers;

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
    workspace.workspace_preset = file.workspace_preset;
    workspace.font_size_base = file.font_size_base;

    // Insert pending board load for deferred application.
    commands.insert_resource(PendingBoardLoad {
        tiles: file.tiles,
        units: file.units,
    });

    // Transition to editor (may already be in editor if loading from editor).
    next_state.set(AppScreen::Editor);

    commands.trigger(ToastEvent {
        message: "Project loaded".to_string(),
        kind: ToastKind::Success,
    });

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
    mut turn_structure: ResMut<TurnStructure>,
    mut crt: ResMut<CombatResultsTable>,
    mut combat_modifiers: ResMut<CombatModifierRegistry>,
    mut turn_state: ResMut<TurnState>,
    mut active_combat: ResMut<ActiveCombat>,
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

    // Reset mechanics to factory defaults.
    *turn_structure = crate::game_system::create_default_turn_structure();
    *crt = crate::game_system::create_default_crt();
    *combat_modifiers = CombatModifierRegistry::default();
    *turn_state = TurnState::default();
    *active_combat = ActiveCombat::default();

    workspace.name.clone_from(&event.name);
    workspace.file_path = None;
    workspace.dirty = false;
    workspace.workspace_preset = String::new();
    workspace.font_size_base = 15.0;

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

/// Handles file commands dispatched via the shortcut registry.
/// Maps `CommandExecutedEvent` command IDs to persistence events.
pub fn handle_file_command(
    trigger: On<crate::contracts::shortcuts::CommandExecutedEvent>,
    mut commands: Commands,
) {
    match trigger.event().command_id.0 {
        "file.save" => commands.trigger(SaveRequestEvent { save_as: false }),
        "file.save_as" => commands.trigger(SaveRequestEvent { save_as: true }),
        "file.open" => commands.trigger(LoadRequestEvent),
        "file.new" => commands.trigger(CloseProjectEvent),
        _ => {} // Not our command.
    }
}

/// Despawns all editor-spawned entities on `OnExit(AppScreen::Editor)`.
/// Ensures a clean slate when returning to the launcher or re-entering the editor.
/// The camera is NOT despawned — it is a global entity spawned at `Startup`.
pub fn cleanup_editor_entities(
    mut commands: Commands,
    tiles: Query<Entity, With<HexTile>>,
    units: Query<Entity, With<UnitInstance>>,
    overlays: Query<Entity, With<MoveOverlay>>,
) {
    for entity in tiles.iter().chain(units.iter()).chain(overlays.iter()) {
        commands.entity(entity).despawn();
    }
}

/// Applies pending board state after a load operation. Runs when
/// `PendingBoardLoad` exists, matching loaded tile data to spawned
/// tile entities by `HexPosition` and spawning unit entities.
///
/// Defers execution until every `HexTile` has an `EntityData` component.
/// Tiles are spawned without `EntityData` by `spawn_grid`; the cell
/// plugin's `assign_default_cell_data` adds it via deferred commands.
/// Waiting avoids a race where both this system and the cell system
/// queue competing deferred inserts on the same frame.
///
/// Unit entities are spawned with core ECS components only (no mesh/material).
/// The unit plugin's `sync_unit_visuals` and `sync_unit_materials` systems
/// will attach visuals on the next frame via change detection.
pub fn apply_pending_board_load(
    pending: Option<Res<PendingBoardLoad>>,
    mut tiles: Query<(&HexPosition, &mut EntityData), With<HexTile>>,
    tiles_pending_data: Query<(), (With<HexTile>, Without<EntityData>)>,
    config: Res<HexGridConfig>,
    mut commands: Commands,
) {
    let Some(pending) = pending else {
        return;
    };

    // Tiles spawned by spawn_grid initially lack EntityData — the cell
    // plugin adds defaults via deferred commands. Wait until every tile
    // has EntityData before overwriting, otherwise the deferred default
    // insert can race with our changes.
    if !tiles_pending_data.is_empty() {
        return;
    }

    // Build a lookup from position to tile save data.
    let tile_lookup: HashMap<HexPosition, &TileSaveData> =
        pending.tiles.iter().map(|t| (t.position, t)).collect();

    // Apply tile data to existing tile entities via direct mutation
    // (not deferred commands) so the values are visible immediately.
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
