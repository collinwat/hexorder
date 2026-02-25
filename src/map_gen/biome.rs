//! Biome table logic -- maps elevation values to terrain type names.

use std::collections::HashMap;

use hexorder_contracts::hex_grid::HexPosition;

use super::components::{BiomeEntry, BiomeTable};

/// Look up the terrain name for a given elevation value.
///
/// Returns `None` if no biome entry covers the given elevation.
#[cfg(test)]
pub fn lookup_biome(table: &BiomeTable, elevation: f64) -> Option<&str> {
    for (i, entry) in table.entries.iter().enumerate() {
        let is_last = i == table.entries.len() - 1;
        if is_last {
            // Last entry: inclusive on both ends
            if elevation >= entry.min_elevation && elevation <= entry.max_elevation {
                return Some(&entry.terrain_name);
            }
        } else {
            // Non-last entries: inclusive min, exclusive max
            if elevation >= entry.min_elevation && elevation < entry.max_elevation {
                return Some(&entry.terrain_name);
            }
        }
    }
    None
}

/// Look up the biome entry index for a given elevation value.
///
/// Returns `None` if no biome entry covers the given elevation.
pub fn lookup_biome_index(table: &BiomeTable, elevation: f64) -> Option<usize> {
    for (i, entry) in table.entries.iter().enumerate() {
        let is_last = i == table.entries.len() - 1;
        if is_last {
            if elevation >= entry.min_elevation && elevation <= entry.max_elevation {
                return Some(i);
            }
        } else if elevation >= entry.min_elevation && elevation < entry.max_elevation {
            return Some(i);
        }
    }
    None
}

/// Apply a biome table to a heightmap, returning terrain names per hex position.
///
/// Positions whose elevation doesn't match any biome entry are omitted from
/// the result.
#[cfg(test)]
pub fn apply_biome_table(
    heightmap: &HashMap<HexPosition, f64>,
    table: &BiomeTable,
) -> HashMap<HexPosition, String> {
    let mut result = HashMap::with_capacity(heightmap.len());

    for (&pos, &elevation) in heightmap {
        if let Some(name) = lookup_biome(table, elevation) {
            result.insert(pos, name.to_string());
        }
    }

    result
}

/// Apply a biome table to a heightmap, returning biome entry indices per hex
/// position. Used for ordinal mapping to entity types.
///
/// Positions whose elevation doesn't match any biome entry are omitted.
pub fn apply_biome_table_indexed(
    heightmap: &HashMap<HexPosition, f64>,
    table: &BiomeTable,
) -> HashMap<HexPosition, usize> {
    let mut result = HashMap::with_capacity(heightmap.len());

    for (&pos, &elevation) in heightmap {
        if let Some(index) = lookup_biome_index(table, elevation) {
            result.insert(pos, index);
        }
    }

    result
}

/// Validate that a biome table covers the full [0.0, 1.0] range with no gaps.
pub fn validate_biome_table(table: &BiomeTable) -> Result<(), BiomeTableError> {
    if table.entries.is_empty() {
        return Err(BiomeTableError::Empty);
    }

    let sorted: Vec<&BiomeEntry> = {
        let mut entries: Vec<_> = table.entries.iter().collect();
        entries.sort_by(|a, b| {
            a.min_elevation
                .partial_cmp(&b.min_elevation)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        entries
    };

    if sorted[0].min_elevation > 0.0 {
        return Err(BiomeTableError::GapAtStart(sorted[0].min_elevation));
    }

    if let Some(last) = sorted.last()
        && last.max_elevation < 1.0
    {
        return Err(BiomeTableError::GapAtEnd(last.max_elevation));
    }

    for window in sorted.windows(2) {
        let current = window[0];
        let next = window[1];
        if (current.max_elevation - next.min_elevation).abs() > f64::EPSILON {
            return Err(BiomeTableError::Gap {
                after: current.terrain_name.clone(),
                before: next.terrain_name.clone(),
            });
        }
    }

    Ok(())
}

/// Errors from biome table validation.
#[derive(Debug)]
pub enum BiomeTableError {
    Empty,
    GapAtStart(f64),
    GapAtEnd(f64),
    Gap { after: String, before: String },
}

impl std::fmt::Display for BiomeTableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "biome table is empty"),
            Self::GapAtStart(min) => {
                write!(f, "biome table does not start at 0.0 (starts at {min})")
            }
            Self::GapAtEnd(max) => {
                write!(f, "biome table does not reach 1.0 (ends at {max})")
            }
            Self::Gap { after, before } => {
                write!(f, "gap between biome entries '{after}' and '{before}'")
            }
        }
    }
}
