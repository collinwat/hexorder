//! Pure hex algorithms wrapping the `hexx` crate behind `HexPosition`.
//!
//! All functions take and return `HexPosition` so callers never interact
//! with `hexx::Hex` directly. No ECS dependencies — these are testable
//! without a Bevy app.

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
