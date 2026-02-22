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

### Move Overlays (M4)

```rust
/// The visual state of a move overlay on a hex tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveOverlayState {
    /// This hex is a valid destination. Rendered with a green tint.
    Valid,
    /// This hex is within range but blocked by a constraint. Rendered with a red tint.
    Blocked,
    /// No overlay.
    None,
}

/// Component on overlay entities that float above hex tiles to indicate move validity.
/// Managed by hex_grid: spawned when a unit is selected, despawned when deselected.
#[derive(Component, Debug, Clone)]
pub struct MoveOverlay {
    pub state: MoveOverlayState,
    pub position: HexPosition,
}
```

### Line of Sight & Visibility (0.7.0)

```rust
/// Result of a line-of-sight query between two hexes.
#[derive(Debug, Clone)]
pub struct LineOfSightResult {
    pub origin: HexPosition,
    pub target: HexPosition,
    pub clear: bool,
    pub path: Vec<HexPosition>,
    pub blocked_by: Option<HexPosition>,
}

/// Component giving a unit a visibility range (in hexes).
#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct VisibilityRange {
    pub range: u32,
}
```

### Hex Edges (0.12.0)

```rust
/// A canonical representation of a hex edge — the shared boundary between
/// two adjacent hex tiles. Stored in canonical form: the "lower" hex
/// (ordered by q, then r) is always the origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HexEdge {
    /// The canonical origin hex (lower of the two adjacent hexes).
    pub origin: HexPosition,
    /// Direction index (0-5) from origin to the adjacent hex.
    pub direction: u8,
}

/// An annotation on a hex edge. References a user-defined type by name,
/// resolved against `EntityTypeRegistry` at use time. Same pattern as
/// `BiomeEntry.terrain_name` for hex centers.
#[derive(Debug, Clone)]
pub struct EdgeFeature {
    /// Name of the entity type this edge annotation represents.
    pub type_name: String,
}

/// Resource-based registry of edge annotations. Maps canonical hex edges
/// to their features. Plugins insert/query/remove edge features through
/// this registry.
#[derive(Resource, Debug, Clone, Default)]
pub struct HexEdgeRegistry {
    pub edges: HashMap<HexEdge, EdgeFeature>,
}
```

## Invariants

- `HexPosition` coordinates are always valid axial coordinates
- `HexGridConfig` is inserted as a resource during `Startup` by the hex_grid plugin
- `SelectedHex` is inserted as a resource during `Startup` by the hex_grid plugin
- `HexTile` is attached to every hex tile entity spawned by the grid
- `HexMoveEvent` is only fired for moves that have been validated (target is in bounds)
- `HexEdge` is always in canonical form: origin is the lower hex (by q, then r)
- `HexEdge.direction` is always in range 0..6
- `HexEdgeRegistry` is inserted as a resource during `Startup` by the hex_grid plugin

## Changelog

| Date       | Change                                      | Reason                                                                    |
| ---------- | ------------------------------------------- | ------------------------------------------------------------------------- |
| 2026-02-08 | Initial definition                          | Foundation for all hex-based features                                     |
| 2026-02-08 | Added HexTile, SelectedHex                  | Promoted from hex_grid internals to fix contract boundary violations      |
| 2026-02-10 | Added TileBaseMaterial component            | Needed so hover/selection ring overlays can coexist with cell type colors |
| 2026-02-11 | Added MoveOverlay, MoveOverlayState         | M4 — visual feedback for valid/blocked move destinations                  |
| 2026-02-15 | Added LineOfSightResult, VisibilityRange    | 0.7.0 — hex grid foundation: LOS algorithm and visibility                 |
| 2026-02-22 | Added HexEdge, EdgeFeature, HexEdgeRegistry | 0.12.0 — hex edge spatial infrastructure for user-defined annotations     |
