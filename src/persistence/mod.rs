//! Persistence plugin.
//!
//! Handles saving and loading game system definitions and board state
//! to `.hexorder` (RON) files. Provides keyboard shortcuts, file dialogs,
//! and the deferred board load pattern for reconstructing state after load.

use bevy::prelude::*;

use crate::contracts::persistence::{AppScreen, Workspace};
use crate::contracts::storage::{Storage, StorageConfig};

pub(crate) mod storage;
mod systems;

#[cfg(test)]
mod tests;

/// Plugin that manages file persistence, keyboard shortcuts,
/// and board state reconstruction after load.
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
        app.add_systems(
            Update,
            (
                systems::keyboard_shortcuts,
                systems::apply_pending_board_load,
            )
                .run_if(in_state(AppScreen::Editor).or(in_state(AppScreen::Play))),
        );
        app.add_observer(systems::handle_save_request);
        app.add_observer(systems::handle_load_request);
        app.add_observer(systems::handle_new_project);
        app.add_observer(systems::handle_close_project);
        app.add_systems(OnExit(AppScreen::Editor), systems::cleanup_editor_entities);
    }
}
