# Hex-Edge Contract — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan
> task-by-task.

**Goal:** Extend the `hex_grid` contract with spatial infrastructure for hex-edge identity and
user-defined edge annotations, following the contract protocol (spec first, then code).

**Architecture:** Three new types added to `src/contracts/hex_grid.rs`: `HexEdge` (canonical edge
identity using HexPosition + direction), `EdgeFeature` (annotation referencing entity types by name
via `EntityTypeRegistry`, same pattern as `BiomeEntry.terrain_name`), and `HexEdgeRegistry`
(resource-based storage mapping edges to features). No new `EntityRole` — edge annotations store a
type name string resolved at use time.

**Tech Stack:** Rust, Bevy 0.18, hexx 0.22.0 (`EdgeDirection` for direction math)

**Design doc:** `docs/plans/2026-02-21-map-gen-design.md`

---

### Task 1: Update hex-grid contract spec

**Files:**

- Modify: `docs/contracts/hex-grid.md`

**Step 1: Add Hex Edges section to the contract spec**

Add a new section after "Line of Sight & Visibility (0.7.0)" in `docs/contracts/hex-grid.md`:

````markdown
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
    /// Maps to hexx::EdgeDirection.
    pub direction: u8,
}

/// An annotation on a hex edge. References a user-defined type by name,
/// resolved against `EntityTypeRegistry` at use time. Same pattern as
/// `BiomeEntry.terrain_name` for hex centers.
#[derive(Debug, Clone)]
pub struct EdgeFeature {
    /// Name of the entity type this edge annotation represents.
    /// Resolved against `EntityTypeRegistry` by name at use time.
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
````

````

**Step 2: Add invariants for hex edges**

Add to the Invariants section:

```markdown
- `HexEdge` is always in canonical form: origin is the lower hex (by q, then r)
- `HexEdge.direction` is always in range 0..6
- `HexEdgeRegistry` is inserted as a resource during `Startup` by the hex_grid plugin
````

**Step 3: Add changelog entry**

Add to the Changelog table:

```markdown
| 2026-02-22 | Added HexEdge, EdgeFeature, HexEdgeRegistry | 0.12.0 — hex edge spatial
infrastructure for user-defined annotations |
```

**Step 4: Run prettier on the spec**

Run: `npx prettier --write docs/contracts/hex-grid.md`

**Step 5: Commit**

```bash
git add docs/contracts/hex-grid.md
git commit -m "docs(contracts): add hex-edge types to hex-grid spec"
```

---

### Task 2: Implement hex-edge contract types with tests

**Files:**

- Modify: `src/contracts/hex_grid.rs`

**Step 1: Write failing tests for HexEdge canonical form**

Add a `#[cfg(test)] mod tests` section at the end of `src/contracts/hex_grid.rs` with these tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_edge_canonical_form_lower_origin() {
        // Edge from (0,0) to (1,0) direction 0 — origin is already lower
        let edge = HexEdge::new(HexPosition::new(0, 0), 0);
        assert_eq!(edge.origin, HexPosition::new(0, 0));
    }

    #[test]
    fn hex_edge_canonical_form_swaps_when_needed() {
        // Edge from (1,0) toward (0,0) — should canonicalize to origin (0,0)
        let a = HexPosition::new(1, 0);
        let b = HexPosition::new(0, 0);
        let edge = HexEdge::between(a, b);
        assert!(edge.is_some());
        let edge = edge.unwrap();
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
    fn hex_edge_direction_in_range() {
        let edge = HexEdge::new(HexPosition::new(0, 0), 0);
        assert!(edge.direction < 6);
    }

    #[test]
    fn hex_edge_new_clamps_direction() {
        // Direction 7 should wrap to valid range
        let edge = HexEdge::new(HexPosition::new(0, 0), 7);
        assert!(edge.direction < 6);
    }

    #[test]
    fn hex_edge_neighbor_pair() {
        // Verify that the edge connects origin to its neighbor in the given direction
        let edge = HexEdge::new(HexPosition::new(0, 0), 0);
        let (a, b) = edge.neighbor_pair();
        assert_ne!(a, b);
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test hex_grid`

Expected: Compilation errors — `HexEdge` doesn't exist yet.

**Step 3: Implement HexEdge**

Add to `src/contracts/hex_grid.rs` after the VisibilityRange section:

```rust
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Hex Edges (0.12.0)
// ---------------------------------------------------------------------------

/// A canonical representation of a hex edge — the shared boundary between
/// two adjacent hex tiles. Stored in canonical form: the "lower" hex
/// (ordered by q, then r) is always the origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub struct HexEdge {
    /// The canonical origin hex (lower of the two adjacent hexes).
    pub origin: HexPosition,
    /// Direction index (0-5) from origin to the adjacent hex.
    pub direction: u8,
}

impl HexEdge {
    /// Create a new edge from an origin hex and direction (0-5).
    /// Direction is taken modulo 6. The edge is NOT automatically canonicalized —
    /// use `between()` when constructing from two arbitrary hexes.
    pub fn new(origin: HexPosition, direction: u8) -> Self {
        Self {
            origin,
            direction: direction % 6,
        }
    }

    /// Create a canonical edge between two adjacent hex positions.
    /// Returns `None` if the positions are not adjacent.
    /// The "lower" hex (by q, then r) becomes the origin.
    pub fn between(a: HexPosition, b: HexPosition) -> Option<Self> {
        let hex_a = a.to_hex();
        let hex_b = b.to_hex();
        let dir = hex_a.neighbor_direction(hex_b)?;
        let dir_index = dir.index() as u8;

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
                direction: reverse_dir.index() as u8,
            })
        }
    }

    /// Returns the two hex positions connected by this edge.
    pub fn neighbor_pair(&self) -> (HexPosition, HexPosition) {
        let origin_hex = self.origin.to_hex();
        let all_dirs = hexx::EdgeDirection::ALL_DIRECTIONS;
        let dir = all_dirs[self.direction as usize];
        let neighbor = origin_hex.neighbor(dir);
        (self.origin, HexPosition::from_hex(neighbor))
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test hex_grid`

Expected: All HexEdge tests pass.

**Step 5: Write failing tests for EdgeFeature and HexEdgeRegistry**

Add to the test module:

```rust
    #[test]
    fn edge_registry_insert_and_lookup() {
        let mut registry = HexEdgeRegistry::default();
        let edge = HexEdge::new(HexPosition::new(0, 0), 0);
        let feature = EdgeFeature {
            type_name: "River".to_string(),
        };
        registry.insert(edge, feature);
        assert!(registry.get(&edge).is_some());
        assert_eq!(registry.get(&edge).unwrap().type_name, "River");
    }

    #[test]
    fn edge_registry_remove() {
        let mut registry = HexEdgeRegistry::default();
        let edge = HexEdge::new(HexPosition::new(0, 0), 0);
        registry.insert(edge, EdgeFeature { type_name: "Wall".to_string() });
        assert!(registry.remove(&edge).is_some());
        assert!(registry.get(&edge).is_none());
    }

    #[test]
    fn edge_registry_canonical_lookup() {
        let mut registry = HexEdgeRegistry::default();
        let a = HexPosition::new(0, 0);
        let b = HexPosition::new(1, 0);

        // Insert via between (canonical)
        let edge = HexEdge::between(a, b).unwrap();
        registry.insert(edge, EdgeFeature { type_name: "Path".to_string() });

        // Look up from either side — same canonical edge
        let edge_ba = HexEdge::between(b, a).unwrap();
        assert_eq!(registry.get(&edge_ba).unwrap().type_name, "Path");
    }

    #[test]
    fn edge_registry_iter() {
        let mut registry = HexEdgeRegistry::default();
        registry.insert(
            HexEdge::new(HexPosition::new(0, 0), 0),
            EdgeFeature { type_name: "A".to_string() },
        );
        registry.insert(
            HexEdge::new(HexPosition::new(0, 0), 1),
            EdgeFeature { type_name: "B".to_string() },
        );
        assert_eq!(registry.iter().count(), 2);
    }

    #[test]
    fn edge_registry_edges_for_hex() {
        let mut registry = HexEdgeRegistry::default();
        let center = HexPosition::new(0, 0);

        // Add edges on two sides of center hex
        registry.insert(
            HexEdge::new(center, 0),
            EdgeFeature { type_name: "A".to_string() },
        );
        registry.insert(
            HexEdge::new(center, 3),
            EdgeFeature { type_name: "B".to_string() },
        );
        // Add edge not touching center
        registry.insert(
            HexEdge::new(HexPosition::new(5, 5), 0),
            EdgeFeature { type_name: "C".to_string() },
        );

        let edges: Vec<_> = registry.edges_for_hex(center).collect();
        assert_eq!(edges.len(), 2);
    }
```

**Step 6: Run tests to verify they fail**

Run: `cargo test hex_grid`

Expected: Compilation errors — `EdgeFeature`, `HexEdgeRegistry` don't exist yet.

**Step 7: Implement EdgeFeature and HexEdgeRegistry**

Add after HexEdge in `src/contracts/hex_grid.rs`:

```rust
/// An annotation on a hex edge. References a user-defined type by name,
/// resolved against `EntityTypeRegistry` at use time. Same pattern as
/// `BiomeEntry.terrain_name` for hex centers.
#[derive(Debug, Clone, Reflect)]
pub struct EdgeFeature {
    /// Name of the entity type this edge annotation represents.
    pub type_name: String,
}

/// Resource-based registry of edge annotations. Maps canonical hex edges
/// to their features.
#[derive(Resource, Debug, Clone, Default, Reflect)]
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
    pub fn edges_for_hex(&self, pos: HexPosition) -> impl Iterator<Item = (&HexEdge, &EdgeFeature)> {
        let hex = pos.to_hex();
        self.edges.iter().filter(move |(edge, _)| {
            let (a, b) = edge.neighbor_pair();
            a == pos || b == pos
        })
    }

    /// Returns true if the registry has no edge features.
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    /// Returns the number of edge features in the registry.
    pub fn len(&self) -> usize {
        self.edges.len()
    }
}
```

**Step 8: Run tests to verify they pass**

Run: `cargo test hex_grid`

Expected: All tests pass.

**Step 9: Commit**

```bash
git add src/contracts/hex_grid.rs
git commit -m "feat(contracts): add hex-edge types to hex_grid contract"
```

---

### Task 3: Run full checks and update documentation

**Files:**

- Possibly modify: any file with issues
- Modify: `docs/plugins/map-gen/spec.md`
- Modify: `docs/plugins/map-gen/log.md`

**Step 1: Run clippy**

Run: `cargo clippy --all-targets`

Expected: Zero warnings. Fix any that appear.

**Step 2: Run full test suite**

Run: `cargo test`

Expected: All tests pass (existing + new hex-edge tests).

**Step 3: Run boundary and unwrap checks**

Run: `mise check:boundary && mise check:unwrap`

Expected: No violations.

**Step 4: Update spec success criteria**

Mark SC-7, SC-8, SC-9 as complete in `docs/plugins/map-gen/spec.md`.

**Step 5: Update plugin log with test results**

Add test results to `docs/plugins/map-gen/log.md`.

**Step 6: Post scope completion comment on pitch #102**

```bash
gh issue comment 102 --body "Scope 6 complete (hex-edge contract, commit <SHA>): ..."
```

**Step 7: Commit docs**

```bash
git add docs/plugins/map-gen/spec.md docs/plugins/map-gen/log.md
git commit -m "docs(map_gen): update spec and log with scope 6 results"
```
