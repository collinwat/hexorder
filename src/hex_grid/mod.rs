//! Hex grid plugin.
//!
//! Spawns a hexagonal grid on the XZ ground plane, handles tile selection
//! via mouse click, and provides hover feedback.

use bevy::prelude::*;
use bevy_egui::input::egui_wants_any_pointer_input;

use crate::contracts::persistence::AppScreen;
use crate::contracts::shortcuts::{
    CommandCategory, CommandEntry, CommandId, KeyBinding, Modifiers, ShortcutRegistry,
};

#[allow(dead_code)]
mod algorithms;
mod components;
mod systems;

#[cfg(test)]
mod tests;

/// Plugin that spawns and manages the hex grid.
#[derive(Debug)]
pub struct HexGridPlugin;

impl Plugin for HexGridPlugin {
    fn build(&self, app: &mut App) {
        register_shortcuts(&mut app.world_mut().resource_mut::<ShortcutRegistry>());

        app.add_systems(
            OnEnter(AppScreen::Editor),
            (
                systems::setup_grid_config,
                systems::setup_materials,
                systems::spawn_grid,
                systems::setup_indicators,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                systems::update_hover.run_if(not(egui_wants_any_pointer_input)),
                systems::handle_click.run_if(not(egui_wants_any_pointer_input)),
                systems::update_indicators,
                systems::sync_multi_select_indicators,
                systems::sync_move_overlays,
                systems::draw_los_ray,
            )
                .chain()
                .run_if(in_state(AppScreen::Editor)),
        )
        .add_systems(
            OnExit(AppScreen::Editor),
            systems::cleanup_internal_entities,
        )
        .add_observer(systems::handle_hex_grid_command);
    }
}

fn register_shortcuts(registry: &mut ShortcutRegistry) {
    use bevy::input::keyboard::KeyCode;

    registry.register(CommandEntry {
        id: CommandId("edit.deselect"),
        name: "Deselect".to_string(),
        description: "Clear current selection".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::Escape, Modifiers::NONE)],
        category: CommandCategory::Edit,
        continuous: false,
    });
}
