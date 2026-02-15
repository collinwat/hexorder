# Hex Grid Foundation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan
> task-by-task.

**Goal:** Deepen hexx crate integration by exposing pathfinding, field-of-view, line-drawing, and
spatial query algorithms through `HexPosition`-native wrappers, then add a visual LOS ray system.

**Architecture:** A new private `algorithms.rs` module in `hex_grid` wraps hexx's algorithms behind
`HexPosition`. A gizmo-based `draw_los_ray` system renders LOS feedback when a unit is selected and
hovering a target hex. Contract types `LineOfSightResult` and `VisibilityRange` are added to the
hex_grid contract.

**Tech Stack:** Rust, Bevy 0.18, hexx 0.22 (with `algorithms` feature), Bevy gizmos

---

## Task 1: Enable hexx algorithms feature

**Files:**

- Modify: `Cargo.toml:14`

**Step 1: Update Cargo.toml**

Change hexx dependency from:

```toml
hexx = "0.22"
```

to:

```toml
hexx = { version = "0.22", features = ["algorithms"] }
```

**Step 2: Verify it compiles**

Run: `cargo build` Expected: success (no warnings, no errors)

**Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore(hex_grid): enable hexx algorithms feature

Needed for a_star, range_fov, and line-drawing APIs. Part of 0.7.0."
```

---

## Task 2: Add contract types (LineOfSightResult, VisibilityRange)

**Files:**

- Modify: `src/contracts/hex_grid.rs:96` (append after MoveOverlay)
- Modify: `docs/contracts/hex-grid.md` (append new section)

**Step 1: Add Rust types to contract**

Append to `src/contracts/hex_grid.rs` after the MoveOverlay block:

```rust
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
```

**Step 2: Update contract spec doc**

Append to `docs/contracts/hex-grid.md` before the Invariants section a new subsection:

```markdown
### Line of Sight & Visibility (0.7.0)

\`\`\`rust /// Result of a line-of-sight query between two hexes. #[derive(Debug, Clone)] pub struct
LineOfSightResult { pub origin: HexPosition, pub target: HexPosition, pub clear: bool, pub path:
Vec<HexPosition>, pub blocked_by: Option<HexPosition>, }

/// Component giving a unit a visibility range (in hexes). #[derive(Component, Debug, Clone, Copy,
Reflect)] pub struct VisibilityRange { pub range: u32, } \`\`\`
```

Also add a changelog entry to the table at the bottom:

```
| 2026-02-15 | Added LineOfSightResult, VisibilityRange | 0.7.0 — hex grid foundation: LOS algorithm and visibility |
```

**Step 3: Verify it compiles**

Run: `cargo build` Expected: success

**Step 4: Commit**

```bash
git add src/contracts/hex_grid.rs docs/contracts/hex-grid.md
git commit -m "feat(contracts): add LineOfSightResult and VisibilityRange types

Foundation for LOS queries and future fog of war. Part of 0.7.0."
```

---

## Task 3: Create algorithms module — neighbors and spatial queries

**Files:**

- Create: `src/hex_grid/algorithms.rs`
- Modify: `src/hex_grid/mod.rs:12` (add `mod algorithms;`)

**Step 1: Write failing tests in tests.rs**

Append to `src/hex_grid/tests.rs`:

```rust
// ---------------------------------------------------------------------------
// Algorithm tests (0.7.0)
// ---------------------------------------------------------------------------

use super::algorithms;

#[test]
fn neighbors_returns_six() {
    let center = HexPosition::new(0, 0);
    let result = algorithms::neighbors(center);
    assert_eq!(result.len(), 6, "Should have exactly 6 neighbors");
}

#[test]
fn neighbors_are_adjacent() {
    let center = HexPosition::new(3, -2);
    let result = algorithms::neighbors(center);
    let center_hex = center.to_hex();
    for neighbor in &result {
        let dist = center_hex.unsigned_distance_to(neighbor.to_hex());
        assert_eq!(dist, 1, "Each neighbor should be distance 1 from center");
    }
}

#[test]
fn ring_at_radius() {
    let center = HexPosition::new(0, 0);
    let result = algorithms::ring(center, 2);
    assert_eq!(result.len(), 12, "Ring at radius 2 should have 12 hexes");
    let center_hex = center.to_hex();
    for pos in &result {
        let dist = center_hex.unsigned_distance_to(pos.to_hex());
        assert_eq!(dist, 2, "All ring hexes should be at exact distance 2");
    }
}

#[test]
fn ring_at_radius_zero() {
    let center = HexPosition::new(1, 1);
    let result = algorithms::ring(center, 0);
    assert_eq!(result.len(), 1, "Ring at radius 0 is just the center");
    assert_eq!(result[0], center);
}

#[test]
fn hex_range_count() {
    let center = HexPosition::new(0, 0);
    let result = algorithms::hex_range(center, 3);
    // 3*3*(3+1)+1 = 37
    let expected = 3 * 3 * (3 + 1) + 1;
    assert_eq!(
        result.len(),
        expected as usize,
        "Range at radius 3 should have {expected} hexes"
    );
    let center_hex = center.to_hex();
    for pos in &result {
        let dist = center_hex.unsigned_distance_to(pos.to_hex());
        assert!(dist <= 3, "All range hexes should be within distance 3");
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib hex_grid::tests::neighbors_returns_six` Expected: FAIL — `algorithms` module
does not exist

**Step 3: Create algorithms.rs with spatial query functions**

Create `src/hex_grid/algorithms.rs`:

```rust
//! Pure hex algorithms wrapping the `hexx` crate behind `HexPosition`.
//!
//! All functions take and return `HexPosition` so callers never interact
//! with `hexx::Hex` directly. No ECS dependencies — these are testable
//! without a Bevy app.

use std::collections::HashSet;

use crate::contracts::hex_grid::{HexPosition, LineOfSightResult};

/// Returns the 6 adjacent hex positions around `pos`.
pub fn neighbors(pos: HexPosition) -> [HexPosition; 6] {
    pos.to_hex()
        .all_neighbors()
        .map(HexPosition::from_hex)
}

/// Returns all hex positions at exactly `radius` distance from `center`.
///
/// Radius 0 returns a single-element vec containing `center`.
pub fn ring(center: HexPosition, radius: u32) -> Vec<HexPosition> {
    center
        .to_hex()
        .ring(radius)
        .map(HexPosition::from_hex)
        .collect()
}

/// Returns all hex positions within `radius` distance from `center` (inclusive).
pub fn hex_range(center: HexPosition, radius: u32) -> Vec<HexPosition> {
    center
        .to_hex()
        .range(radius)
        .map(HexPosition::from_hex)
        .collect()
}
```

**Step 4: Add `mod algorithms;` to mod.rs**

In `src/hex_grid/mod.rs`, add after line 12 (`mod systems;`):

```rust
mod algorithms;
```

**Step 5: Run tests to verify they pass**

Run: `cargo test --lib hex_grid::tests::neighbors -- --nocapture` Expected: all 5 new tests PASS

Run: `cargo clippy --all-targets` Expected: no warnings

**Step 6: Commit**

```bash
git add src/hex_grid/algorithms.rs src/hex_grid/mod.rs src/hex_grid/tests.rs
git commit -m "feat(hex_grid): add neighbors, ring, and hex_range algorithm wrappers

Pure functions wrapping hexx spatial queries behind HexPosition.
Part of 0.7.0."
```

---

## Task 4: Add line-of-sight algorithm

**Files:**

- Modify: `src/hex_grid/algorithms.rs` (add `line_of_sight` function)
- Modify: `src/hex_grid/tests.rs` (add LOS tests)

**Step 1: Write failing tests**

Append to `src/hex_grid/tests.rs`:

```rust
#[test]
fn line_of_sight_clear_path() {
    let from = HexPosition::new(0, 0);
    let to = HexPosition::new(3, 0);
    let result = algorithms::line_of_sight(from, to, |_| false);
    assert!(result.clear, "Path with no blockers should be clear");
    assert!(result.blocked_by.is_none());
    assert_eq!(result.origin, from);
    assert_eq!(result.target, to);
    assert!(result.path.len() >= 2, "Path should include at least origin and target");
    assert_eq!(*result.path.first().expect("path is non-empty"), from);
    assert_eq!(*result.path.last().expect("path is non-empty"), to);
}

#[test]
fn line_of_sight_blocked() {
    let from = HexPosition::new(0, 0);
    let to = HexPosition::new(3, 0);
    let blocker = HexPosition::new(2, 0);
    let result = algorithms::line_of_sight(from, to, |pos| pos == blocker);
    assert!(!result.clear, "Path should be blocked");
    assert_eq!(result.blocked_by, Some(blocker));
}

#[test]
fn line_of_sight_same_hex() {
    let pos = HexPosition::new(2, -1);
    let result = algorithms::line_of_sight(pos, pos, |_| false);
    assert!(result.clear);
    assert_eq!(result.path.len(), 1);
    assert_eq!(result.path[0], pos);
}

#[test]
fn line_of_sight_adjacent() {
    let from = HexPosition::new(0, 0);
    let to = HexPosition::new(1, 0);
    let result = algorithms::line_of_sight(from, to, |_| false);
    assert!(result.clear);
    assert_eq!(result.path.len(), 2);
    assert_eq!(result.path[0], from);
    assert_eq!(result.path[1], to);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib hex_grid::tests::line_of_sight_clear_path` Expected: FAIL — function does not
exist

**Step 3: Implement line_of_sight**

Add to `src/hex_grid/algorithms.rs`:

```rust
/// Computes line of sight between two hex positions.
///
/// Walks `hexx::Hex::line_to` from `from` to `to`. For each intermediate hex
/// (excluding `from`), calls `is_blocking`. If any hex blocks, the result is
/// not clear and `blocked_by` records the first blocker.
pub fn line_of_sight(
    from: HexPosition,
    to: HexPosition,
    is_blocking: impl Fn(HexPosition) -> bool,
) -> LineOfSightResult {
    let path: Vec<HexPosition> = from
        .to_hex()
        .line_to(to.to_hex())
        .map(HexPosition::from_hex)
        .collect();

    // Check each hex along the path (skip the origin — you can always see from
    // where you stand).
    let mut blocked_by = None;
    for &pos in path.iter().skip(1) {
        if is_blocking(pos) {
            blocked_by = Some(pos);
            break;
        }
    }

    LineOfSightResult {
        origin: from,
        target: to,
        clear: blocked_by.is_none(),
        path,
        blocked_by,
    }
}
```

**Step 4: Run tests**

Run: `cargo test --lib hex_grid::tests::line_of_sight` Expected: all 4 LOS tests PASS

Run: `cargo clippy --all-targets` Expected: no warnings

**Step 5: Commit**

```bash
git add src/hex_grid/algorithms.rs src/hex_grid/tests.rs
git commit -m "feat(hex_grid): add line_of_sight algorithm

Walks hexx line_to with per-hex blocking predicate. Caller controls
what blocks (placeholder: nothing blocks until property system ships).
Part of 0.7.0."
```

---

## Task 5: Add field-of-view and pathfinding algorithms

**Files:**

- Modify: `src/hex_grid/algorithms.rs` (add `field_of_view`, `find_path`)
- Modify: `src/hex_grid/tests.rs` (add field-of-view and pathfinding tests)

**Step 1: Write failing tests**

Append to `src/hex_grid/tests.rs`:

```rust
#[test]
fn field_of_view_no_blockers() {
    let origin = HexPosition::new(0, 0);
    let visible = algorithms::field_of_view(origin, 2, |_| false);
    let expected = algorithms::hex_range(origin, 2);
    assert_eq!(
        visible.len(),
        expected.len(),
        "With no blockers, visible set should equal full range"
    );
    for pos in &expected {
        assert!(visible.contains(pos), "Visible set should contain {pos:?}");
    }
}

#[test]
fn field_of_view_with_blocker() {
    let origin = HexPosition::new(0, 0);
    // Block hex (1,0) — hexes behind it in that direction should be hidden.
    let blocker = HexPosition::new(1, 0);
    let visible = algorithms::field_of_view(origin, 3, |pos| pos == blocker);
    // The blocker itself should be visible (you can see the wall).
    assert!(visible.contains(&blocker), "Blocker itself should be visible");
    // But (2,0) directly behind the blocker should be hidden.
    let behind = HexPosition::new(2, 0);
    assert!(
        !visible.contains(&behind),
        "Hex directly behind blocker should be hidden"
    );
}

#[test]
fn field_of_view_range_zero() {
    let origin = HexPosition::new(5, -3);
    let visible = algorithms::field_of_view(origin, 0, |_| false);
    assert_eq!(visible.len(), 1, "Range 0 should return only origin");
    assert!(visible.contains(&origin));
}

#[test]
fn find_path_straight_line() {
    let from = HexPosition::new(0, 0);
    let to = HexPosition::new(3, 0);
    let path = algorithms::find_path(from, to, |_, _| Some(1));
    assert!(path.is_some(), "Unobstructed path should exist");
    let path = path.expect("already checked");
    assert_eq!(*path.first().expect("path is non-empty"), from);
    assert_eq!(*path.last().expect("path is non-empty"), to);
}

#[test]
fn find_path_around_obstacle() {
    let from = HexPosition::new(0, 0);
    let to = HexPosition::new(2, 0);
    let wall = HexPosition::new(1, 0);
    let path = algorithms::find_path(from, to, |_, next| {
        if next == wall { None } else { Some(1) }
    });
    assert!(path.is_some(), "Path around obstacle should exist");
    let path = path.expect("already checked");
    assert!(!path.contains(&wall), "Path should not go through wall");
    assert_eq!(*path.first().expect("path is non-empty"), from);
    assert_eq!(*path.last().expect("path is non-empty"), to);
}

#[test]
fn find_path_no_route() {
    let from = HexPosition::new(0, 0);
    let to = HexPosition::new(3, 0);
    // Block all neighbors of origin — no way out.
    let blocked: std::collections::HashSet<HexPosition> =
        algorithms::neighbors(from).into_iter().collect();
    let path = algorithms::find_path(from, to, |_, next| {
        if blocked.contains(&next) { None } else { Some(1) }
    });
    assert!(path.is_none(), "Walled-off path should return None");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib hex_grid::tests::field_of_view_no_blockers` Expected: FAIL — function does
not exist

**Step 3: Implement field_of_view and find_path**

Add to `src/hex_grid/algorithms.rs`:

```rust
/// Computes field of view from `origin` within `range` hexes.
///
/// Wraps `hexx::algorithms::range_fov`. Returns all visible hexes (including
/// blocking hexes themselves — you can see a wall, just not through it).
pub fn field_of_view(
    origin: HexPosition,
    range: u32,
    is_blocking: impl Fn(HexPosition) -> bool,
) -> HashSet<HexPosition> {
    hexx::algorithms::range_fov(origin.to_hex(), range, |hex| {
        is_blocking(HexPosition::from_hex(hex))
    })
    .into_iter()
    .map(HexPosition::from_hex)
    .collect()
}

/// Finds the shortest path between two hex positions using A*.
///
/// Wraps `hexx::algorithms::a_star`. The `cost` function receives the current
/// hex and the candidate next hex. Return `Some(cost)` for traversable hexes
/// or `None` for impassable ones.
pub fn find_path(
    from: HexPosition,
    to: HexPosition,
    cost: impl Fn(HexPosition, HexPosition) -> Option<u32>,
) -> Option<Vec<HexPosition>> {
    hexx::algorithms::a_star(from.to_hex(), to.to_hex(), |current, next| {
        cost(
            HexPosition::from_hex(current),
            HexPosition::from_hex(next),
        )
    })
    .map(|path| path.into_iter().map(HexPosition::from_hex).collect())
}
```

**Step 4: Run tests**

Run: `cargo test --lib hex_grid::tests` Expected: all new field-of-view and pathfinding tests PASS
(plus all existing tests)

Run: `cargo clippy --all-targets` Expected: no warnings

**Step 5: Commit**

```bash
git add src/hex_grid/algorithms.rs src/hex_grid/tests.rs
git commit -m "feat(hex_grid): add field_of_view and find_path algorithms

Wraps hexx range_fov and a_star behind HexPosition API. Blocking
and cost predicates are caller-provided closures. Part of 0.7.0."
```

---

## Task 6: Add draw_los_ray gizmo system

**Files:**

- Modify: `src/hex_grid/systems.rs` (add `draw_los_ray` function)
- Modify: `src/hex_grid/mod.rs` (register system in Update chain)

**Step 1: Add draw_los_ray system to systems.rs**

Add import at top of `src/hex_grid/systems.rs`:

```rust
use crate::contracts::game_system::{SelectedUnit, UnitInstance};
use super::algorithms;
```

Then add the system function at the end of the file (before the `#[cfg(test)]` block):

```rust
/// Draws a LOS ray from the selected unit to the hovered hex using gizmos.
///
/// Green line if line of sight is clear, red if blocked. Only active when
/// a unit is selected and the mouse hovers a different hex.
pub fn draw_los_ray(
    selected_unit: Res<SelectedUnit>,
    hovered: Res<HoveredHex>,
    config: Res<HexGridConfig>,
    unit_positions: Query<&HexPosition, With<UnitInstance>>,
    mut gizmos: Gizmos,
) {
    let Some(unit_entity) = selected_unit.entity else {
        return;
    };
    let Ok(&unit_pos) = unit_positions.get(unit_entity) else {
        return;
    };
    let Some(hover_pos) = hovered.position else {
        return;
    };
    if unit_pos == hover_pos {
        return;
    }

    // Placeholder: nothing blocks LOS until property system (#81) ships.
    let result = algorithms::line_of_sight(unit_pos, hover_pos, |_| false);

    let color = if result.clear {
        Color::srgb(0.2, 0.9, 0.2)
    } else {
        Color::srgb(0.9, 0.2, 0.2)
    };

    for window in result.path.windows(2) {
        let a = config.layout.hex_to_world_pos(window[0].to_hex());
        let b = config.layout.hex_to_world_pos(window[1].to_hex());
        gizmos.line(
            Vec3::new(a.x, 0.03, a.y),
            Vec3::new(b.x, 0.03, b.y),
            color,
        );
    }
}
```

**Step 2: Register system in mod.rs**

In `src/hex_grid/mod.rs`, add `draw_los_ray` to the Update chain. Change:

```rust
                systems::sync_move_overlays,
            )
```

to:

```rust
                systems::sync_move_overlays,
                systems::draw_los_ray,
            )
```

**Step 3: Verify it compiles**

Run: `cargo build` Expected: success

Run: `cargo clippy --all-targets` Expected: no warnings

**Step 4: Commit**

```bash
git add src/hex_grid/systems.rs src/hex_grid/mod.rs
git commit -m "feat(hex_grid): add LOS ray gizmo visualization

Draws green/red line from selected unit to hovered hex showing
line of sight. Uses placeholder blocking (nothing blocks) until
property system ships. Part of 0.7.0."
```

---

## Task 7: Add system integration test

**Files:**

- Modify: `src/hex_grid/tests.rs`

**Step 1: Write the integration test**

Append to `src/hex_grid/tests.rs`:

```rust
// ---------------------------------------------------------------------------
// LOS system tests (0.7.0)
// ---------------------------------------------------------------------------

use crate::contracts::game_system::SelectedUnit;

#[test]
fn los_ray_not_drawn_without_unit() {
    // Verify draw_los_ray does not panic when SelectedUnit has no entity.
    let mut app = test_app();
    app.add_systems(
        Startup,
        (
            systems::setup_grid_config,
            systems::setup_materials,
            systems::spawn_grid,
        )
            .chain(),
    );
    app.insert_resource(SelectedUnit::default());
    app.add_systems(Update, systems::draw_los_ray);
    app.update(); // Startup
    app.update(); // Update — should not panic
}
```

**Step 2: Run test**

Run: `cargo test --lib hex_grid::tests::los_ray_not_drawn_without_unit` Expected: PASS (system
early-returns when no unit selected)

**Step 3: Run full test suite**

Run: `cargo test` Expected: all tests pass (existing 140 + new ~16 = ~156)

Run: `cargo clippy --all-targets` Expected: no warnings

**Step 4: Commit**

```bash
git add src/hex_grid/tests.rs
git commit -m "test(hex_grid): add LOS system integration test

Verifies draw_los_ray does not panic when no unit is selected.
Part of 0.7.0."
```

---

## Task 8: Update spec and log

**Files:**

- Modify: `docs/plugins/hex-grid/spec.md`
- Modify: `docs/plugins/hex-grid/log.md`

**Step 1: Update spec with new requirements and success criteria**

In `docs/plugins/hex-grid/spec.md`, add a new milestone section after M4:

```markdown
### M7 (new — hex algorithms and LOS)

13. [REQ-ALGORITHMS] Expose hexx pathfinding (A\*), field-of-view, line-drawing, neighbor, ring, and
    range algorithms through `HexPosition`-native pure functions in a private `algorithms` module.
14. [REQ-LOS] Given two hex positions and a blocking predicate, compute line of sight returning
    path, clear/blocked status, and first blocker (if any).
15. [REQ-LOS-VISUAL] When a unit is selected and the mouse hovers a different hex, draw a gizmo line
    from unit to hover target. Green if LOS is clear, red if blocked.
16. [REQ-VIS-RANGE] `VisibilityRange` component available for units (foundation for future fog of
    war).
```

Add success criteria:

```markdown
### M7 (new)

- [ ] [SC-14] Algorithm unit tests pass: LOS, field-of-view, pathfinding, neighbors, ring, range
- [ ] [SC-15] `draw_los_ray` system registered and does not panic without a selected unit
- [ ] [SC-16] `LineOfSightResult` and `VisibilityRange` contract types exist with required derives
```

**Step 2: Update plugin log**

Add entry to `docs/plugins/hex-grid/log.md`:

```markdown
### 2026-02-15 — Hex algorithms and LOS (0.7.0)

**Context**: Pitch #78 calls for hexx integration, LOS, and fog of war. After scope hammering, fog
of war was deferred (needs sides/factions) and elevation-based LOS was deferred (future cycle). LOS
terrain blocking uses a placeholder predicate until property system (#81) ships.

**Decision**: Add `algorithms.rs` private module with pure functions wrapping hexx behind
`HexPosition`. LOS visual uses Bevy gizmos (zero allocation, auto-cleared). No entity spawning for
the ray — gizmos are the right tool for transient hover feedback.

**Rationale**: Pure functions are testable without a Bevy app. Closure-based blocking predicates
decouple the algorithm from the data source. Gizmos avoid the complexity of managing overlay
entities for a visual that changes every frame.

**Deferred**: Fog of war (needs factions), elevation LOS (needs per-tile height), LOS terrain
blocking (needs property system #81).
```

Add to status updates table:

```
| 2026-02-15 | in-progress | 0.7.0: hexx algorithms, LOS algorithm + gizmo visual |
```

**Step 3: Commit**

```bash
git add docs/plugins/hex-grid/spec.md docs/plugins/hex-grid/log.md
git commit -m "docs(hex_grid): update spec and log for 0.7.0 algorithms and LOS

Adds M7 requirements, success criteria, and decision log for hexx
algorithm wrappers and LOS gizmo system. Part of 0.7.0."
```

---

## Task 9: Create GitHub Issues for deferred items

**Step 1: Search for existing issues**

```bash
gh issue list --search "fog of war" --state all
gh issue list --search "elevation LOS" --state all
gh issue list --search "blocks_los property" --state all
```

**Step 2: Create issues for deferred scope (only if not already covered)**

If #71 (fog of war) exists, update it with a dependency note:

```bash
gh issue comment 71 --body "Deferred from 0.7.0 scope: fog of war requires a sides/factions system before it can determine friendly vs enemy visibility. The hex_grid algorithms module (field_of_view, line_of_sight) is ready — fog of war needs a Side component and active-side tracking."
```

Create elevation LOS issue if none exists:

```bash
gh issue create --title "Elevation-based line of sight" --label "type:feature" --label "area:hex-grid" --body "Add integer elevation levels (0-3) per hex tile and height-interpolated LOS checks. Deferred from pitch #78 (0.7.0) to keep scope manageable. Requires: Elevation component on tiles, interpolation rules for LOS ray height at each hex."
```

Create LOS terrain blocking issue if none exists:

```bash
gh issue create --title "Wire LOS terrain blocking via property system" --label "type:feature" --label "area:hex-grid" --body "Replace the placeholder LOS blocking predicate (currently |_| false) with a property-based check once the property system (#81) ships. EntityTypes with a blocks_los boolean property should block line of sight. Blocked on: #81."
```

**Step 3: Commit**

No file changes — issues are on GitHub.

---

## Task 10: Run full quality gate

**Step 1: Run full check suite**

```bash
mise check
```

Expected: all checks pass (fmt, clippy, test, deny, typos, taplo, boundary, unwrap)

**Step 2: Verify boundary check**

```bash
mise check:boundary
```

Expected: no violations — `algorithms.rs` only imports from `crate::contracts::hex_grid`, not from
other plugins.

**Step 3: Post progress update on pitch issue**

```bash
gh issue comment 78 --body "Completed hexx algorithm integration and LOS system. Scope hammered: fog of war deferred (needs factions), elevation LOS deferred, LOS terrain blocking uses placeholder until property system (#81) ships. All algorithm tests pass. Visual LOS ray draws green/red gizmo lines from selected unit to hovered hex."
```
