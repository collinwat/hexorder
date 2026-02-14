//! Editor UI plugin.
//!
//! Provides the egui-based editor interface with dark theme, tool mode selector,
//! cell type palette, cell type editor, property editors, and tile inspector.

use bevy::prelude::*;
use bevy_egui::{EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass};

use crate::contracts::editor_ui::EditorTool;
use crate::contracts::ontology::{ConceptRegistry, ConstraintRegistry, RelationRegistry};
use crate::contracts::persistence::AppScreen;
use crate::contracts::validation::SchemaValidation;

mod components;
mod systems;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod ui_tests;

/// Plugin that provides the editor UI overlay via egui.
#[derive(Debug)]
pub struct EditorUiPlugin;

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default());

        // Absorb keyboard/pointer events from Bevy's input buffers when egui
        // has focus (e.g. text field active). Without this, Bevy's internal
        // systems can consume keyboard events before egui processes them,
        // preventing text input from working.
        app.world_mut()
            .resource_mut::<EguiGlobalSettings>()
            .enable_absorb_bevy_input_system = true;

        app.insert_resource(EditorTool::default());
        app.insert_resource(components::EditorState::default());
        app.init_resource::<ConceptRegistry>();
        app.init_resource::<RelationRegistry>();
        app.init_resource::<ConstraintRegistry>();
        app.init_resource::<SchemaValidation>();

        // Theme applies unconditionally so both launcher and editor get dark theming.
        app.add_systems(EguiPrimaryContextPass, systems::configure_theme);
        // Launcher screen shown only in Launcher state.
        app.add_systems(
            EguiPrimaryContextPass,
            systems::launcher_system.run_if(in_state(AppScreen::Launcher)),
        );
        // Editor panel shown only in Editor state.
        app.add_systems(
            EguiPrimaryContextPass,
            systems::editor_panel_system.run_if(in_state(AppScreen::Editor)),
        );
    }
}
