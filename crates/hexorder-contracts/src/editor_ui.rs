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
    /// Two-click flow to assign edge annotations between adjacent hexes.
    /// First click selects a hex, second click on an adjacent hex assigns
    /// the active edge feature type to the shared boundary.
    EdgePaint,
    /// Two-click combat selection: first click assigns attacker, second
    /// click assigns defender. Only active during Combat phases.
    CombatSelect,
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

/// Tracks the currently selected hex edge, if any.
/// Used by the inspector to show edge feature details and by the edge paint
/// tool as a visual indicator of the first click in the two-click flow.
#[derive(Resource, Debug, Default, Reflect)]
pub struct SelectedEdge {
    /// The first hex clicked in the two-click edge selection flow.
    /// Set after the first click, cleared after the second click completes
    /// or when switching tools.
    pub first_hex: Option<super::hex_grid::HexPosition>,
    /// The fully selected edge (after both clicks).
    #[reflect(ignore)]
    pub edge: Option<super::hex_grid::HexEdge>,
}

/// Tracks which edge feature type the user is currently painting with.
/// Analogous to `ActiveBoardType` for cell painting.
#[derive(Resource, Debug, Default)]
pub struct ActiveEdgeType {
    /// Name of the edge feature type to assign (e.g., "River", "Road").
    pub type_name: Option<String>,
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
#[must_use]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_tool_default_is_select() {
        assert_eq!(EditorTool::default(), EditorTool::Select);
    }

    #[test]
    fn editor_tool_variants_are_distinct() {
        assert_ne!(EditorTool::Select, EditorTool::Paint);
        assert_ne!(EditorTool::Paint, EditorTool::Place);
        assert_ne!(EditorTool::Select, EditorTool::Place);
        assert_ne!(EditorTool::Select, EditorTool::EdgePaint);
        assert_ne!(EditorTool::Paint, EditorTool::EdgePaint);
        assert_ne!(EditorTool::Place, EditorTool::EdgePaint);
    }

    #[test]
    fn selected_edge_default_is_none() {
        let se = SelectedEdge::default();
        assert!(se.first_hex.is_none());
        assert!(se.edge.is_none());
    }

    #[test]
    fn active_edge_type_default_is_none() {
        let aet = ActiveEdgeType::default();
        assert!(aet.type_name.is_none());
    }

    #[test]
    fn viewport_margins_default_is_zero() {
        let m = ViewportMargins::default();
        assert!((m.left).abs() < f32::EPSILON);
        assert!((m.top).abs() < f32::EPSILON);
        assert!((m.right).abs() < f32::EPSILON);
        assert!((m.bottom).abs() < f32::EPSILON);
    }

    #[test]
    fn selection_default_is_empty() {
        let s = Selection::default();
        assert!(s.entities.is_empty());
    }

    #[test]
    fn toast_kind_debug() {
        let kinds = [ToastKind::Success, ToastKind::Error, ToastKind::Info];
        for kind in kinds {
            assert!(!format!("{kind:?}").is_empty());
        }
    }

    #[test]
    fn toast_event_construction() {
        let evt = ToastEvent {
            message: "Saved!".to_string(),
            kind: ToastKind::Success,
        };
        assert_eq!(evt.message, "Saved!");
        assert_eq!(evt.kind, ToastKind::Success);
    }

    #[test]
    fn viewport_rect_default_is_none() {
        let vr = ViewportRect::default();
        assert!(vr.0.is_none());
    }

    #[test]
    fn paint_preview_default_is_none() {
        let pp = PaintPreview::default();
        assert!(pp.material.is_none());
    }
}
