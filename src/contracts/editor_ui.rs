//! Shared editor UI types. See `.specs/contracts/editor_ui.md`.

use bevy::prelude::*;

/// The current editor tool mode. Other plugins (e.g., cell, unit) read this
/// to decide whether a click should select, paint, or place.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
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
#[derive(Resource, Debug, Default)]
pub struct PaintPreview {
    pub material: Option<Handle<StandardMaterial>>,
}
