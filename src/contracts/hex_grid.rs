//! Shared hex grid types. See `.specs/contracts/hex_grid.md`.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Re-export `hexx::Hex` for coordinate math.
pub use hexx::Hex;

/// Marks an entity as occupying a hex tile position.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
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
#[derive(Resource, Debug, Reflect)]
pub struct HexGridConfig {
    /// Hex layout (pointy-top or flat-top). We use pointy-top.
    #[reflect(ignore)]
    pub layout: hexx::HexLayout,
    /// Radius of the map in hex tiles from center.
    pub map_radius: u32,
}

/// Fired when an entity moves to a new hex position.
#[derive(Event, Debug, Reflect)]
pub struct HexMoveEvent {
    pub entity: Entity,
    pub from: HexPosition,
    pub to: HexPosition,
}

/// Fired when a hex tile is selected (clicked/tapped).
#[derive(Event, Debug, Reflect)]
pub struct HexSelectedEvent {
    pub position: HexPosition,
}

/// Marker component for hex tile entities spawned by the grid.
#[derive(Component, Debug, Reflect)]
pub struct HexTile;

/// Tracks the currently selected hex tile, if any.
#[derive(Resource, Debug, Default, Reflect)]
pub struct SelectedHex {
    pub position: Option<HexPosition>,
}

/// Stores the "base" material for a hex tile — the cell type color
/// that should be shown when the tile is not hovered or selected.
/// Updated by the cell plugin when cell data changes.
#[derive(Component, Debug, Clone, Reflect)]
pub struct TileBaseMaterial(pub Handle<StandardMaterial>);

// ---------------------------------------------------------------------------
// Move Overlays (0.4.0)
// ---------------------------------------------------------------------------

/// The visual state of a move overlay on a hex tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum MoveOverlayState {
    /// This hex is a valid destination. Rendered with a green tint.
    Valid,
    /// This hex is within range but blocked by a constraint. Rendered red.
    Blocked,
}

/// Component on overlay entities that float above hex tiles to indicate
/// move validity. Managed by `hex_grid`: spawned when a unit is selected,
/// despawned when deselected.
#[derive(Component, Debug, Clone, Reflect)]
pub struct MoveOverlay {
    pub state: MoveOverlayState,
    pub position: HexPosition,
}
