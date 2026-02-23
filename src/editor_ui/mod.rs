//! Editor UI plugin.
//!
//! Provides the egui-based editor interface with dark theme, tool mode selector,
//! cell type palette, cell type editor, property editors, and tile inspector.

use bevy::prelude::*;
use bevy::window::{MonitorSelection, WindowMode};
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

use crate::contracts::editor_ui::{
    EditorTool, Selection, ToastEvent, ViewportMargins, ViewportRect,
};
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

        // Do NOT use `enable_absorb_bevy_input_system = true`. That absorbs
        // both keyboard AND pointer input when egui claims the area. With
        // DockArea covering the full window, egui always claims pointer input,
        // which clears ButtonInput<MouseButton> and MouseWheel every frame —
        // breaking hex_grid clicks and camera zoom/pan.
        //
        // Instead, register a custom keyboard-only absorb system that clears
        // keyboard input when egui wants it (for text fields) but leaves
        // pointer input alone. Pointer gating is handled by the
        // `pointer_over_ui_panel` run condition on consumer systems.

        app.insert_resource(EditorTool::default());
        app.init_resource::<ViewportMargins>();
        app.insert_resource(components::EditorState::default());
        app.init_resource::<Selection>();
        app.init_resource::<components::DockLayoutState>();
        app.init_resource::<ViewportRect>();
        app.init_resource::<components::ToastState>();
        app.init_resource::<components::GridOverlayVisible>();
        app.init_resource::<ConceptRegistry>();
        app.init_resource::<RelationRegistry>();
        app.init_resource::<ConstraintRegistry>();
        app.init_resource::<SchemaValidation>();
        app.init_resource::<ActiveCombat>();
        app.init_resource::<TurnState>();

        // Disable egui's built-in Cmd+0 zoom-reset shortcut to prevent
        // Retina HiDPI jitter after native file dialogs on macOS.
        app.add_systems(
            EguiPrimaryContextPass,
            systems::disable_egui_zoom_shortcuts.run_if(run_once),
        );

        // Theme applies unconditionally so both launcher and editor get dark theming.
        app.add_systems(EguiPrimaryContextPass, systems::configure_theme);
        // Launcher screen shown only in Launcher state.
        app.add_systems(
            EguiPrimaryContextPass,
            systems::launcher_system.run_if(in_state(AppScreen::Launcher)),
        );
        // Editor dock system shown only in Editor state.
        // Single DockArea system replaces the four separate zone systems.
        // When the inspector feature is enabled, chain the debug panel after.
        #[cfg(not(feature = "inspector"))]
        app.add_systems(
            EguiPrimaryContextPass,
            (
                systems::editor_dock_system,
                systems::sync_workspace_preset,
                systems::sync_font_size,
                systems::update_viewport_margins,
            )
                .chain()
                .run_if(in_state(AppScreen::Editor)),
        );
        #[cfg(feature = "inspector")]
        app.add_systems(
            EguiPrimaryContextPass,
            (
                systems::editor_dock_system,
                systems::sync_workspace_preset,
                systems::sync_font_size,
                systems::debug_inspector_panel,
                systems::update_viewport_margins,
            )
                .chain()
                .run_if(in_state(AppScreen::Editor)),
        );
        // Restore workspace preset, dock layout, and font size on editor entry.
        // restore_dock_layout runs after restore_workspace_preset to override
        // the preset with the user's saved panel arrangement (if any).
        app.add_systems(
            OnEnter(AppScreen::Editor),
            (
                systems::restore_workspace_preset,
                systems::restore_dock_layout,
                systems::restore_font_size,
            )
                .chain(),
        );
        // Persist dock layout changes to config file.
        app.add_systems(
            PostUpdate,
            systems::save_dock_layout.run_if(in_state(AppScreen::Editor)),
        );
        // Play panel shown only in Play state.
        app.add_systems(
            EguiPrimaryContextPass,
            systems::play_panel_system.run_if(in_state(AppScreen::Play)),
        );

        app.add_observer(handle_editor_ui_command);
        app.add_observer(handle_toast_event);

        // Grid overlay renders only in Editor state.
        app.add_systems(
            EguiPrimaryContextPass,
            systems::render_grid_overlay.run_if(in_state(AppScreen::Editor)),
        );
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
    mut grid_overlay: ResMut<components::GridOverlayVisible>,
    tile_entities: Query<Entity, With<HexTile>>,
    mut commands: Commands,
    mut windows: Query<&mut Window>,
    mut dock_layout: ResMut<components::DockLayoutState>,
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
        "view.toggle_grid_overlay" => {
            grid_overlay.0 = !grid_overlay.0;
        }
        "help.about" => {
            editor_state.about_panel_visible = !editor_state.about_panel_visible;
        }
        "edit.deselect" => {
            // Escape exits fullscreen if active.
            if let Ok(mut window) = windows.single_mut()
                && window.mode != WindowMode::Windowed
            {
                window.mode = WindowMode::Windowed;
            }
        }
        // Workspace presets (Cmd+1–4).
        "workspace.map_editing" => {
            dock_layout.apply_preset(components::WorkspacePreset::MapEditing);
        }
        "workspace.unit_design" => {
            dock_layout.apply_preset(components::WorkspacePreset::UnitDesign);
        }
        "workspace.rule_authoring" => {
            dock_layout.apply_preset(components::WorkspacePreset::RuleAuthoring);
        }
        "workspace.playtesting" => {
            dock_layout.apply_preset(components::WorkspacePreset::Playtesting);
        }
        // Undo/redo handled by UndoRedoPlugin — no no-op fallback needed.
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
        bindings: vec![], // Cmd+1 reassigned to workspace.map_editing
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

    // Workspace presets (Cmd+1–4).
    registry.register(CommandEntry {
        id: CommandId("workspace.map_editing"),
        name: "Map Editing".to_string(),
        description: "Switch to Map Editing workspace".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::Digit1, Modifiers::CMD)],
        category: CommandCategory::View,
        continuous: false,
    });
    registry.register(CommandEntry {
        id: CommandId("workspace.unit_design"),
        name: "Unit Design".to_string(),
        description: "Switch to Unit Design workspace".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::Digit2, Modifiers::CMD)],
        category: CommandCategory::View,
        continuous: false,
    });
    registry.register(CommandEntry {
        id: CommandId("workspace.rule_authoring"),
        name: "Rule Authoring".to_string(),
        description: "Switch to Rule Authoring workspace".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::Digit3, Modifiers::CMD)],
        category: CommandCategory::View,
        continuous: false,
    });
    registry.register(CommandEntry {
        id: CommandId("workspace.playtesting"),
        name: "Playtesting".to_string(),
        description: "Switch to Playtesting workspace".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::Digit4, Modifiers::CMD)],
        category: CommandCategory::View,
        continuous: false,
    });

    // Edit actions (backing features pending — registered for discoverability).
    // Note: edit.undo and edit.redo are registered by undo_redo plugin.
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

    // Help.
    registry.register(CommandEntry {
        id: CommandId("help.about"),
        name: "About Hexorder".to_string(),
        description: "Show the About panel".to_string(),
        bindings: vec![],
        category: CommandCategory::View,
        continuous: false,
    });
}
