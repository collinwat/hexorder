# Design: Hex Grid Foundation

**Pitch**: #78 — Hex grid foundation: hexx crate, line of sight, and fog of war **Cycle**: 2 — The
Foundation **Branch**: `0.7.0/hex-grid-foundation` **Date**: 2026-02-15

## Scope After Hammering

The original pitch called for three features: hexx integration, line of sight (LOS), and fog of war
(fog of war). After clarifying dependencies, scope has been hammered to:

**In scope:**

- Deepen hexx integration — expose pathfinding (A\*), field-of-view, line-drawing, neighbor/ring
  APIs through `HexPosition`-native wrapper functions
- Line of sight algorithm — `line_of_sight(from, to, blocking_fn)` returning path and blocked status
- LOS visual ray — gizmo-based green/red line when unit selected and hovering a target hex
- `VisibilityRange` component on units (foundation for future fog of war)
- Enable hexx `algorithms` Cargo feature

**Deferred:**

- Fog of war — requires sides/factions system that doesn't exist yet. Capture as GitHub Issue.
- Elevation-based LOS — requires per-tile elevation component and height interpolation. Capture as
  GitHub Issue.
- LOS terrain blocking — waits for property system (#81) to provide `blocks_los` property. The
  algorithm accepts a `Fn(HexPosition) -> bool` closure so blocking logic is swappable. Initial
  implementation uses `|_| false` (nothing blocks).

## Approach

**Algorithm module + visual system.** A new `algorithms.rs` submodule in `hex_grid` contains pure
functions wrapping hexx APIs behind `HexPosition`. A Bevy gizmo system handles the visual LOS ray.
This keeps algorithms testable in isolation and reusable when fog of war arrives later.

Rejected alternatives:

- All-in-systems: LOS logic tightly coupled to ECS, harder to test and reuse.
- Separate LOS plugin: over-engineered for current scope after hammering.

## Contract Changes

New types in `src/contracts/hex_grid.rs` and `docs/contracts/hex-grid.md`:

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

## Algorithm Module

**File:** `src/hex_grid/algorithms.rs` — private module, pure functions, no ECS dependencies.

| Function        | Wraps                         | Signature                                              |
| --------------- | ----------------------------- | ------------------------------------------------------ |
| `line_of_sight` | `Hex::line_to`                | `(from, to, is_blocking) -> LineOfSightResult`         |
| `field_of_view` | `hexx::algorithms::range_fov` | `(origin, range, is_blocking) -> HashSet<HexPosition>` |
| `find_path`     | `hexx::algorithms::a_star`    | `(from, to, cost) -> Option<Vec<HexPosition>>`         |
| `neighbors`     | `Hex::all_neighbors`          | `(pos) -> [HexPosition; 6]`                            |
| `ring`          | `Hex::ring`                   | `(center, radius) -> Vec<HexPosition>`                 |
| `hex_range`     | `Hex::range`                  | `(center, radius) -> Vec<HexPosition>`                 |

Design decisions:

- Functions take and return `HexPosition` — callers never touch `hexx::Hex`.
- Blocking/cost predicates are closures so the caller controls blocking logic.
- `field_of_view` returns `HashSet<HexPosition>` for efficient visibility lookups.
- No caching in the algorithm layer — caching is a system-level concern.

## Visual LOS System

**System:** `draw_los_ray` in `systems.rs`, runs in `Update` chain after `sync_move_overlays`.

Behavior:

- Only active when `SelectedUnit.entity` is `Some` and `HoveredHex.position` is `Some` and they
  differ
- Calls `line_of_sight(unit_pos, hover_pos, |_| false)` — placeholder blocking
- Draws gizmo line segments through hex centers along the LOS path
- Green (`Color::srgb(0.2, 0.9, 0.2)`) if clear, red (`Color::srgb(0.9, 0.2, 0.2)`) if blocked
- Line drawn at Y=0.03, above all existing overlays

Why gizmos:

- Zero entity spawning/despawning overhead
- Auto-cleared each frame — perfect for transient hover feedback
- No state to manage

## System Registration

`draw_los_ray` appended to the existing chained `Update` tuple in `HexGridPlugin::build`. Reads
`HoveredHex` (from `update_hover`) and `SelectedUnit` (from `game_system` contract).

## File Changes

| File                            | Change                                     |
| ------------------------------- | ------------------------------------------ |
| `Cargo.toml`                    | Add `features = ["algorithms"]` to hexx    |
| `src/contracts/hex_grid.rs`     | Add `LineOfSightResult`, `VisibilityRange` |
| `docs/contracts/hex-grid.md`    | Mirror new contract types                  |
| `src/hex_grid/algorithms.rs`    | NEW — pure functions wrapping hexx         |
| `src/hex_grid/systems.rs`       | Add `draw_los_ray` system                  |
| `src/hex_grid/mod.rs`           | Add `mod algorithms;`, register new system |
| `src/hex_grid/tests.rs`         | Add algorithm and LOS tests                |
| `docs/plugins/hex-grid/spec.md` | Update requirements and success criteria   |
| `docs/plugins/hex-grid/log.md`  | Log design decisions                       |

## Testing

**Algorithm tests** (pure functions, no Bevy app):

- `line_of_sight_clear_path` — no blockers, clear, path length matches distance
- `line_of_sight_blocked` — blocker in path, blocked_by correct
- `line_of_sight_same_hex` — trivially clear, single-hex path
- `line_of_sight_adjacent` — distance 1, path is [from, to]
- `field_of_view_no_blockers` — returns all hexes in range
- `field_of_view_with_blocker` — hexes behind blocker excluded
- `field_of_view_range_zero` — returns only origin
- `find_path_straight_line` — shortest unobstructed path
- `find_path_around_obstacle` — routes around blocked hex
- `find_path_no_route` — walled off, returns None
- `neighbors_returns_six` — always 6
- `neighbors_are_adjacent` — each at distance 1
- `ring_at_radius` — correct count and distance
- `hex_range_count` — correct count and within distance

**System test** (Bevy test app):

- `los_ray_not_drawn_without_unit` — no panic when SelectedUnit is empty

**Spec success criteria:**

- [SC-14] Algorithm tests pass
- [SC-15] `draw_los_ray` registered, no panic without selected unit
- [SC-16] Contract types exist with required derives
- SC-BUILD, SC-CLIPPY, SC-TEST, SC-BOUNDARY retained

## Deferred Items (GitHub Issues to create)

- Fog of war system — requires sides/factions (#71, update with dependency note)
- Elevation-based LOS — per-tile elevation component + height interpolation
- LOS terrain blocking via property system — wire `blocks_los` property when #81 ships
