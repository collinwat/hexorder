//! Pure hex algorithms wrapping the `hexx` crate behind `HexPosition`.
//!
//! All functions take and return `HexPosition` so callers never interact
//! with `hexx::Hex` directly. No ECS dependencies — these are testable
//! without a Bevy app.

use crate::contracts::hex_grid::HexPosition;

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
