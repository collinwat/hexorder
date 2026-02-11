//! Shared hex grid types. See `.specs/contracts/hex_grid.md`.

use bevy::prelude::*;

/// Re-export `hexx::Hex` for coordinate math.
pub use hexx::Hex;

/// Marks an entity as occupying a hex tile position.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HexPosition {
    pub q: i32,
    pub r: i32,
}

impl HexPosition {
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// Convert to `hexx::Hex` for math operations.
    pub fn to_hex(self) -> Hex {
        Hex::new(self.q, self.r)
    }

    /// Convert from `hexx::Hex`.
    pub fn from_hex(hex: Hex) -> Self {
        Self {
            q: hex.x(),
            r: hex.y(),
        }
    }
}

/// Global grid configuration.
#[derive(Resource, Debug)]
pub struct HexGridConfig {
    /// Hex layout (pointy-top or flat-top). We use pointy-top.
    pub layout: hexx::HexLayout,
    /// Radius of the map in hex tiles from center.
    pub map_radius: u32,
}

/// Fired when an entity moves to a new hex position.
#[derive(Event, Debug)]
pub struct HexMoveEvent {
    pub entity: Entity,
    pub from: HexPosition,
    pub to: HexPosition,
}

/// Fired when a hex tile is selected (clicked/tapped).
#[derive(Event, Debug)]
pub struct HexSelectedEvent {
    pub position: HexPosition,
}

/// Marker component for hex tile entities spawned by the grid.
#[derive(Component, Debug)]
pub struct HexTile;

/// Tracks the currently selected hex tile, if any.
#[derive(Resource, Debug, Default)]
pub struct SelectedHex {
    pub position: Option<HexPosition>,
}

/// Stores the "base" material for a hex tile — the cell type color
/// that should be shown when the tile is not hovered or selected.
/// Updated by the cell plugin when cell data changes.
#[derive(Component, Debug, Clone)]
pub struct TileBaseMaterial(pub Handle<StandardMaterial>);
