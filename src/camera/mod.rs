//! Camera feature plugin.
//!
//! Provides a top-down orthographic camera locked perpendicular to the XZ ground plane.
//! Supports pan (middle-click drag, WASD, arrow keys) and zoom (scroll wheel).
//! No rotation is permitted.

use bevy::prelude::*;
use bevy_egui::input::{egui_wants_any_keyboard_input, egui_wants_any_pointer_input};

use crate::contracts::persistence::AppScreen;

mod components;
mod systems;
#[cfg(test)]
mod tests;

// Re-exported for other features that may query the camera entity or read camera state.
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
        app.init_resource::<CameraState>()
            .configure_sets(
                Update,
                CameraSet::Apply
                    .after(CameraSet::Input)
                    .run_if(in_state(AppScreen::Editor)),
            )
            .add_systems(Startup, systems::spawn_camera)
            .add_systems(
                OnEnter(AppScreen::Editor),
                systems::configure_bounds_from_grid,
            )
            .add_systems(
                Update,
                (
                    systems::keyboard_pan
                        .in_set(CameraSet::Input)
                        .run_if(not(egui_wants_any_keyboard_input)),
                    systems::view_shortcuts
                        .in_set(CameraSet::Input)
                        .run_if(not(egui_wants_any_keyboard_input)),
                    systems::mouse_pan
                        .in_set(CameraSet::Input)
                        .run_if(not(egui_wants_any_pointer_input)),
                    systems::scroll_zoom
                        .in_set(CameraSet::Input)
                        .run_if(not(egui_wants_any_pointer_input)),
                    systems::compensate_resize.in_set(CameraSet::Apply),
                    systems::smooth_camera
                        .in_set(CameraSet::Apply)
                        .after(systems::compensate_resize),
                )
                    .run_if(in_state(AppScreen::Editor)),
            );
    }
}
