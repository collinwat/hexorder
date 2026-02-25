//! Systems for the persistence plugin.

use std::collections::HashMap;

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

use crate::persistence::async_dialog::{
    AsyncDialogTask, ConfirmChoice, DialogCompleted, DialogKind, DialogResult, PendingAction,
    spawn_confirm_dialog, spawn_open_dialog,
};

// ---------------------------------------------------------------------------
// Shared Helpers
// ---------------------------------------------------------------------------

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

/// Build a `GameSystemFile` from current world state and pre-collected board data.
fn build_game_system_file(
    world: &World,
    tiles: &[(HexPosition, EntityData)],
    units: &[(HexPosition, EntityData)],
) -> GameSystemFile {
    let workspace = world.resource::<Workspace>();
    let game_system = world.resource::<GameSystem>();
    let entity_types = world.resource::<EntityTypeRegistry>();
    let enum_registry = world.resource::<EnumRegistry>();
    let struct_registry = world.resource::<StructRegistry>();
    let concepts = world.resource::<ConceptRegistry>();
    let relations = world.resource::<RelationRegistry>();
    let constraints = world.resource::<ConstraintRegistry>();
    let turn_structure = world.resource::<TurnStructure>();
    let crt = world.resource::<CombatResultsTable>();
    let combat_modifiers = world.resource::<CombatModifierRegistry>();
    let config = world.resource::<HexGridConfig>();

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

    GameSystemFile {
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
    }
}

/// Save the current project to the given path. Returns `true` on success.
/// Updates workspace path and dirty flag on success.
/// No dialog logic — pure file I/O and state update.
pub(crate) fn save_to_path(path: &std::path::Path, world: &mut World) -> bool {
    // Collect board data via queries (releases world borrow after each block).
    let tiles: Vec<(HexPosition, EntityData)> = {
        let mut q = world.query_filtered::<(&HexPosition, &EntityData), With<HexTile>>();
        q.iter(world).map(|(p, d)| (*p, d.clone())).collect()
    };
    let units: Vec<(HexPosition, EntityData)> = {
        let mut q = world.query_filtered::<(&HexPosition, &EntityData), With<UnitInstance>>();
        q.iter(world).map(|(p, d)| (*p, d.clone())).collect()
    };

    let file = build_game_system_file(world, &tiles, &units);

    // Write to disk — scope the storage borrow.
    let write_result = {
        let storage = world.resource::<Storage>();
        storage.provider().save_at(path, &file)
    };

    match write_result {
        Ok(()) => {
            info!("Saved to {}", path.display());
            let mut workspace = world.resource_mut::<Workspace>();
            workspace.file_path = Some(path.to_path_buf());
            workspace.dirty = false;
            drop(workspace);

            world.trigger(ToastEvent {
                message: "Project saved".to_string(),
                kind: ToastKind::Success,
            });
            true
        }
        Err(e) => {
            error!("Failed to save: {e}");
            world.trigger(ToastEvent {
                message: format!("Save failed: {e}"),
                kind: ToastKind::Error,
            });
            false
        }
    }
}

/// Load a project from the given path. Returns `true` on success.
/// Overwrites all registries, updates workspace, inserts `PendingBoardLoad`,
/// and transitions to Editor state. No dialog logic.
pub(crate) fn load_from_path(path: &std::path::Path, world: &mut World) -> bool {
    // Read file from disk — scope the storage borrow.
    let file = {
        let storage = world.resource::<Storage>();
        storage.provider().load(path)
    };

    let file = match file {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to load: {e}");
            world.trigger(ToastEvent {
                message: format!("Load failed: {e}"),
                kind: ToastKind::Error,
            });
            return false;
        }
    };

    // Overwrite registries.
    *world.resource_mut::<GameSystem>() = file.game_system;
    *world.resource_mut::<EntityTypeRegistry>() = file.entity_types;
    *world.resource_mut::<EnumRegistry>() = file.enums;
    *world.resource_mut::<StructRegistry>() = file.structs;
    *world.resource_mut::<ConceptRegistry>() = file.concepts;
    *world.resource_mut::<RelationRegistry>() = file.relations;
    *world.resource_mut::<ConstraintRegistry>() = file.constraints;
    *world.resource_mut::<TurnStructure>() = file.turn_structure;
    *world.resource_mut::<CombatResultsTable>() = file.combat_results_table;
    *world.resource_mut::<CombatModifierRegistry>() = file.combat_modifiers;
    *world.resource_mut::<SchemaValidation>() = SchemaValidation::default();

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

    {
        let mut workspace = world.resource_mut::<Workspace>();
        workspace.name = name;
        workspace.file_path = Some(path.to_path_buf());
        workspace.dirty = false;
        workspace.workspace_preset = file.workspace_preset;
        workspace.font_size_base = file.font_size_base;
    }

    // Insert pending board load for deferred application.
    world.insert_resource(PendingBoardLoad {
        tiles: file.tiles,
        units: file.units,
    });

    // Transition to editor (may already be in editor if loading from editor).
    world
        .resource_mut::<NextState<AppScreen>>()
        .set(AppScreen::Editor);

    world.trigger(ToastEvent {
        message: "Project loaded".to_string(),
        kind: ToastKind::Success,
    });

    let gs_id = world.resource::<GameSystem>().id.clone();
    info!("Loaded game system: {gs_id}");

    true
}

/// Reset all registries and derived state to factory defaults using world access.
fn reset_all_registries_world(world: &mut World) {
    *world.resource_mut::<GameSystem>() = crate::game_system::create_game_system();
    *world.resource_mut::<EntityTypeRegistry>() = crate::game_system::create_entity_type_registry();
    *world.resource_mut::<EnumRegistry>() = crate::game_system::create_enum_registry();
    *world.resource_mut::<StructRegistry>() = StructRegistry::default();
    *world.resource_mut::<ConceptRegistry>() = ConceptRegistry::default();
    *world.resource_mut::<RelationRegistry>() = RelationRegistry::default();
    *world.resource_mut::<ConstraintRegistry>() = ConstraintRegistry::default();
    *world.resource_mut::<SchemaValidation>() = SchemaValidation::default();
    world.resource_mut::<SelectedUnit>().entity = None;
}

/// Reset all state and initialize a new project with the given name.
fn reset_to_new_project(name: &str, world: &mut World) {
    reset_all_registries_world(world);

    // Reset mechanics to factory defaults.
    *world.resource_mut::<TurnStructure>() = crate::game_system::create_default_turn_structure();
    *world.resource_mut::<CombatResultsTable>() = crate::game_system::create_default_crt();
    *world.resource_mut::<CombatModifierRegistry>() = CombatModifierRegistry::default();
    *world.resource_mut::<TurnState>() = TurnState::default();
    *world.resource_mut::<ActiveCombat>() = ActiveCombat::default();

    {
        let mut workspace = world.resource_mut::<Workspace>();
        workspace.name = name.to_string();
        workspace.file_path = None;
        workspace.dirty = false;
        workspace.workspace_preset = String::new();
        workspace.font_size_base = 15.0;
    }

    world
        .resource_mut::<NextState<AppScreen>>()
        .set(AppScreen::Editor);
}

/// Reset all state and return to the launcher screen.
fn close_project(world: &mut World) {
    *world.resource_mut::<Workspace>() = Workspace::default();
    reset_all_registries_world(world);
    world
        .resource_mut::<NextState<AppScreen>>()
        .set(AppScreen::Launcher);
}

/// Spawn an async save dialog configured for the current project.
/// `then` specifies what to do after the save completes (if anything).
fn spawn_save_dialog_for_current_project(world: &mut World, then: Option<PendingAction>) {
    let (file_name, dir_from_workspace) = {
        let workspace = world.resource::<Workspace>();
        let sanitized = sanitize_filename(&workspace.name);
        let file_name = format!("{sanitized}.hexorder");
        let dir = workspace
            .file_path
            .as_ref()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf());
        (file_name, dir)
    };

    let initial_dir = dir_from_workspace.or_else(|| {
        let storage = world.resource::<Storage>();
        let base = storage.provider().base_dir().to_path_buf();
        std::fs::create_dir_all(&base).ok()?;
        Some(base)
    });

    let task = super::async_dialog::spawn_save_dialog(initial_dir.as_deref(), &file_name);
    world.insert_resource(AsyncDialogTask {
        kind: DialogKind::SaveFile {
            save_as: true,
            then,
        },
        task,
    });
}

/// Dispatch a pending action after a dialog chain resolves.
fn execute_pending_action(action: PendingAction, world: &mut World) {
    match action {
        PendingAction::Load => {
            // Spawn async open-file dialog.
            let task = spawn_open_dialog();
            world.insert_resource(AsyncDialogTask {
                kind: DialogKind::OpenFile,
                task,
            });
        }
        PendingAction::NewProject { name } => {
            reset_to_new_project(&name, world);
        }
        PendingAction::CloseProject => {
            close_project(world);
        }
    }
}

/// Central router for dialog completion results. Handles all dialog kind + result
/// combinations including dialog chaining (confirm → save → action).
pub(crate) fn dispatch_dialog_result(kind: DialogKind, result: DialogResult, world: &mut World) {
    match (kind, result) {
        // --- Confirm Unsaved Changes ---
        (DialogKind::ConfirmUnsavedChanges { then }, DialogResult::Confirmed(choice)) => {
            match choice {
                ConfirmChoice::Yes => {
                    // Save first, then execute the pending action.
                    let maybe_path = world.resource::<Workspace>().file_path.clone();
                    if let Some(path) = maybe_path {
                        if save_to_path(&path, world) {
                            execute_pending_action(then, world);
                        }
                        // Save failed → abort chain.
                    } else {
                        // No existing path — spawn save-as dialog with chained action.
                        spawn_save_dialog_for_current_project(world, Some(then));
                    }
                }
                ConfirmChoice::No => {
                    // Skip save, execute the pending action directly.
                    execute_pending_action(then, world);
                }
                ConfirmChoice::Cancel => {
                    // User cancelled — do nothing.
                }
            }
        }

        // --- Save File ---
        (DialogKind::SaveFile { then, .. }, DialogResult::FilePicked(Some(path))) => {
            if save_to_path(&path, world) {
                if let Some(action) = then {
                    execute_pending_action(action, world);
                }
            }
        }
        (DialogKind::SaveFile { .. }, DialogResult::FilePicked(None)) => {
            // User cancelled save dialog — abort chain.
        }

        // --- Open File ---
        (DialogKind::OpenFile, DialogResult::FilePicked(Some(path))) => {
            load_from_path(&path, world);
        }
        (DialogKind::OpenFile, DialogResult::FilePicked(None)) => {
            // User cancelled — do nothing.
        }

        // --- Unhandled combinations ---
        (kind, result) => {
            warn!("Unhandled dialog completion: {kind:?} + {result:?}");
        }
    }
}

/// Observer for `DialogCompleted` events. Clones event data and queues an
/// exclusive-world command for dispatch (observers can't take `&mut World`
/// alongside `On<E>`).
pub(crate) fn handle_dialog_completed(trigger: On<DialogCompleted>, mut commands: Commands) {
    let kind = trigger.event().kind.clone();
    let result = trigger.event().result.clone();

    commands.queue(move |world: &mut World| {
        dispatch_dialog_result(kind, result, world);
    });
}

// ---------------------------------------------------------------------------
// Observer Systems
// ---------------------------------------------------------------------------

/// Handles save requests. If the workspace has a path and this is not save-as,
/// saves directly. Otherwise spawns an async save dialog.
pub fn handle_save_request(trigger: On<SaveRequestEvent>, mut commands: Commands) {
    let save_as = trigger.event().save_as;
    commands.queue(move |world: &mut World| {
        if world.contains_resource::<AsyncDialogTask>() {
            return; // Dialog already open.
        }

        let maybe_path = if save_as {
            None
        } else {
            world.resource::<Workspace>().file_path.clone()
        };

        if let Some(path) = maybe_path {
            save_to_path(&path, world);
        } else {
            spawn_save_dialog_for_current_project(world, None);
        }
    });
}

/// Handles load requests. If the workspace is dirty, spawns a confirm dialog
/// first. Otherwise spawns an async open-file dialog directly.
pub fn handle_load_request(_trigger: On<LoadRequestEvent>, mut commands: Commands) {
    commands.queue(move |world: &mut World| {
        if world.contains_resource::<AsyncDialogTask>() {
            return;
        }

        let dirty = world.resource::<Workspace>().dirty;
        if dirty {
            let task = spawn_confirm_dialog();
            world.insert_resource(AsyncDialogTask {
                kind: DialogKind::ConfirmUnsavedChanges {
                    then: PendingAction::Load,
                },
                task,
            });
        } else {
            let task = spawn_open_dialog();
            world.insert_resource(AsyncDialogTask {
                kind: DialogKind::OpenFile,
                task,
            });
        }
    });
}

/// Handles new project requests. If the workspace is dirty, spawns a confirm
/// dialog first. Otherwise resets to a new project directly.
pub fn handle_new_project(trigger: On<NewProjectEvent>, mut commands: Commands) {
    let name = trigger.event().name.clone();
    commands.queue(move |world: &mut World| {
        if world.contains_resource::<AsyncDialogTask>() {
            return;
        }

        let dirty = world.resource::<Workspace>().dirty;
        if dirty {
            let task = spawn_confirm_dialog();
            world.insert_resource(AsyncDialogTask {
                kind: DialogKind::ConfirmUnsavedChanges {
                    then: PendingAction::NewProject { name },
                },
                task,
            });
        } else {
            reset_to_new_project(&name, world);
        }
    });
}

/// Handles close project requests. If the workspace is dirty, spawns a confirm
/// dialog first. Otherwise closes the project directly.
pub fn handle_close_project(_trigger: On<CloseProjectEvent>, mut commands: Commands) {
    commands.queue(move |world: &mut World| {
        if world.contains_resource::<AsyncDialogTask>() {
            return;
        }

        let dirty = world.resource::<Workspace>().dirty;
        if dirty {
            let task = spawn_confirm_dialog();
            world.insert_resource(AsyncDialogTask {
                kind: DialogKind::ConfirmUnsavedChanges {
                    then: PendingAction::CloseProject,
                },
                task,
            });
        } else {
            close_project(world);
        }
    });
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
