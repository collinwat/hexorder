# Contract: hex_grid

## Purpose

Defines the core hex coordinate types and grid configuration shared by all features that interact
with the hex map. The grid exists on the 3D ground plane (XZ).

## Consumers

- (all features that place or query entities on the hex grid)

## Producers

- hex_grid feature (owns the grid storage and coordinate math)

## Types

### Components

```rust
/// Marks an entity as occupying a hex tile position.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HexPosition {
    pub q: i32,
    pub r: i32,
}

/// Marker component for hex tile entities spawned by the grid.
#[derive(Component, Debug)]
pub struct HexTile;
```

```rust
/// Stores the "base" material for a hex tile — the cell type color
/// that should be shown when the tile is not hovered or selected.
/// Updated by the cell plugin when cell data changes.
#[derive(Component, Debug, Clone)]
pub struct TileBaseMaterial(pub Handle<StandardMaterial>);
```

### Resources

```rust
/// Global grid configuration.
#[derive(Resource, Debug)]
pub struct HexGridConfig {
    /// Hex layout (pointy-top or flat-top). We use pointy-top.
    pub layout: hexx::HexLayout,
    /// Radius of the map in hex tiles from center.
    pub map_radius: u32,
}

/// Tracks the currently selected hex tile, if any.
#[derive(Resource, Debug, Default)]
pub struct SelectedHex {
    pub position: Option<HexPosition>,
}
```

### Events

```rust
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
```

### Utility Types

```rust
/// Re-export hexx::Hex for coordinate math.
pub use hexx::Hex;

impl HexPosition {
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// Convert to hexx::Hex for math operations.
    pub fn to_hex(self) -> Hex {
        Hex::new(self.q, self.r)
    }

    /// Convert from hexx::Hex.
    pub fn from_hex(hex: Hex) -> Self {
        Self { q: hex.x(), r: hex.y() }
    }
}
```

## Invariants

- `HexPosition` coordinates are always valid axial coordinates
- `HexGridConfig` is inserted as a resource during `Startup` by the hex_grid plugin
- `SelectedHex` is inserted as a resource during `Startup` by the hex_grid plugin
- `HexTile` is attached to every hex tile entity spawned by the grid
- `HexMoveEvent` is only fired for moves that have been validated (target is in bounds)

## Changelog

| Date       | Change                           | Reason                                                                    |
| ---------- | -------------------------------- | ------------------------------------------------------------------------- |
| 2026-02-08 | Initial definition               | Foundation for all hex-based features                                     |
| 2026-02-08 | Added HexTile, SelectedHex       | Promoted from hex_grid internals to fix contract boundary violations      |
| 2026-02-10 | Added TileBaseMaterial component | Needed so hover/selection ring overlays can coexist with cell type colors |
