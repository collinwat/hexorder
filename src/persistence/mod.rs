//! Persistence feature plugin.
//!
//! Handles saving and loading game system definitions and board state
//! to `.hexorder` (RON) files. Provides keyboard shortcuts, file dialogs,
//! and the deferred board load pattern for reconstructing state after load.

use bevy::prelude::*;

use crate::contracts::persistence::{AppScreen, CurrentFilePath};

mod systems;

#[cfg(test)]
mod tests;

/// Plugin that manages file persistence, keyboard shortcuts,
/// and board state reconstruction after load.
#[derive(Debug)]
pub struct PersistencePlugin;

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentFilePath>();
        app.add_systems(
            Update,
            (
                systems::keyboard_shortcuts,
                systems::apply_pending_board_load,
            )
                .run_if(in_state(AppScreen::Editor)),
        );
        app.add_observer(systems::handle_save_request);
        app.add_observer(systems::handle_load_request);
        app.add_observer(systems::handle_new_project);
    }
}
