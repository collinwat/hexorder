//! Feature-local components and resources for `hex_grid`.
//!
//! Contract types (`HexPosition`, `HexGridConfig`, `HexTile`, `SelectedHex`, etc.)
//! live in `crate::contracts::hex_grid`.
//! This module holds types that are internal to the `hex_grid` feature plugin.

use bevy::prelude::*;

use crate::contracts::hex_grid::HexPosition;

/// Tracks the hex tile currently under the mouse cursor, if any.
#[derive(Resource, Debug, Default)]
pub struct HoveredHex {
    pub position: Option<HexPosition>,
}

/// Stores the handle to the shared default material used for hex tile rendering.
#[derive(Resource, Debug)]
pub struct HexMaterials {
    /// Default tile color (light gray).
    pub default: Handle<StandardMaterial>,
}

/// Marker component for the hover ring overlay entity.
#[derive(Component, Debug)]
pub struct HoverIndicator;

/// Marker component for the selection ring overlay entity.
#[derive(Component, Debug)]
pub struct SelectIndicator;

/// Stores the default hover ring material handle.
/// The selection ring material is set once at spawn and never changes.
/// The hover ring material may change in Paint mode to preview the active color.
#[derive(Resource, Debug)]
pub struct IndicatorMaterials {
    /// Default hover ring material (used in Select mode).
    pub hover: Handle<StandardMaterial>,
}
