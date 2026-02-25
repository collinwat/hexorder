//! Shared hex grid types. See `docs/contracts/hex-grid.md`.

use std::collections::HashMap;

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
    #[must_use]
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// Convert to `hexx::Hex` for math operations.
    #[must_use]
    pub fn to_hex(self) -> Hex {
        Hex::new(self.q, self.r)
    }

    /// Convert from `hexx::Hex`.
    #[must_use]
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

// ---------------------------------------------------------------------------
// Line of Sight & Visibility (0.7.0)
// ---------------------------------------------------------------------------

/// Result of a line-of-sight query between two hexes.
#[derive(Debug, Clone)]
pub struct LineOfSightResult {
    /// Origin hex of the LOS query.
    pub origin: HexPosition,
    /// Target hex of the LOS query.
    pub target: HexPosition,
    /// Whether the line of sight is clear (no blocking hexes).
    pub clear: bool,
    /// All hexes along the line from origin to target.
    pub path: Vec<HexPosition>,
    /// The first hex that blocks the line of sight, if any.
    pub blocked_by: Option<HexPosition>,
}

/// Component giving a unit a visibility range (in hexes).
/// Used by field-of-view queries and future fog of war.
#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct VisibilityRange {
    pub range: u32,
}

// ---------------------------------------------------------------------------
// Hex Edges (0.12.0)
// ---------------------------------------------------------------------------

/// A canonical representation of a hex edge — the shared boundary between
/// two adjacent hex tiles. Stored in canonical form: the "lower" hex
/// (ordered by q, then r) is always the origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub struct HexEdge {
    /// The canonical origin hex (lower of the two adjacent hexes).
    pub origin: HexPosition,
    /// Direction index (0-5) from origin to the adjacent hex.
    pub direction: u8,
}

impl HexEdge {
    /// Create a new edge from an origin hex and direction (0-5).
    /// Direction is taken modulo 6. The result is always in canonical
    /// form: the "lower" hex (by q, then r) becomes the origin.
    pub fn new(origin: HexPosition, direction: u8) -> Self {
        let dir = direction % 6;
        let origin_hex = origin.to_hex();
        let all_dirs = hexx::EdgeDirection::ALL_DIRECTIONS;
        let edge_dir = all_dirs[dir as usize];
        let neighbor = origin_hex.neighbor(edge_dir);
        let neighbor_pos = HexPosition::from_hex(neighbor);

        if (origin.q, origin.r) <= (neighbor_pos.q, neighbor_pos.r) {
            Self {
                origin,
                direction: dir,
            }
        } else {
            // Swap: use neighbor as origin with the reverse direction
            let reverse_dir = neighbor
                .neighbor_direction(origin_hex)
                .map_or(0, hexx::EdgeDirection::index);
            Self {
                origin: neighbor_pos,
                direction: reverse_dir,
            }
        }
    }

    /// Create a canonical edge between two adjacent hex positions.
    /// Returns `None` if the positions are not adjacent.
    /// The "lower" hex (by q, then r) becomes the origin.
    #[must_use]
    pub fn between(a: HexPosition, b: HexPosition) -> Option<Self> {
        let hex_a = a.to_hex();
        let hex_b = b.to_hex();
        let dir = hex_a.neighbor_direction(hex_b)?;
        let dir_index = dir.index();

        // Canonicalize: lower hex is origin
        if (a.q, a.r) <= (b.q, b.r) {
            Some(Self {
                origin: a,
                direction: dir_index,
            })
        } else {
            let reverse_dir = hex_b.neighbor_direction(hex_a)?;
            Some(Self {
                origin: b,
                direction: reverse_dir.index(),
            })
        }
    }

    /// Returns the two hex positions connected by this edge.
    #[must_use]
    pub fn neighbor_pair(&self) -> (HexPosition, HexPosition) {
        let origin_hex = self.origin.to_hex();
        let all_dirs = hexx::EdgeDirection::ALL_DIRECTIONS;
        let dir = all_dirs[self.direction as usize];
        let neighbor = origin_hex.neighbor(dir);
        (self.origin, HexPosition::from_hex(neighbor))
    }
}

/// An annotation on a hex edge. References a user-defined type by name,
/// resolved against `EntityTypeRegistry` at use time.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct EdgeFeature {
    /// Name of the entity type this edge annotation represents.
    pub type_name: String,
}

/// Resource-based registry of edge annotations.
#[derive(Resource, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
#[reflect(opaque)]
pub struct HexEdgeRegistry {
    pub edges: HashMap<HexEdge, EdgeFeature>,
}

impl HexEdgeRegistry {
    /// Insert or replace an edge feature.
    pub fn insert(&mut self, edge: HexEdge, feature: EdgeFeature) {
        self.edges.insert(edge, feature);
    }

    /// Look up the feature on an edge.
    #[must_use]
    pub fn get(&self, edge: &HexEdge) -> Option<&EdgeFeature> {
        self.edges.get(edge)
    }

    /// Remove an edge feature. Returns the removed feature.
    pub fn remove(&mut self, edge: &HexEdge) -> Option<EdgeFeature> {
        self.edges.remove(edge)
    }

    /// Iterate over all edge features.
    pub fn iter(&self) -> impl Iterator<Item = (&HexEdge, &EdgeFeature)> {
        self.edges.iter()
    }

    /// Iterate over all edges touching a specific hex position.
    pub fn edges_for_hex(
        &self,
        pos: HexPosition,
    ) -> impl Iterator<Item = (&HexEdge, &EdgeFeature)> {
        self.edges.iter().filter(move |(edge, _)| {
            let (a, b) = edge.neighbor_pair();
            a == pos || b == pos
        })
    }

    /// Returns true if the registry has no edge features.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    /// Returns the number of edge features in the registry.
    #[must_use]
    pub fn len(&self) -> usize {
        self.edges.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_edge_canonical_form_lower_origin() {
        let edge = HexEdge::new(HexPosition::new(0, 0), 0);
        assert_eq!(edge.origin, HexPosition::new(0, 0));
        assert_eq!(edge.direction, 0);
    }

    #[test]
    fn hex_edge_canonical_form_swaps_when_needed() {
        let a = HexPosition::new(1, 0);
        let b = HexPosition::new(0, 0);
        let edge = HexEdge::between(a, b).expect("adjacent hexes should produce an edge");
        // (0,0) < (1,0) so origin should be (0,0)
        assert_eq!(edge.origin, HexPosition::new(0, 0));
    }

    #[test]
    fn hex_edge_between_non_adjacent_returns_none() {
        let a = HexPosition::new(0, 0);
        let b = HexPosition::new(5, 5);
        assert!(HexEdge::between(a, b).is_none());
    }

    #[test]
    fn hex_edge_same_edge_from_both_sides_equal() {
        let a = HexPosition::new(0, 0);
        let b = HexPosition::new(1, 0);
        let edge_ab = HexEdge::between(a, b);
        let edge_ba = HexEdge::between(b, a);
        assert_eq!(edge_ab, edge_ba);
    }

    #[test]
    fn hex_edge_direction_wraps() {
        let edge = HexEdge::new(HexPosition::new(0, 0), 7);
        assert_eq!(edge.direction, 1); // 7 % 6 = 1
    }

    #[test]
    fn hex_edge_neighbor_pair_returns_both_hexes() {
        let edge = HexEdge::new(HexPosition::new(0, 0), 0);
        let (a, b) = edge.neighbor_pair();
        assert_eq!(a, HexPosition::new(0, 0));
        assert_ne!(a, b);
    }

    #[test]
    fn hex_edge_all_six_directions_produce_unique_edges() {
        let origin = HexPosition::new(0, 0);
        let edges: Vec<HexEdge> = (0..6).map(|d| HexEdge::new(origin, d)).collect();
        for (i, a) in edges.iter().enumerate() {
            for (j, b) in edges.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b);
                }
            }
        }
    }

    #[test]
    fn edge_registry_insert_and_lookup() {
        let mut registry = HexEdgeRegistry::default();
        let edge = HexEdge::new(HexPosition::new(0, 0), 0);
        let feature = EdgeFeature {
            type_name: "River".to_string(),
        };
        registry.insert(edge, feature);
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
        assert_eq!(
            registry
                .get(&edge)
                .expect("inserted edge should be present")
                .type_name,
            "River"
        );
    }

    #[test]
    fn edge_registry_remove() {
        let mut registry = HexEdgeRegistry::default();
        let edge = HexEdge::new(HexPosition::new(0, 0), 0);
        registry.insert(
            edge,
            EdgeFeature {
                type_name: "Wall".to_string(),
            },
        );
        assert!(registry.remove(&edge).is_some());
        assert!(registry.get(&edge).is_none());
        assert!(registry.is_empty());
    }

    #[test]
    fn edge_registry_canonical_lookup() {
        let mut registry = HexEdgeRegistry::default();
        let a = HexPosition::new(0, 0);
        let b = HexPosition::new(1, 0);
        let edge = HexEdge::between(a, b).expect("adjacent hexes should produce an edge");
        registry.insert(
            edge,
            EdgeFeature {
                type_name: "Path".to_string(),
            },
        );
        // Look up from the other side — same canonical edge
        let edge_ba = HexEdge::between(b, a).expect("adjacent hexes should produce an edge");
        assert_eq!(
            registry
                .get(&edge_ba)
                .expect("canonical lookup should find edge")
                .type_name,
            "Path"
        );
    }

    #[test]
    fn edge_registry_iter() {
        let mut registry = HexEdgeRegistry::default();
        registry.insert(
            HexEdge::new(HexPosition::new(0, 0), 0),
            EdgeFeature {
                type_name: "A".to_string(),
            },
        );
        registry.insert(
            HexEdge::new(HexPosition::new(0, 0), 1),
            EdgeFeature {
                type_name: "B".to_string(),
            },
        );
        assert_eq!(registry.iter().count(), 2);
    }

    #[test]
    fn hex_edge_new_produces_canonical_form() {
        // HexEdge::new from the "higher" hex should canonicalize to match between()
        let a = HexPosition::new(0, 0);
        let b = HexPosition::new(1, 0);
        let edge_between = HexEdge::between(a, b).expect("adjacent hexes should produce an edge");
        // Find which direction from b points to a
        let hex_b = b.to_hex();
        let hex_a = a.to_hex();
        let dir_ba = hex_b
            .neighbor_direction(hex_a)
            .expect("adjacent hexes should have a direction");
        let edge_new = HexEdge::new(b, dir_ba.index());
        assert_eq!(edge_new, edge_between);
    }

    #[test]
    fn edge_feature_type_name_resolves_against_entity_registry() {
        use crate::game_system::{EntityRole, EntityType, EntityTypeRegistry, TypeId};

        let mut registry = EntityTypeRegistry::default();
        registry.types.push(EntityType {
            id: TypeId::new(),
            name: "Wall".to_string(),
            role: EntityRole::BoardPosition,
            color: bevy::color::Color::srgb(0.5, 0.5, 0.5),
            properties: vec![],
        });

        let feature = EdgeFeature {
            type_name: "Wall".to_string(),
        };

        // Resolve: look up feature type_name in entity registry
        let resolved = registry.types.iter().find(|t| t.name == feature.type_name);
        let resolved = resolved.expect("Wall type should resolve in entity registry");
        assert_eq!(resolved.name, "Wall");
    }

    #[test]
    fn edge_registry_edges_for_hex() {
        let mut registry = HexEdgeRegistry::default();
        let center = HexPosition::new(0, 0);
        registry.insert(
            HexEdge::new(center, 0),
            EdgeFeature {
                type_name: "A".to_string(),
            },
        );
        registry.insert(
            HexEdge::new(center, 3),
            EdgeFeature {
                type_name: "B".to_string(),
            },
        );
        // Edge not touching center
        registry.insert(
            HexEdge::new(HexPosition::new(5, 5), 0),
            EdgeFeature {
                type_name: "C".to_string(),
            },
        );
        let edges: Vec<_> = registry.edges_for_hex(center).collect();
        assert_eq!(edges.len(), 2);
    }
}
