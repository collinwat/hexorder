//! Editor UI plugin.
//!
//! Provides the egui-based editor interface with dark theme, tool mode selector,
//! cell type palette, cell type editor, property editors, and tile inspector.

use bevy::prelude::*;
use bevy::window::{MonitorSelection, WindowMode};
use bevy_egui::{EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass};

use crate::contracts::editor_ui::{EditorTool, Selection, ToastEvent, ViewportMargins};
use crate::contracts::game_system::SelectedUnit;
use crate::contracts::hex_grid::HexTile;
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
        app.init_resource::<ViewportMargins>();
        app.insert_resource(components::EditorState::default());
        app.init_resource::<Selection>();
        app.init_resource::<components::ToastState>();
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
        // When the inspector feature is enabled, chain the debug panel before
        // update_viewport_margins so available_rect() reflects both side panels.
        #[cfg(not(feature = "inspector"))]
        app.add_systems(
            EguiPrimaryContextPass,
            (
                systems::editor_panel_system,
                systems::update_viewport_margins,
            )
                .chain()
                .run_if(in_state(AppScreen::Editor)),
        );
        #[cfg(feature = "inspector")]
        app.add_systems(
            EguiPrimaryContextPass,
            (
                systems::editor_panel_system,
                systems::debug_inspector_panel,
                systems::update_viewport_margins,
            )
                .chain()
                .run_if(in_state(AppScreen::Editor)),
        );
        // Play panel shown only in Play state.
        app.add_systems(
            EguiPrimaryContextPass,
            systems::play_panel_system.run_if(in_state(AppScreen::Play)),
        );

        app.add_observer(handle_editor_ui_command);
        app.add_observer(handle_toast_event);

        // Toast renders on all screens (Editor and Play).
        app.add_systems(EguiPrimaryContextPass, systems::render_toast);
    }
}

/// Observer: handles tool switching, mode switching, and discoverable commands.
#[allow(clippy::too_many_arguments)] // Tracked by #115 — decompose editor observers.
fn handle_editor_ui_command(
    trigger: On<CommandExecutedEvent>,
    mut tool: ResMut<EditorTool>,
    mut next_state: ResMut<NextState<AppScreen>>,
    mut selected_unit: ResMut<SelectedUnit>,
    mut editor_state: ResMut<components::EditorState>,
    mut selection: ResMut<Selection>,
    tile_entities: Query<Entity, With<HexTile>>,
    mut commands: Commands,
    mut windows: Query<&mut Window>,
) {
    match trigger.event().command_id.0 {
        "tool.select" => *tool = EditorTool::Select,
        "tool.paint" => *tool = EditorTool::Paint,
        "tool.place" => *tool = EditorTool::Place,
        "mode.editor" => next_state.set(AppScreen::Editor),
        "mode.close" => next_state.set(AppScreen::Launcher),
        "edit.delete" => {
            // Bulk delete multi-selected entities first.
            if !selection.entities.is_empty() {
                for entity in selection.entities.drain() {
                    commands.entity(entity).despawn();
                }
            } else if let Some(entity) = selected_unit.entity {
                commands.entity(entity).despawn();
                selected_unit.entity = None;
            }
        }
        "edit.select_all" => {
            selection.entities.clear();
            for entity in &tile_entities {
                selection.entities.insert(entity);
            }
        }
        "view.toggle_inspector" => {
            editor_state.inspector_visible = !editor_state.inspector_visible;
        }
        "view.toggle_toolbar" => {
            editor_state.toolbar_visible = !editor_state.toolbar_visible;
        }
        "view.toggle_debug_panel" => {
            editor_state.debug_panel_visible = !editor_state.debug_panel_visible;
        }
        "view.toggle_fullscreen" => {
            if let Ok(mut window) = windows.single_mut() {
                window.mode = match window.mode {
                    WindowMode::Windowed => {
                        WindowMode::BorderlessFullscreen(MonitorSelection::Current)
                    }
                    _ => WindowMode::Windowed,
                };
            }
        }
        // Discoverable no-ops — registered for palette visibility, backing features pending.
        "edit.undo" | "edit.redo" | "view.toggle_grid_overlay" => {
            info!(
                "Command '{}' is not yet implemented",
                trigger.event().command_id.0
            );
        }
        _ => {}
    }
}

/// Observer: handles toast notification events and populates the toast state.
fn handle_toast_event(trigger: On<ToastEvent>, mut toast_state: ResMut<components::ToastState>) {
    let event = trigger.event();
    toast_state.active = Some(components::ActiveToast {
        message: event.message.clone(),
        kind: event.kind,
        remaining: 2.5,
    });
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
        id: CommandId("mode.close"),
        name: "Close".to_string(),
        description: "Close project and return to launcher".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::KeyW, Modifiers::CMD)],
        category: CommandCategory::Mode,
        continuous: false,
    });

    // Edit actions (backing features pending — registered for discoverability).
    registry.register(CommandEntry {
        id: CommandId("edit.undo"),
        name: "Undo".to_string(),
        description: "Undo last action".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::KeyZ, Modifiers::CMD)],
        category: CommandCategory::Edit,
        continuous: false,
    });
    registry.register(CommandEntry {
        id: CommandId("edit.redo"),
        name: "Redo".to_string(),
        description: "Redo last undone action".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::KeyZ, Modifiers::CMD_SHIFT)],
        category: CommandCategory::Edit,
        continuous: false,
    });
    registry.register(CommandEntry {
        id: CommandId("edit.select_all"),
        name: "Select All".to_string(),
        description: "Select all elements".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::KeyA, Modifiers::CMD)],
        category: CommandCategory::Edit,
        continuous: false,
    });
    registry.register(CommandEntry {
        id: CommandId("edit.delete"),
        name: "Delete Selection".to_string(),
        description: "Delete selected element".to_string(),
        bindings: vec![
            KeyBinding::new(KeyCode::Backspace, Modifiers::NONE),
            KeyBinding::new(KeyCode::Delete, Modifiers::NONE),
        ],
        category: CommandCategory::Edit,
        continuous: false,
    });

    // View toggles (backing features pending — registered for discoverability).
    registry.register(CommandEntry {
        id: CommandId("view.toggle_inspector"),
        name: "Toggle Inspector".to_string(),
        description: "Show or hide the inspector panel".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::KeyI, Modifiers::CMD)],
        category: CommandCategory::View,
        continuous: false,
    });
    registry.register(CommandEntry {
        id: CommandId("view.toggle_toolbar"),
        name: "Toggle Toolbar".to_string(),
        description: "Show or hide the toolbar".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::KeyT, Modifiers::CMD)],
        category: CommandCategory::View,
        continuous: false,
    });
    #[cfg(feature = "inspector")]
    registry.register(CommandEntry {
        id: CommandId("view.toggle_debug_panel"),
        name: "Toggle Debug Panel".to_string(),
        description: "Show or hide the debug inspector panel".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::Backquote, Modifiers::NONE)],
        category: CommandCategory::View,
        continuous: false,
    });
    registry.register(CommandEntry {
        id: CommandId("view.toggle_grid_overlay"),
        name: "Toggle Grid Overlay".to_string(),
        description: "Show or hide the grid overlay".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::KeyG, Modifiers::NONE)],
        category: CommandCategory::View,
        continuous: false,
    });
    registry.register(CommandEntry {
        id: CommandId("view.zoom_to_selection"),
        name: "Zoom to Selection".to_string(),
        description: "Zoom camera to the selected element".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::KeyZ, Modifiers::NONE)],
        category: CommandCategory::View,
        continuous: false,
    });
    registry.register(CommandEntry {
        id: CommandId("view.toggle_fullscreen"),
        name: "Toggle Fullscreen".to_string(),
        description: "Toggle fullscreen mode".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::KeyF, Modifiers::CMD)],
        category: CommandCategory::View,
        continuous: false,
    });
}
