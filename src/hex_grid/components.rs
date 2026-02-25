//! Plugin-local components and resources for `hex_grid`.
//!
//! Contract types (`HexPosition`, `HexGridConfig`, `HexTile`, `SelectedHex`, etc.)
//! live in `hexorder_contracts::hex_grid`.
//! This module holds types that are internal to the `hex_grid` plugin.

use bevy::prelude::*;

use hexorder_contracts::hex_grid::HexPosition;

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

/// Stores material and mesh handles for indicator ring overlays.
/// The selection ring material is set once at spawn and never changes.
/// The hover ring material may change in Paint mode to preview the active color.
#[derive(Resource, Debug)]
pub struct IndicatorMaterials {
    /// Default hover ring material (used in Select mode).
    pub hover: Handle<StandardMaterial>,
    /// Multi-selection ring material (teal).
    pub multi_select: Handle<StandardMaterial>,
    /// Shared ring mesh handle for indicators.
    pub ring_mesh: Handle<Mesh>,
    /// Rotation quaternion for flat ring on ground plane.
    pub flat_rotation: Quat,
}

/// Marker component for multi-selection ring overlay entities.
/// Stores the tile entity this indicator belongs to for cleanup.
#[derive(Component, Debug)]
pub struct MultiSelectIndicator {
    pub tile_entity: Entity,
}

/// Stores material handles for move overlay rendering.
#[derive(Resource, Debug)]
pub struct OverlayMaterials {
    /// Semi-transparent green for valid move destinations.
    pub valid: Handle<StandardMaterial>,
    /// Semi-transparent red for blocked destinations.
    pub blocked: Handle<StandardMaterial>,
    /// Shared ring mesh handle for overlays.
    pub ring_mesh: Handle<Mesh>,
}
