//! Persistence plugin.
//!
//! Handles saving and loading game system definitions and board state
//! to `.hexorder` (RON) files. Provides file dialogs and the deferred
//! board load pattern for reconstructing state after load.
//!
//! Keyboard shortcuts (Cmd+S, Cmd+O, Cmd+N) are registered with the
//! `ShortcutRegistry` and dispatched via `CommandExecutedEvent`.

use bevy::prelude::*;

use hexorder_contracts::persistence::{AppScreen, Workspace};
use hexorder_contracts::shortcuts::{
    CommandCategory, CommandEntry, CommandId, KeyBinding, Modifiers, ShortcutRegistry,
};
use hexorder_contracts::storage::{Storage, StorageConfig};

pub(crate) mod storage;
mod systems;

#[cfg(test)]
mod tests;

/// Plugin that manages file persistence and board state reconstruction.
#[derive(Debug)]
pub struct PersistencePlugin;

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        // Use pre-inserted config if the caller provided one,
        // otherwise resolve from compile-time feature flags.
        let config = app
            .world()
            .get_resource::<StorageConfig>()
            .cloned()
            .unwrap_or_else(storage::resolve_storage_config);
        let provider = storage::FilesystemProvider::new(config.clone());
        app.insert_resource(config);
        app.insert_resource(Storage::new(Box::new(provider)));

        app.init_resource::<Workspace>();

        // Register file shortcuts with the central registry.
        let mut registry = app.world_mut().resource_mut::<ShortcutRegistry>();
        register_shortcuts(&mut registry);
        app.add_systems(
            Update,
            (
                systems::apply_pending_board_load
                    .run_if(in_state(AppScreen::Editor).or(in_state(AppScreen::Play))),
                systems::sync_dirty_flag
                    .run_if(in_state(AppScreen::Editor).or(in_state(AppScreen::Play))),
            ),
        );
        app.add_observer(systems::handle_save_request);
        app.add_observer(systems::handle_load_request);
        app.add_observer(systems::handle_new_project);
        app.add_observer(systems::handle_close_project);
        app.add_observer(systems::handle_file_command);
        app.add_systems(OnExit(AppScreen::Editor), systems::cleanup_editor_entities);
    }
}

fn register_shortcuts(registry: &mut ShortcutRegistry) {
    registry.register(CommandEntry {
        id: CommandId("file.save"),
        name: "Save".to_string(),
        description: "Save to current file".to_string(),
        bindings: vec![KeyBinding::new(
            bevy::input::keyboard::KeyCode::KeyS,
            Modifiers::CMD,
        )],
        category: CommandCategory::File,
        continuous: false,
    });
    registry.register(CommandEntry {
        id: CommandId("file.save_as"),
        name: "Save As".to_string(),
        description: "Save to a new file".to_string(),
        bindings: vec![KeyBinding::new(
            bevy::input::keyboard::KeyCode::KeyS,
            Modifiers::CMD_SHIFT,
        )],
        category: CommandCategory::File,
        continuous: false,
    });
    registry.register(CommandEntry {
        id: CommandId("file.open"),
        name: "Open".to_string(),
        description: "Open a file".to_string(),
        bindings: vec![KeyBinding::new(
            bevy::input::keyboard::KeyCode::KeyO,
            Modifiers::CMD,
        )],
        category: CommandCategory::File,
        continuous: false,
    });
    registry.register(CommandEntry {
        id: CommandId("file.new"),
        name: "New Project".to_string(),
        description: "Create a new project".to_string(),
        bindings: vec![KeyBinding::new(
            bevy::input::keyboard::KeyCode::KeyN,
            Modifiers::CMD,
        )],
        category: CommandCategory::File,
        continuous: false,
    });
}
