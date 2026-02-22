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
    /// Height in logical pixels consumed by the bottom panel.
    pub bottom: f32,
}

impl Default for ViewportMargins {
    fn default() -> Self {
        Self {
            left: 0.0,
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
        }
    }
}

/// Multi-selection set for bulk operations (Shift+click, Cmd+A).
/// Coexists with `SelectedHex` — `SelectedHex` is the primary selection for
/// the inspector and single-tile operations; `Selection` is for bulk actions.
#[derive(Resource, Debug, Default)]
pub struct Selection {
    pub entities: std::collections::HashSet<Entity>,
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

/// Screen rect of the Viewport dock tab, updated each frame by `editor_ui`.
/// Used by `pointer_over_ui_panel` and viewport margin calculation.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct ViewportRect(pub Option<bevy_egui::egui::Rect>);

/// Returns `true` when the pointer is over a non-viewport UI panel.
///
/// Replacement for `egui_wants_any_pointer_input` which always returns `true`
/// when `DockArea` covers the full window. Uses Bevy's window cursor position
/// to avoid borrowing `EguiContexts` (which would conflict in run conditions).
pub fn pointer_over_ui_panel(
    viewport_rect: Res<ViewportRect>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
) -> bool {
    let Some(vp_rect) = viewport_rect.0 else {
        return false;
    };
    let Ok(window) = windows.single() else {
        return false;
    };
    let Some(cursor) = window.cursor_position() else {
        return false;
    };
    // Bevy cursor is (0,0) at top-left, Y increases downward — same as egui.
    let pos = bevy_egui::egui::Pos2::new(cursor.x, cursor.y);
    !vp_rect.contains(pos)
}
