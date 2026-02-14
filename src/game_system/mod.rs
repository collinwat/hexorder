//! Game System feature plugin.
//!
//! Provides the Game System container and the unified `EntityTypeRegistry`.
//! This is the root design artifact that holds all user-defined definitions
//! for a hex board game system.

use bevy::prelude::*;

use crate::contracts::game_system::{ActiveBoardType, ActiveTokenType, EntityRole, SelectedUnit};

mod systems;

// Re-export factory functions for use by persistence plugin.
pub(crate) use systems::{create_entity_type_registry, create_game_system};

#[cfg(test)]
mod tests;

/// Plugin that initializes the Game System, registry, and active-selection
/// resources at build time so they are immediately available to downstream plugins.
#[derive(Debug)]
pub struct GameSystemPlugin;

impl Plugin for GameSystemPlugin {
    fn build(&self, app: &mut App) {
        let registry = systems::create_entity_type_registry();
        let first_board_id = registry
            .first_by_role(EntityRole::BoardPosition)
            .map(|t| t.id);
        let first_token_id = registry.first_by_role(EntityRole::Token).map(|t| t.id);

        app.insert_resource(systems::create_game_system());
        app.insert_resource(registry);
        app.insert_resource(ActiveBoardType {
            entity_type_id: first_board_id,
        });
        app.insert_resource(ActiveTokenType {
            entity_type_id: first_token_id,
        });
        app.insert_resource(SelectedUnit::default());
    }
}
