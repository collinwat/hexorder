//! Camera plugin.
//!
//! Provides a top-down orthographic camera locked perpendicular to the XZ ground plane.
//! Supports pan (middle-click drag, WASD, arrow keys) and zoom (scroll wheel).
//! No rotation is permitted.

use bevy::prelude::*;
use bevy_egui::input::egui_wants_any_keyboard_input;

use crate::contracts::editor_ui::{ViewportMargins, pointer_over_ui_panel};
use crate::contracts::persistence::AppScreen;
use crate::contracts::shortcuts::{
    CommandCategory, CommandEntry, CommandId, KeyBinding, Modifiers, ShortcutRegistry,
};

mod components;
mod systems;
#[cfg(test)]
mod tests;

// Re-exported for other plugins that may query the camera entity or read camera state.
#[allow(unused_imports)]
pub use components::{CameraState, TopDownCamera};

/// System sets for camera input vs. camera application ordering.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum CameraSet {
    /// Input processing: keyboard pan, mouse pan, scroll zoom.
    Input,
    /// Apply smoothed camera transform and projection updates.
    Apply,
}

/// Top-down orthographic camera plugin with pan and zoom.
#[derive(Debug)]
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        register_shortcuts(&mut app.world_mut().resource_mut::<ShortcutRegistry>());

        app.init_resource::<ViewportMargins>()
            .init_resource::<CameraState>()
            .configure_sets(
                Update,
                CameraSet::Apply
                    .after(CameraSet::Input)
                    .run_if(in_state(AppScreen::Editor).or(in_state(AppScreen::Play))),
            )
            .add_systems(Startup, systems::spawn_camera)
            .add_systems(
                OnEnter(AppScreen::Editor),
                systems::configure_bounds_from_grid,
            )
            .add_systems(
                Update,
                (
                    systems::apply_pending_reset.in_set(CameraSet::Input),
                    systems::keyboard_pan
                        .in_set(CameraSet::Input)
                        .run_if(not(egui_wants_any_keyboard_input)),
                    systems::mouse_pan
                        .in_set(CameraSet::Input)
                        .run_if(not(pointer_over_ui_panel)),
                    systems::scroll_zoom
                        .in_set(CameraSet::Input)
                        .run_if(not(pointer_over_ui_panel)),
                    systems::compensate_resize.in_set(CameraSet::Apply),
                    systems::smooth_camera
                        .in_set(CameraSet::Apply)
                        .after(systems::compensate_resize),
                )
                    .run_if(in_state(AppScreen::Editor).or(in_state(AppScreen::Play))),
            )
            .add_observer(systems::handle_camera_command);
    }
}

fn register_shortcuts(registry: &mut ShortcutRegistry) {
    use bevy::input::keyboard::KeyCode;

    // Continuous pan commands (held keys, not fired via `CommandExecutedEvent`).
    registry.register(CommandEntry {
        id: CommandId("camera.pan_up"),
        name: "Pan Up".to_string(),
        description: "Pan camera up".to_string(),
        bindings: vec![
            KeyBinding::new(KeyCode::KeyW, Modifiers::NONE),
            KeyBinding::new(KeyCode::ArrowUp, Modifiers::NONE),
        ],
        category: CommandCategory::Camera,
        continuous: true,
    });
    registry.register(CommandEntry {
        id: CommandId("camera.pan_down"),
        name: "Pan Down".to_string(),
        description: "Pan camera down".to_string(),
        bindings: vec![
            KeyBinding::new(KeyCode::KeyS, Modifiers::NONE),
            KeyBinding::new(KeyCode::ArrowDown, Modifiers::NONE),
        ],
        category: CommandCategory::Camera,
        continuous: true,
    });
    registry.register(CommandEntry {
        id: CommandId("camera.pan_left"),
        name: "Pan Left".to_string(),
        description: "Pan camera left".to_string(),
        bindings: vec![
            KeyBinding::new(KeyCode::KeyA, Modifiers::NONE),
            KeyBinding::new(KeyCode::ArrowLeft, Modifiers::NONE),
        ],
        category: CommandCategory::Camera,
        continuous: true,
    });
    registry.register(CommandEntry {
        id: CommandId("camera.pan_right"),
        name: "Pan Right".to_string(),
        description: "Pan camera right".to_string(),
        bindings: vec![
            KeyBinding::new(KeyCode::KeyD, Modifiers::NONE),
            KeyBinding::new(KeyCode::ArrowRight, Modifiers::NONE),
        ],
        category: CommandCategory::Camera,
        continuous: true,
    });

    // Discrete view commands (fired via `CommandExecutedEvent` on `just_pressed`).
    registry.register(CommandEntry {
        id: CommandId("camera.zoom_in"),
        name: "Zoom In".to_string(),
        description: "Zoom camera in".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::Equal, Modifiers::NONE)],
        category: CommandCategory::Camera,
        continuous: false,
    });
    registry.register(CommandEntry {
        id: CommandId("camera.zoom_out"),
        name: "Zoom Out".to_string(),
        description: "Zoom camera out".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::Minus, Modifiers::NONE)],
        category: CommandCategory::Camera,
        continuous: false,
    });
    registry.register(CommandEntry {
        id: CommandId("camera.center"),
        name: "Center View".to_string(),
        description: "Center camera on grid".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::KeyC, Modifiers::NONE)],
        category: CommandCategory::Camera,
        continuous: false,
    });
    registry.register(CommandEntry {
        id: CommandId("camera.fit"),
        name: "Fit View".to_string(),
        description: "Zoom to fit grid in viewport".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::KeyF, Modifiers::NONE)],
        category: CommandCategory::Camera,
        continuous: false,
    });
    registry.register(CommandEntry {
        id: CommandId("camera.reset_view"),
        name: "Reset View".to_string(),
        description: "Zoom to fit and center".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::Digit0, Modifiers::NONE)],
        category: CommandCategory::Camera,
        continuous: false,
    });
}
