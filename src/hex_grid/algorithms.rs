//! Pure hex algorithms wrapping the `hexx` crate behind `HexPosition`.
//!
//! All functions take and return `HexPosition` so callers never interact
//! with `hexx::Hex` directly. No ECS dependencies — these are testable
//! without a Bevy app.

use std::collections::HashSet;

use crate::contracts::hex_grid::{HexPosition, LineOfSightResult};

/// Returns the 6 adjacent hex positions around `pos`.
pub fn neighbors(pos: HexPosition) -> [HexPosition; 6] {
    pos.to_hex().all_neighbors().map(HexPosition::from_hex)
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

/// Computes field of view from `origin` within `range` hexes.
///
/// Casts rays from `origin` to each hex on the outer ring and walks inward.
/// Blocking hexes themselves are visible (you can see a wall), but hexes
/// behind them are hidden. Based on `hexx` ray-casting with inclusive
/// blocker semantics.
pub fn field_of_view(
    origin: HexPosition,
    range: u32,
    is_blocking: impl Fn(HexPosition) -> bool,
) -> HashSet<HexPosition> {
    let origin_hex = origin.to_hex();
    origin_hex
        .ring(range)
        .flat_map(|target| {
            // `scan` state: `true` means the ray is still open (not yet blocked).
            origin_hex.line_to(target).scan(true, |open, h| {
                if !*open {
                    return None; // Ray was blocked on a previous hex.
                }
                let pos = HexPosition::from_hex(h);
                if is_blocking(pos) {
                    *open = false; // Include blocker, then close the ray.
                }
                Some(pos)
            })
        })
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
        cost(HexPosition::from_hex(current), HexPosition::from_hex(next))
    })
    .map(|path| path.into_iter().map(HexPosition::from_hex).collect())
}
