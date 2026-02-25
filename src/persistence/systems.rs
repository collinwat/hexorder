//! Systems for the persistence plugin.

use std::collections::HashMap;

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use hexorder_contracts::editor_ui::{ToastEvent, ToastKind};
use hexorder_contracts::game_system::{
    EntityData, EntityTypeRegistry, EnumRegistry, GameSystem, SelectedUnit, StructRegistry,
    UnitInstance,
};
use hexorder_contracts::hex_grid::{HexGridConfig, HexPosition, HexTile, MoveOverlay};
use hexorder_contracts::mechanics::{
    ActiveCombat, CombatModifierRegistry, CombatResultsTable, TurnState, TurnStructure,
};
use hexorder_contracts::ontology::{ConceptRegistry, ConstraintRegistry, RelationRegistry};
use hexorder_contracts::persistence::{
    AppScreen, CloseProjectEvent, FORMAT_VERSION, GameSystemFile, LoadRequestEvent,
    NewProjectEvent, PendingBoardLoad, SaveRequestEvent, TileSaveData, UnitSaveData, Workspace,
};
use hexorder_contracts::storage::Storage;
use hexorder_contracts::validation::SchemaValidation;

// ---------------------------------------------------------------------------
// Shared Helpers
// ---------------------------------------------------------------------------

/// Clears all keyboard input state via a deferred command.
///
/// Native file dialogs (`rfd`) take over the macOS event loop, so key-up
/// events that occur while the dialog is open are never delivered to Bevy.
/// Without this reset, keys pressed during dialog navigation (especially
/// arrow keys) remain "stuck" in `ButtonInput`, causing systems like
/// `keyboard_pan` to fire continuously after the dialog closes.
///
/// Also resets the `bevy_egui` `ModifierKeysState` so that modifier keys
/// (Cmd, Ctrl, Shift, Alt) held when the dialog opened are not treated
/// as still pressed after it closes.
fn clear_keyboard_after_dialog(commands: &mut Commands) {
    commands.queue(|world: &mut World| {
        if let Some(mut keys) = world.get_resource_mut::<ButtonInput<KeyCode>>() {
            keys.reset_all();
        }
        if let Some(mut mods) = world.get_resource_mut::<bevy_egui::input::ModifierKeysState>() {
            mods.shift = false;
            mods.ctrl = false;
            mods.alt = false;
            mods.win = false;
        }
    });
}

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

/// Result of the unsaved-changes confirmation dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfirmAction {
    /// No unsaved changes, or user chose "Don't Save" — proceed with the action.
    Proceed,
    /// User chose "Save" and save succeeded — proceed with the action.
    SavedThenProceed,
    /// User cancelled or save failed — abort the action.
    Cancel,
}

/// Check for unsaved changes and prompt the user if dirty.
/// Returns the action to take. When `workspace.dirty` is false, returns
/// `Proceed` immediately without showing a dialog.
///
/// This function is NOT called in tests because it shows a blocking dialog.
/// Tests verify the `ConfirmAction` enum and the clean-path logic.
pub(crate) fn check_unsaved_changes(workspace: &Workspace) -> ConfirmAction {
    if !workspace.dirty {
        return ConfirmAction::Proceed;
    }

    let result = rfd::MessageDialog::new()
        .set_title("Unsaved Changes")
        .set_description("You have unsaved changes. Do you want to save before continuing?")
        .set_buttons(rfd::MessageButtons::YesNoCancel)
        .set_level(rfd::MessageLevel::Warning)
        .show();

    match result {
        rfd::MessageDialogResult::Yes => ConfirmAction::SavedThenProceed,
        rfd::MessageDialogResult::No => ConfirmAction::Proceed,
        _ => ConfirmAction::Cancel,
    }
}

/// Perform the save operation. Returns `true` if save succeeded, `false` if
/// cancelled or failed. When `force_dialog` is true, always shows the file
/// picker (Save As behavior).
#[allow(clippy::too_many_arguments)]
fn do_save(
    force_dialog: bool,
    workspace: &mut Workspace,
    game_system: &GameSystem,
    entity_types: &EntityTypeRegistry,
    enum_registry: &EnumRegistry,
    struct_registry: &StructRegistry,
    concepts: &ConceptRegistry,
    relations: &RelationRegistry,
    constraints: &ConstraintRegistry,
    turn_structure: &TurnStructure,
    crt: &CombatResultsTable,
    combat_modifiers: &CombatModifierRegistry,
    config: &HexGridConfig,
    tiles: &[(HexPosition, EntityData)],
    units: &[(HexPosition, EntityData)],
    storage: &Storage,
    commands: &mut Commands,
) -> bool {
    // Determine target path.
    let path = if force_dialog || workspace.file_path.is_none() {
        let sanitized_name = sanitize_filename(&workspace.name);
        let file_name = format!("{sanitized_name}.hexorder");

        let mut dialog = rfd::FileDialog::new()
            .add_filter("Hexorder", &["hexorder"])
            .set_file_name(&file_name);

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

        let result = dialog.save_file();
        clear_keyboard_after_dialog(commands);
        match result {
            Some(p) => p,
            None => return false, // User cancelled.
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
            true
        }
        Err(e) => {
            error!("Failed to save: {e}");
            commands.trigger(ToastEvent {
                message: format!("Save failed: {e}"),
                kind: ToastKind::Error,
            });
            false
        }
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
    let tile_vec: Vec<_> = tiles.iter().map(|(p, d)| (*p, d.clone())).collect();
    let unit_vec: Vec<_> = units.iter().map(|(p, d)| (*p, d.clone())).collect();

    do_save(
        trigger.event().save_as,
        &mut workspace,
        &game_system,
        &entity_types,
        &enum_registry,
        &struct_registry,
        &concepts,
        &relations,
        &constraints,
        &turn_structure,
        &crt,
        &combat_modifiers,
        &config,
        &tile_vec,
        &unit_vec,
        &storage,
        &mut commands,
    );
}

/// Handles load requests. Opens a file dialog, loads the file, overwrites
/// registries, and inserts `PendingBoardLoad` for deferred board reconstruction.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
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
    mut workspace: ResMut<Workspace>,
    storage: Res<Storage>,
    save_ctx: (
        Res<HexGridConfig>,
        Query<(&HexPosition, &EntityData), With<HexTile>>,
        Query<(&HexPosition, &EntityData), With<UnitInstance>>,
    ),
    load_ctx: (ResMut<SchemaValidation>, ResMut<NextState<AppScreen>>),
    mut commands: Commands,
) {
    let (config, tiles, units_q) = save_ctx;
    let (mut schema, mut next_state) = load_ctx;
    let confirm = check_unsaved_changes(&workspace);
    match confirm {
        ConfirmAction::Cancel => return,
        ConfirmAction::SavedThenProceed => {
            let tile_vec: Vec<_> = tiles.iter().map(|(p, d)| (*p, d.clone())).collect();
            let unit_vec: Vec<_> = units_q.iter().map(|(p, d)| (*p, d.clone())).collect();
            if !do_save(
                false,
                &mut workspace,
                &game_system,
                &entity_types,
                &enum_registry,
                &struct_registry,
                &concepts,
                &relations,
                &constraints,
                &turn_structure,
                &crt,
                &combat_modifiers,
                &config,
                &tile_vec,
                &unit_vec,
                &storage,
                &mut commands,
            ) {
                return; // Save cancelled or failed — abort load.
            }
        }
        ConfirmAction::Proceed => {}
    }

    let dialog = rfd::FileDialog::new().add_filter("Hexorder", &["hexorder"]);
    let path = dialog.pick_file();
    clear_keyboard_after_dialog(&mut commands);
    let Some(path) = path else {
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
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
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
    mut workspace: ResMut<Workspace>,
    storage: Res<Storage>,
    save_ctx: (
        Res<HexGridConfig>,
        Query<(&HexPosition, &EntityData), With<HexTile>>,
        Query<(&HexPosition, &EntityData), With<UnitInstance>>,
    ),
    new_ctx: (
        ResMut<TurnState>,
        ResMut<ActiveCombat>,
        ResMut<SchemaValidation>,
        ResMut<SelectedUnit>,
        ResMut<NextState<AppScreen>>,
    ),
    mut commands: Commands,
) {
    let (config, tiles, units_q) = save_ctx;
    let (mut turn_state, mut active_combat, mut schema, mut selected_unit, mut next_state) =
        new_ctx;

    let confirm = check_unsaved_changes(&workspace);
    match confirm {
        ConfirmAction::Cancel => return,
        ConfirmAction::SavedThenProceed => {
            let tile_vec: Vec<_> = tiles.iter().map(|(p, d)| (*p, d.clone())).collect();
            let unit_vec: Vec<_> = units_q.iter().map(|(p, d)| (*p, d.clone())).collect();
            if !do_save(
                false,
                &mut workspace,
                &game_system,
                &entity_types,
                &enum_registry,
                &struct_registry,
                &concepts,
                &relations,
                &constraints,
                &turn_structure,
                &crt,
                &combat_modifiers,
                &config,
                &tile_vec,
                &unit_vec,
                &storage,
                &mut commands,
            ) {
                return; // Save cancelled or failed — abort new project.
            }
        }
        ConfirmAction::Proceed => {}
    }

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
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
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
    save_ctx: (
        Res<TurnStructure>,
        Res<CombatResultsTable>,
        Res<CombatModifierRegistry>,
        Res<HexGridConfig>,
        Res<Storage>,
    ),
    board_queries: (
        Query<(&HexPosition, &EntityData), With<HexTile>>,
        Query<(&HexPosition, &EntityData), With<UnitInstance>>,
    ),
    close_ctx: (
        ResMut<SchemaValidation>,
        ResMut<SelectedUnit>,
        ResMut<NextState<AppScreen>>,
    ),
    mut commands: Commands,
) {
    let (turn_structure, crt, combat_modifiers, config, storage) = save_ctx;
    let (tiles, units_q) = board_queries;
    let (mut schema, mut selected_unit, mut next_state) = close_ctx;

    let confirm = check_unsaved_changes(&workspace);
    match confirm {
        ConfirmAction::Cancel => return,
        ConfirmAction::SavedThenProceed => {
            let tile_vec: Vec<_> = tiles.iter().map(|(p, d)| (*p, d.clone())).collect();
            let unit_vec: Vec<_> = units_q.iter().map(|(p, d)| (*p, d.clone())).collect();
            if !do_save(
                false,
                &mut workspace,
                &game_system,
                &entity_types,
                &enum_registry,
                &struct_registry,
                &concepts,
                &relations,
                &constraints,
                &turn_structure,
                &crt,
                &combat_modifiers,
                &config,
                &tile_vec,
                &unit_vec,
                &storage,
                &mut commands,
            ) {
                return; // Save cancelled or failed — abort close.
            }
        }
        ConfirmAction::Proceed => {}
    }

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

/// Propagates the `UndoStack`'s `has_new_records` flag to `Workspace.dirty`.
/// Runs every frame in `Update`. When new commands have been recorded,
/// sets dirty to true and acknowledges the records.
pub fn sync_dirty_flag(
    undo_stack: Option<ResMut<hexorder_contracts::undo_redo::UndoStack>>,
    mut workspace: ResMut<Workspace>,
) {
    let Some(mut undo_stack) = undo_stack else {
        return;
    };
    if undo_stack.has_new_records() {
        workspace.dirty = true;
        undo_stack.acknowledge_records();
    }
}

/// Updates the window title to reflect the current workspace name and dirty state.
/// Format: "Hexorder \u{2014} `ProjectName`" (clean) or "Hexorder \u{2014} `ProjectName`*" (dirty).
/// When the workspace has no name (launcher), the title is just "Hexorder".
pub fn sync_window_title(workspace: Res<Workspace>, mut windows: Query<&mut Window>) {
    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    let title = if workspace.name.is_empty() {
        "Hexorder".to_string()
    } else if workspace.dirty {
        format!("Hexorder \u{2014} {}*", workspace.name)
    } else {
        format!("Hexorder \u{2014} {}", workspace.name)
    };
    if window.title != title {
        window.title = title;
    }
}

/// Handles file commands dispatched via the shortcut registry.
/// Maps `CommandExecutedEvent` command IDs to persistence events.
pub fn handle_file_command(
    trigger: On<hexorder_contracts::shortcuts::CommandExecutedEvent>,
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
