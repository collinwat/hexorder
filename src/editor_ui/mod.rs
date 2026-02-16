//! Editor UI plugin.
//!
//! Provides the egui-based editor interface with dark theme, tool mode selector,
//! cell type palette, cell type editor, property editors, and tile inspector.

use bevy::prelude::*;
use bevy_egui::{EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass};

use crate::contracts::editor_ui::EditorTool;
use crate::contracts::mechanics::{ActiveCombat, TurnState};
use crate::contracts::ontology::{ConceptRegistry, ConstraintRegistry, RelationRegistry};
use crate::contracts::persistence::AppScreen;
use crate::contracts::shortcuts::{
    CommandCategory, CommandEntry, CommandExecutedEvent, CommandId, KeyBinding, Modifiers,
    ShortcutRegistry,
};
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
        register_shortcuts(&mut app.world_mut().resource_mut::<ShortcutRegistry>());

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
        app.init_resource::<ActiveCombat>();
        app.init_resource::<TurnState>();

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
        // Play panel shown only in Play state.
        app.add_systems(
            EguiPrimaryContextPass,
            systems::play_panel_system.run_if(in_state(AppScreen::Play)),
        );

        app.add_observer(handle_editor_ui_command);
    }
}

/// Observer: handles tool switching and mode commands from the shortcut registry.
fn handle_editor_ui_command(
    trigger: On<CommandExecutedEvent>,
    mut tool: ResMut<EditorTool>,
    mut next_state: ResMut<NextState<AppScreen>>,
) {
    match trigger.event().command_id.0 {
        "tool.select" => *tool = EditorTool::Select,
        "tool.paint" => *tool = EditorTool::Paint,
        "tool.place" => *tool = EditorTool::Place,
        "mode.editor" => next_state.set(AppScreen::Editor),
        "mode.play" => next_state.set(AppScreen::Launcher),
        _ => {}
    }
}

fn register_shortcuts(registry: &mut ShortcutRegistry) {
    use bevy::input::keyboard::KeyCode;

    // Tool switching.
    registry.register(CommandEntry {
        id: CommandId("tool.select"),
        name: "Select Tool".to_string(),
        description: "Switch to select mode".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::Digit1, Modifiers::NONE)],
        category: CommandCategory::Tool,
        continuous: false,
    });
    registry.register(CommandEntry {
        id: CommandId("tool.paint"),
        name: "Paint Tool".to_string(),
        description: "Switch to paint mode".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::Digit2, Modifiers::NONE)],
        category: CommandCategory::Tool,
        continuous: false,
    });
    registry.register(CommandEntry {
        id: CommandId("tool.place"),
        name: "Place Tool".to_string(),
        description: "Switch to place mode".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::Digit3, Modifiers::NONE)],
        category: CommandCategory::Tool,
        continuous: false,
    });

    // Mode switching.
    registry.register(CommandEntry {
        id: CommandId("mode.editor"),
        name: "Editor Mode".to_string(),
        description: "Switch to editor mode".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::Digit1, Modifiers::CMD)],
        category: CommandCategory::Mode,
        continuous: false,
    });
    registry.register(CommandEntry {
        id: CommandId("mode.play"),
        name: "Play Mode".to_string(),
        description: "Switch to launcher".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::Digit2, Modifiers::CMD)],
        category: CommandCategory::Mode,
        continuous: false,
    });
}
