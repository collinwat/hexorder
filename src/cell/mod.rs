//! Cell plugin.
//!
//! Replaces the 0.1.0 terrain plugin. Handles painting user-defined cell types
//! onto hex tiles and syncing their visual appearance. Cell type definitions
//! come from the `game_system` plugin's registry.

use bevy::prelude::*;

use crate::contracts::editor_ui::PaintPreview;
use crate::contracts::persistence::AppScreen;

mod components;
mod systems;

#[cfg(test)]
mod tests;

/// Plugin that manages cell type materials, painting, and visual sync.
#[derive(Debug)]
pub struct CellPlugin;

impl Plugin for CellPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PaintPreview>()
            .add_systems(OnEnter(AppScreen::Editor), systems::setup_cell_materials)
            .add_systems(
                Update,
                (
                    systems::assign_default_cell_data,
                    systems::sync_cell_materials,
                    systems::sync_cell_visuals,
                    systems::update_paint_preview,
                )
                    .chain()
                    .run_if(in_state(AppScreen::Editor).or(in_state(AppScreen::Play))),
            )
            .add_observer(systems::paint_cell);
    }
}
