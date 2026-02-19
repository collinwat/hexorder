//! Shared editor UI types. See `docs/contracts/editor-ui.md`.

use bevy::prelude::*;

/// The current editor tool mode. Other plugins (e.g., cell, unit) read this
/// to decide whether a click should select, paint, or place.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum EditorTool {
    /// Click to select hex tiles or units. Also handles unit movement.
    #[default]
    Select,
    /// Click to paint cell types onto hex tiles.
    Paint,
    /// Click to place unit tokens on hex tiles.
    Place,
}

/// Holds the material handle for the currently active paint color.
/// Updated by the cell plugin when the active cell type changes.
/// Read by `hex_grid` to show a paint preview on hover in Paint mode.
#[derive(Resource, Debug, Default, Reflect)]
pub struct PaintPreview {
    pub material: Option<Handle<StandardMaterial>>,
}

/// Pixel margins consumed by the editor UI panels.
/// Written by `editor_ui` each frame after egui panels render.
/// Read by `camera` to compute viewport-aware centering.
#[derive(Resource, Debug, Clone, Copy)]
pub struct ViewportMargins {
    /// Width in logical pixels consumed by the left side panel.
    pub left: f32,
    /// Height in logical pixels consumed by the top menu bar.
    pub top: f32,
    /// Width in logical pixels consumed by the right side panel (e.g. debug inspector).
    pub right: f32,
}

impl Default for ViewportMargins {
    fn default() -> Self {
        Self {
            left: 0.0,
            top: 0.0,
            right: 0.0,
        }
    }
}

/// Toast notification event. Fire from any plugin to show a toast message.
#[derive(Event, Debug, Clone)]
pub struct ToastEvent {
    pub message: String,
    pub kind: ToastKind,
}

/// The visual style of a toast notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Success,
    Error,
    Info,
}
