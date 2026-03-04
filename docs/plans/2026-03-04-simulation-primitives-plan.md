# Simulation Primitives Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan
> task-by-task.

**Goal:** Add seeded RNG and generic table resolution primitives as contract types with a thin
simulation plugin.

**Architecture:** Contract-heavy approach — all data types and pure functions in
`crates/hexorder-contracts/src/simulation.rs`, thin Bevy plugin in `src/simulation/` for resource
hosting and observer events. Follows the exact pattern of `mechanics.rs`.

**Tech Stack:** Rust, Bevy 0.18, `rand` 0.9 + `rand_chacha` 0.9 for deterministic RNG.

**Design doc:** `docs/plans/2026-03-04-simulation-primitives-design.md`

---

## Pre-flight

Before starting any task, read these files for context:

- `docs/constitution.md` — project rules
- `docs/guides/git.md` — branching, commit conventions
- `docs/guides/bevy.md` — Bevy 0.18 patterns
- `docs/guides/contract.md` — contract protocol
- `crates/hexorder-contracts/src/mechanics.rs` — the pattern to follow (CRT types + pure functions)
- `crates/hexorder-contracts/src/lib.rs` — module registration
- `src/main.rs` — plugin registration order

---

### Task 1: Add `rand` and `rand_chacha` workspace dependencies

**Files:**

- Modify: `Cargo.toml:4-10` (workspace dependencies section)
- Modify: `crates/hexorder-contracts/Cargo.toml:8-18` (dependencies section)

**Step 1: Add workspace dependency pins**

In root `Cargo.toml`, add to `[workspace.dependencies]`:

```toml
rand = "0.9"
rand_chacha = "0.9"
```

**Step 2: Add contract crate dependency**

In `crates/hexorder-contracts/Cargo.toml`, add to `[dependencies]`:

```toml
rand = { workspace = true }
rand_chacha = { workspace = true }
```

**Step 3: Verify compilation**

Run: `cargo build -p hexorder-contracts` Expected: compiles with no errors

**Step 4: Commit**

```
chore(contracts): add rand and rand_chacha workspace dependencies
```

---

### Task 2: RNG contract types — write failing tests

**Files:**

- Create: `crates/hexorder-contracts/src/simulation.rs`
- Modify: `crates/hexorder-contracts/src/lib.rs` (add `pub mod simulation;`)

**Step 1: Register the module**

In `crates/hexorder-contracts/src/lib.rs`, add after the last `pub mod` line:

```rust
#[allow(dead_code)]
pub mod simulation;
```

**Step 2: Write type definitions and failing tests**

Create `crates/hexorder-contracts/src/simulation.rs` with the RNG types and test stubs. Start with
only the type definitions and tests — no function implementations yet.

```rust
//! Shared simulation types. See `docs/contracts/simulation.md`.
//!
//! Defines seeded RNG, die types, roll logging, lookup tables, and
//! resolution tables. These are generic simulation primitives per ADR-005.

use bevy::prelude::*;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};

use crate::game_system::TypeId;

// ---------------------------------------------------------------------------
// Seeded RNG
// ---------------------------------------------------------------------------

/// The type of die being rolled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum DieType {
    /// Standard 6-sided die (1-6).
    D6,
    /// 10-sided die (1-10).
    D10,
    /// Percentile die (1-100).
    D100,
    /// Arbitrary die with the given number of sides (1-sides).
    Custom { sides: u32 },
}

impl DieType {
    /// Returns the number of sides for this die type.
    #[must_use]
    pub fn sides(self) -> u32 {
        match self {
            Self::D6 => 6,
            Self::D10 => 10,
            Self::D100 => 100,
            Self::Custom { sides } => sides,
        }
    }
}

/// A single recorded die roll.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct RollRecord {
    /// Monotonically increasing index within this RNG session.
    pub roll_index: u64,
    /// What kind of die was rolled.
    pub die_type: DieType,
    /// The result value (1-based).
    pub result: u32,
    /// Human-readable context (e.g., "CRT resolution at 3:1").
    pub context: String,
}

/// Deterministic RNG resource wrapping ChaCha8Rng.
///
/// All randomness in simulation flows through this resource.
/// The full roll log enables deterministic replay and future
/// Monte Carlo analysis (#57).
#[derive(Resource, Debug)]
pub struct SimulationRng {
    /// The seed used to initialize this RNG.
    seed: u64,
    /// The underlying deterministic RNG.
    #[reflect(ignore)]
    rng: ChaCha8Rng,
    /// Complete log of all rolls made with this RNG.
    roll_log: Vec<RollRecord>,
    /// Monotonic counter for roll indexing.
    next_roll_index: u64,
}

impl SimulationRng {
    /// Create a new RNG with the given seed.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            rng: ChaCha8Rng::seed_from_u64(seed),
            roll_log: Vec::new(),
            next_roll_index: 0,
        }
    }

    /// Create a new RNG with a random seed.
    #[must_use]
    pub fn new_random() -> Self {
        let seed = rand::rng().random::<u64>();
        Self::new(seed)
    }

    /// Returns the seed used to initialize this RNG.
    #[must_use]
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Returns the full roll log.
    #[must_use]
    pub fn roll_log(&self) -> &[RollRecord] {
        &self.roll_log
    }

    /// Returns the number of rolls made so far.
    #[must_use]
    pub fn roll_count(&self) -> u64 {
        self.next_roll_index
    }
}
```

**Step 3: Write failing tests for RNG functions**

Add to the same file, at the bottom:

```rust
// ---------------------------------------------------------------------------
// RNG Functions (stubs — implementations in next task)
// ---------------------------------------------------------------------------

/// Roll a die of the given type, logging the result.
/// Returns a value in the range [1, die.sides()].
#[must_use]
pub fn roll_die(rng: &mut SimulationRng, die: DieType, context: &str) -> u32 {
    todo!()
}

/// Roll a value in the inclusive range [min, max], logging the result.
/// Uses `DieType::Custom` with `max - min + 1` sides.
#[must_use]
pub fn roll_range(rng: &mut SimulationRng, min: u32, max: u32, context: &str) -> u32 {
    todo!()
}

/// Reset the RNG with a new seed, clearing the roll log.
pub fn reset_rng(rng: &mut SimulationRng, seed: u64) {
    todo!()
}

/// Replay rolls from a seed — returns the first `count` d6 results.
/// Useful for verifying deterministic replay.
#[must_use]
pub fn replay_from_seed(seed: u64, count: u64) -> Vec<u32> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn die_type_sides() {
        assert_eq!(DieType::D6.sides(), 6);
        assert_eq!(DieType::D10.sides(), 10);
        assert_eq!(DieType::D100.sides(), 100);
        assert_eq!(DieType::Custom { sides: 20 }.sides(), 20);
    }

    #[test]
    fn new_rng_has_empty_log() {
        let rng = SimulationRng::new(42);
        assert_eq!(rng.seed(), 42);
        assert_eq!(rng.roll_count(), 0);
        assert!(rng.roll_log().is_empty());
    }

    #[test]
    fn roll_die_d6_in_range() {
        let mut rng = SimulationRng::new(42);
        for _ in 0..100 {
            let result = roll_die(&mut rng, DieType::D6, "test");
            assert!((1..=6).contains(&result), "d6 result {result} out of range");
        }
    }

    #[test]
    fn roll_die_logs_result() {
        let mut rng = SimulationRng::new(42);
        let result = roll_die(&mut rng, DieType::D6, "combat");
        assert_eq!(rng.roll_count(), 1);
        assert_eq!(rng.roll_log().len(), 1);
        let record = &rng.roll_log()[0];
        assert_eq!(record.roll_index, 0);
        assert_eq!(record.die_type, DieType::D6);
        assert_eq!(record.result, result);
        assert_eq!(record.context, "combat");
    }

    #[test]
    fn roll_die_deterministic() {
        let mut rng1 = SimulationRng::new(42);
        let mut rng2 = SimulationRng::new(42);
        for _ in 0..50 {
            assert_eq!(
                roll_die(&mut rng1, DieType::D6, ""),
                roll_die(&mut rng2, DieType::D6, ""),
            );
        }
    }

    #[test]
    fn roll_range_in_bounds() {
        let mut rng = SimulationRng::new(42);
        for _ in 0..100 {
            let result = roll_range(&mut rng, 3, 8, "test");
            assert!((3..=8).contains(&result), "range result {result} out of [3,8]");
        }
    }

    #[test]
    fn reset_rng_clears_log_and_reseeds() {
        let mut rng = SimulationRng::new(42);
        roll_die(&mut rng, DieType::D6, "before");
        assert_eq!(rng.roll_count(), 1);

        reset_rng(&mut rng, 99);
        assert_eq!(rng.seed(), 99);
        assert_eq!(rng.roll_count(), 0);
        assert!(rng.roll_log().is_empty());
    }

    #[test]
    fn replay_from_seed_matches_sequential_rolls() {
        let replayed = replay_from_seed(42, 10);
        assert_eq!(replayed.len(), 10);

        let mut rng = SimulationRng::new(42);
        for expected in &replayed {
            assert_eq!(roll_die(&mut rng, DieType::D6, ""), *expected);
        }
    }

    #[test]
    fn roll_die_d100_in_range() {
        let mut rng = SimulationRng::new(42);
        for _ in 0..100 {
            let result = roll_die(&mut rng, DieType::D100, "test");
            assert!((1..=100).contains(&result), "d100 result {result} out of range");
        }
    }

    #[test]
    fn roll_die_custom_sides() {
        let mut rng = SimulationRng::new(42);
        let die = DieType::Custom { sides: 20 };
        for _ in 0..100 {
            let result = roll_die(&mut rng, die, "test");
            assert!((1..=20).contains(&result), "d20 result {result} out of range");
        }
    }

    #[test]
    fn roll_log_index_increments() {
        let mut rng = SimulationRng::new(42);
        roll_die(&mut rng, DieType::D6, "first");
        roll_die(&mut rng, DieType::D10, "second");
        roll_die(&mut rng, DieType::D100, "third");
        assert_eq!(rng.roll_log()[0].roll_index, 0);
        assert_eq!(rng.roll_log()[1].roll_index, 1);
        assert_eq!(rng.roll_log()[2].roll_index, 2);
    }
}
```

**Step 4: Run tests to verify they fail**

Run: `cargo test -p hexorder-contracts simulation -- --no-capture 2>&1 | head -30` Expected: tests
fail with `not yet implemented` panics

**Step 5: Commit**

```
test(contracts): add failing tests for simulation RNG types
```

---

### Task 3: RNG pure functions — make tests pass

**Files:**

- Modify: `crates/hexorder-contracts/src/simulation.rs` (replace `todo!()` stubs)

**Step 1: Implement `roll_die`**

Replace the `roll_die` stub:

```rust
pub fn roll_die(rng: &mut SimulationRng, die: DieType, context: &str) -> u32 {
    let sides = die.sides();
    let result = rng.rng.random_range(1..=sides);
    let record = RollRecord {
        roll_index: rng.next_roll_index,
        die_type: die,
        result,
        context: context.to_string(),
    };
    rng.roll_log.push(record);
    rng.next_roll_index += 1;
    result
}
```

**Step 2: Implement `roll_range`**

Replace the `roll_range` stub:

```rust
pub fn roll_range(rng: &mut SimulationRng, min: u32, max: u32, context: &str) -> u32 {
    let sides = max - min + 1;
    let die = DieType::Custom { sides };
    let raw = roll_die(rng, die, context);
    // roll_die returns [1, sides], shift to [min, max]
    raw - 1 + min
}
```

**Step 3: Implement `reset_rng`**

Replace the `reset_rng` stub:

```rust
pub fn reset_rng(rng: &mut SimulationRng, seed: u64) {
    rng.seed = seed;
    rng.rng = ChaCha8Rng::seed_from_u64(seed);
    rng.roll_log.clear();
    rng.next_roll_index = 0;
}
```

**Step 4: Implement `replay_from_seed`**

Replace the `replay_from_seed` stub:

```rust
pub fn replay_from_seed(seed: u64, count: u64) -> Vec<u32> {
    let mut rng = SimulationRng::new(seed);
    (0..count)
        .map(|_| roll_die(&mut rng, DieType::D6, ""))
        .collect()
}
```

**Step 5: Run tests**

Run: `cargo test -p hexorder-contracts simulation` Expected: all RNG tests pass

**Step 6: Run clippy**

Run: `cargo clippy -p hexorder-contracts --all-targets` Expected: zero warnings

**Step 7: Commit**

```
feat(contracts): implement seeded RNG with deterministic replay
```

---

### Task 4: Table resolution types — write failing tests

**Files:**

- Modify: `crates/hexorder-contracts/src/simulation.rs` (append table types and tests)

**Step 1: Add table type definitions**

Append after the RNG section, before `#[cfg(test)]`:

```rust
// ---------------------------------------------------------------------------
// Table Resolution
// ---------------------------------------------------------------------------

/// How a resolution table column input is calculated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum ColumnType {
    /// Column threshold is compared against input_a / input_b.
    Ratio,
    /// Column threshold is compared against input_a - input_b.
    Differential,
    /// Column threshold is compared against input_a directly.
    Direct,
}

/// A column header in a resolution table.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct TableColumn {
    /// Display label (e.g., "3:1" or "+2").
    pub label: String,
    /// How the input value is calculated for comparison.
    pub column_type: ColumnType,
    /// Minimum input value to select this column.
    pub threshold: f64,
}

/// A row header in a resolution table (die roll range).
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct TableRow {
    /// Display label (e.g., "1-2").
    pub label: String,
    /// Minimum die value (inclusive) for this row.
    pub value_min: u32,
    /// Maximum die value (inclusive) for this row.
    pub value_max: u32,
}

/// The result of a table lookup.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub enum TableResult {
    /// A label-only outcome (e.g., "NE", "DR").
    Text(String),
    /// A numeric value (e.g., movement cost 2.0).
    NumericValue(f64),
    /// A modifier to a named property (e.g., morale -1).
    PropertyModifier {
        property: String,
        delta: f64,
    },
}

/// A column shift modifier applied during table resolution.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct ColumnModifier {
    /// Display name of the modifier.
    pub name: String,
    /// Signed column shift (positive = rightward, negative = leftward).
    pub column_shift: i32,
    /// Optional absolute cap on cumulative shift after this modifier.
    pub cap: Option<i32>,
    /// Priority for evaluation order (higher = evaluated first).
    pub priority: u32,
}

/// A 2D resolution table: columns × rows → outcomes.
///
/// Generalizes the CRT pattern to any 2D lookup: input determines
/// the column, a die roll determines the row, and the intersection
/// yields the result.
#[derive(Resource, Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct ResolutionTable {
    pub id: TypeId,
    pub name: String,
    pub columns: Vec<TableColumn>,
    pub rows: Vec<TableRow>,
    /// Outcome grid indexed as `outcomes[row][column]`.
    pub outcomes: Vec<Vec<TableResult>>,
}

impl Default for ResolutionTable {
    fn default() -> Self {
        Self {
            id: TypeId::new(),
            name: "Resolution Table".to_string(),
            columns: Vec::new(),
            rows: Vec::new(),
            outcomes: Vec::new(),
        }
    }
}

/// A 1D lookup table: input threshold → result.
///
/// Entries are ordered by ascending threshold. The rightmost entry
/// whose threshold the input meets or exceeds is selected.
#[derive(Resource, Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct LookupTable {
    pub id: TypeId,
    pub name: String,
    pub entries: Vec<LookupEntry>,
}

impl Default for LookupTable {
    fn default() -> Self {
        Self {
            id: TypeId::new(),
            name: "Lookup Table".to_string(),
            entries: Vec::new(),
        }
    }
}

/// A single entry in a lookup table.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct LookupEntry {
    /// Display label.
    pub label: String,
    /// Minimum input value to select this entry.
    pub threshold: f64,
    /// The result when this entry is selected.
    pub result: TableResult,
}

/// Result of a full 2D table resolution.
#[derive(Debug, Clone)]
pub struct TableResolution {
    pub column_index: usize,
    pub row_index: usize,
    pub column_label: String,
    pub row_label: String,
    pub result: TableResult,
}

// ---------------------------------------------------------------------------
// Table Resolution Functions (stubs — implementations in next task)
// ---------------------------------------------------------------------------

/// Resolve a 1D lookup: find the rightmost entry whose threshold
/// the input meets or exceeds.
#[must_use]
pub fn resolve_lookup(table: &LookupTable, input: f64) -> Option<usize> {
    todo!()
}

/// Find the best matching column for the given inputs.
/// Returns the index of the rightmost column whose threshold is met.
#[must_use]
pub fn find_table_column(
    input_a: f64,
    input_b: f64,
    columns: &[TableColumn],
) -> Option<usize> {
    todo!()
}

/// Find the row matching a given die roll value.
#[must_use]
pub fn find_table_row(roll: u32, rows: &[TableRow]) -> Option<usize> {
    todo!()
}

/// Resolve a full 2D table lookup.
#[must_use]
pub fn resolve_table(
    table: &ResolutionTable,
    input_a: f64,
    input_b: f64,
    roll: u32,
) -> Option<TableResolution> {
    todo!()
}

/// Evaluate column modifiers in priority order (highest first).
/// Returns final shift and display list.
#[must_use]
pub fn evaluate_column_modifiers(
    modifiers: &[ColumnModifier],
    column_count: usize,
) -> (i32, Vec<(String, i32)>) {
    todo!()
}

/// Apply a column shift to a base index, clamping to bounds.
#[must_use]
pub fn apply_column_shift(base_column: usize, shift: i32, column_count: usize) -> usize {
    todo!()
}
```

**Step 2: Add failing tests for table resolution**

Append to the existing `mod tests` block:

```rust
    // --- Table resolution tests ---

    fn test_resolution_table() -> ResolutionTable {
        ResolutionTable {
            id: TypeId::new(),
            name: "Test Table".to_string(),
            columns: vec![
                TableColumn {
                    label: "1:2".to_string(),
                    column_type: ColumnType::Ratio,
                    threshold: 0.5,
                },
                TableColumn {
                    label: "1:1".to_string(),
                    column_type: ColumnType::Ratio,
                    threshold: 1.0,
                },
                TableColumn {
                    label: "2:1".to_string(),
                    column_type: ColumnType::Ratio,
                    threshold: 2.0,
                },
            ],
            rows: vec![
                TableRow { label: "1-2".to_string(), value_min: 1, value_max: 2 },
                TableRow { label: "3-4".to_string(), value_min: 3, value_max: 4 },
                TableRow { label: "5-6".to_string(), value_min: 5, value_max: 6 },
            ],
            outcomes: vec![
                vec![
                    TableResult::Text("AE".to_string()),
                    TableResult::Text("NE".to_string()),
                    TableResult::Text("DR".to_string()),
                ],
                vec![
                    TableResult::Text("NE".to_string()),
                    TableResult::Text("DR".to_string()),
                    TableResult::Text("DE".to_string()),
                ],
                vec![
                    TableResult::NumericValue(1.0),
                    TableResult::NumericValue(2.0),
                    TableResult::NumericValue(3.0),
                ],
            ],
        }
    }

    fn test_lookup_table() -> LookupTable {
        LookupTable {
            id: TypeId::new(),
            name: "Movement Cost".to_string(),
            entries: vec![
                LookupEntry {
                    label: "Open".to_string(),
                    threshold: 0.0,
                    result: TableResult::NumericValue(1.0),
                },
                LookupEntry {
                    label: "Rough".to_string(),
                    threshold: 2.0,
                    result: TableResult::NumericValue(2.0),
                },
                LookupEntry {
                    label: "Mountain".to_string(),
                    threshold: 5.0,
                    result: TableResult::NumericValue(3.0),
                },
            ],
        }
    }

    #[test]
    fn resolve_lookup_selects_rightmost_match() {
        let table = test_lookup_table();
        let idx = resolve_lookup(&table, 3.0);
        assert_eq!(idx, Some(1)); // Rough (threshold 2.0)
    }

    #[test]
    fn resolve_lookup_exact_threshold() {
        let table = test_lookup_table();
        let idx = resolve_lookup(&table, 5.0);
        assert_eq!(idx, Some(2)); // Mountain (threshold 5.0)
    }

    #[test]
    fn resolve_lookup_below_all() {
        let table = test_lookup_table();
        let idx = resolve_lookup(&table, -1.0);
        assert!(idx.is_none());
    }

    #[test]
    fn find_table_column_ratio() {
        let table = test_resolution_table();
        // 6 / 2 = 3.0 → meets 2:1 threshold
        let col = find_table_column(6.0, 2.0, &table.columns);
        assert_eq!(col, Some(2));
    }

    #[test]
    fn find_table_column_differential() {
        let columns = vec![TableColumn {
            label: "+2".to_string(),
            column_type: ColumnType::Differential,
            threshold: 2.0,
        }];
        let col = find_table_column(5.0, 2.0, &columns);
        assert_eq!(col, Some(0)); // 5-2=3 >= 2
    }

    #[test]
    fn find_table_column_direct() {
        let columns = vec![
            TableColumn {
                label: "Low".to_string(),
                column_type: ColumnType::Direct,
                threshold: 0.0,
            },
            TableColumn {
                label: "High".to_string(),
                column_type: ColumnType::Direct,
                threshold: 10.0,
            },
        ];
        let col = find_table_column(15.0, 999.0, &columns);
        assert_eq!(col, Some(1)); // input_a=15 >= 10
    }

    #[test]
    fn find_table_column_below_all() {
        let table = test_resolution_table();
        let col = find_table_column(1.0, 10.0, &table.columns);
        assert!(col.is_none());
    }

    #[test]
    fn find_table_row_matching() {
        let table = test_resolution_table();
        assert_eq!(find_table_row(1, &table.rows), Some(0));
        assert_eq!(find_table_row(4, &table.rows), Some(1));
        assert_eq!(find_table_row(6, &table.rows), Some(2));
    }

    #[test]
    fn find_table_row_no_match() {
        let table = test_resolution_table();
        assert!(find_table_row(7, &table.rows).is_none());
    }

    #[test]
    fn resolve_table_full() {
        let table = test_resolution_table();
        // 6/2=3:1 → col 2; roll 3 → row 1 → "DE"
        let result = resolve_table(&table, 6.0, 2.0, 3);
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.column_index, 2);
        assert_eq!(r.row_index, 1);
        assert!(matches!(r.result, TableResult::Text(ref s) if s == "DE"));
    }

    #[test]
    fn resolve_table_no_column() {
        let table = test_resolution_table();
        let result = resolve_table(&table, 1.0, 10.0, 3);
        assert!(result.is_none());
    }

    #[test]
    fn resolve_table_no_row() {
        let table = test_resolution_table();
        let result = resolve_table(&table, 6.0, 2.0, 99);
        assert!(result.is_none());
    }

    #[test]
    fn resolve_table_numeric_result() {
        let table = test_resolution_table();
        // 6/2=3:1 → col 2; roll 5 → row 2 → NumericValue(3.0)
        let r = resolve_table(&table, 6.0, 2.0, 5).unwrap();
        assert!(matches!(r.result, TableResult::NumericValue(v) if (v - 3.0).abs() < f64::EPSILON));
    }

    #[test]
    fn evaluate_column_modifiers_empty() {
        let (total, display) = evaluate_column_modifiers(&[], 3);
        assert_eq!(total, 0);
        assert!(display.is_empty());
    }

    #[test]
    fn evaluate_column_modifiers_with_cap() {
        let mods = vec![ColumnModifier {
            name: "Terrain".to_string(),
            column_shift: -3,
            priority: 10,
            cap: Some(2),
        }];
        let (total, display) = evaluate_column_modifiers(&mods, 5);
        assert_eq!(total, -2); // -3 clamped to [-2, 2]
        assert_eq!(display.len(), 1);
    }

    #[test]
    fn evaluate_column_modifiers_priority_order() {
        let mods = vec![
            ColumnModifier { name: "Low".to_string(), column_shift: 1, priority: 1, cap: None },
            ColumnModifier { name: "High".to_string(), column_shift: 2, priority: 10, cap: None },
        ];
        let (total, display) = evaluate_column_modifiers(&mods, 10);
        assert_eq!(total, 3);
        assert_eq!(display[0].0, "High");
        assert_eq!(display[1].0, "Low");
    }

    #[test]
    fn apply_column_shift_basic() {
        assert_eq!(apply_column_shift(1, 2, 5), 3);
    }

    #[test]
    fn apply_column_shift_clamp_right() {
        assert_eq!(apply_column_shift(3, 5, 5), 4);
    }

    #[test]
    fn apply_column_shift_clamp_left() {
        assert_eq!(apply_column_shift(1, -5, 5), 0);
    }

    #[test]
    fn apply_column_shift_zero_columns() {
        assert_eq!(apply_column_shift(0, 3, 0), 0);
    }

    #[test]
    fn resolution_table_ron_round_trip() {
        let table = test_resolution_table();
        let ron_str = ron::to_string(&table).expect("serialize");
        let deserialized: ResolutionTable = ron::from_str(&ron_str).expect("deserialize");
        assert_eq!(deserialized.columns.len(), 3);
        assert_eq!(deserialized.rows.len(), 3);
        assert_eq!(deserialized.outcomes.len(), 3);
    }

    #[test]
    fn lookup_table_ron_round_trip() {
        let table = test_lookup_table();
        let ron_str = ron::to_string(&table).expect("serialize");
        let deserialized: LookupTable = ron::from_str(&ron_str).expect("deserialize");
        assert_eq!(deserialized.entries.len(), 3);
    }
```

**Step 3: Run tests to verify they fail**

Run:
`cargo test -p hexorder-contracts simulation -- 2>&1 | grep -E "(FAILED|test result)" | head -5`
Expected: table resolution tests fail with `not yet implemented`, RNG tests still pass

**Step 4: Commit**

```
test(contracts): add failing tests for table resolution types
```

---

### Task 5: Table resolution pure functions — make tests pass

**Files:**

- Modify: `crates/hexorder-contracts/src/simulation.rs` (replace table function stubs)

**Step 1: Implement `resolve_lookup`**

```rust
pub fn resolve_lookup(table: &LookupTable, input: f64) -> Option<usize> {
    let mut best: Option<usize> = None;
    for (i, entry) in table.entries.iter().enumerate() {
        if input >= entry.threshold {
            best = Some(i);
        }
    }
    best
}
```

**Step 2: Implement `find_table_column`**

```rust
pub fn find_table_column(
    input_a: f64,
    input_b: f64,
    columns: &[TableColumn],
) -> Option<usize> {
    let mut best: Option<usize> = None;
    for (i, col) in columns.iter().enumerate() {
        let value = match col.column_type {
            ColumnType::Ratio => {
                if input_b <= 0.0 {
                    f64::INFINITY
                } else {
                    input_a / input_b
                }
            }
            ColumnType::Differential => input_a - input_b,
            ColumnType::Direct => input_a,
        };
        if value >= col.threshold {
            best = Some(i);
        }
    }
    best
}
```

**Step 3: Implement `find_table_row`**

```rust
pub fn find_table_row(roll: u32, rows: &[TableRow]) -> Option<usize> {
    rows.iter()
        .position(|row| roll >= row.value_min && roll <= row.value_max)
}
```

**Step 4: Implement `resolve_table`**

```rust
pub fn resolve_table(
    table: &ResolutionTable,
    input_a: f64,
    input_b: f64,
    roll: u32,
) -> Option<TableResolution> {
    let col_idx = find_table_column(input_a, input_b, &table.columns)?;
    let row_idx = find_table_row(roll, &table.rows)?;
    let result = table.outcomes.get(row_idx).and_then(|row| row.get(col_idx))?;

    Some(TableResolution {
        column_index: col_idx,
        row_index: row_idx,
        column_label: table.columns[col_idx].label.clone(),
        row_label: table.rows[row_idx].label.clone(),
        result: result.clone(),
    })
}
```

**Step 5: Implement `evaluate_column_modifiers`**

```rust
pub fn evaluate_column_modifiers(
    modifiers: &[ColumnModifier],
    column_count: usize,
) -> (i32, Vec<(String, i32)>) {
    let mut sorted: Vec<&ColumnModifier> = modifiers.iter().collect();
    sorted.sort_by(|a, b| b.priority.cmp(&a.priority));

    let mut total_shift: i32 = 0;
    let mut display: Vec<(String, i32)> = Vec::with_capacity(sorted.len());

    for modifier in &sorted {
        total_shift += modifier.column_shift;
        if let Some(cap) = modifier.cap {
            total_shift = total_shift.clamp(-cap, cap);
        }
        display.push((modifier.name.clone(), modifier.column_shift));
    }

    if column_count > 0 {
        let max_shift = (column_count - 1) as i32;
        total_shift = total_shift.clamp(-max_shift, max_shift);
    }

    (total_shift, display)
}
```

**Step 6: Implement `apply_column_shift`**

```rust
pub fn apply_column_shift(base_column: usize, shift: i32, column_count: usize) -> usize {
    if column_count == 0 {
        return 0;
    }
    let shifted = base_column as i32 + shift;
    shifted.clamp(0, (column_count - 1) as i32) as usize
}
```

**Step 7: Run all tests**

Run: `cargo test -p hexorder-contracts simulation` Expected: all tests pass (RNG + table resolution)

**Step 8: Run clippy**

Run: `cargo clippy -p hexorder-contracts --all-targets` Expected: zero warnings

**Step 9: Commit**

```
feat(contracts): implement table resolution with 1D lookup and 2D grid
```

---

### Task 6: Contract spec documentation

**Files:**

- Create: `docs/contracts/simulation.md`

**Step 1: Write the contract spec**

Follow the pattern of `docs/contracts/mechanics.md`. Document all public types, their fields, and
the pure functions. Include consumers and producers sections.

Key sections:

- **Purpose**: Generic simulation primitives (seeded RNG, table resolution) per ADR-005
- **Consumers**: `simulation` plugin (runtime host), `rules_engine` (future), `editor_ui` (future)
- **Producers**: `simulation` plugin (inserts `SimulationRng`)
- **Types**: All structs and enums from `simulation.rs`
- **Functions**: All pure functions with signatures

**Step 2: Commit**

```
docs(contracts): add simulation contract spec
```

---

### Task 7: Simulation plugin — scaffold and register

**Files:**

- Create: `src/simulation/mod.rs`
- Create: `src/simulation/events.rs`
- Create: `src/simulation/systems.rs`
- Create: `src/simulation/tests.rs`
- Modify: `src/main.rs` (add `mod simulation;` and plugin registration)

**Step 1: Write failing integration test**

Create `src/simulation/tests.rs`:

```rust
use bevy::prelude::*;
use hexorder_contracts::simulation::{
    DieType, SimulationRng, roll_die,
};

/// SimulationPlugin inserts SimulationRng resource.
#[test]
fn simulation_rng_resource_available() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(super::SimulationPlugin);

    assert!(
        app.world().get_resource::<SimulationRng>().is_some(),
        "SimulationRng should exist after plugin build"
    );
}

/// DieRolled event fires when a die is rolled through the plugin.
#[test]
fn die_rolled_event_fires() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(super::SimulationPlugin);
    app.update();

    // Roll a die via the resource.
    let mut rng = app.world_mut().resource_mut::<SimulationRng>();
    let _result = roll_die(&mut rng, DieType::D6, "test");

    assert_eq!(rng.roll_count(), 1);
}
```

**Step 2: Create events**

Create `src/simulation/events.rs`:

```rust
use bevy::prelude::*;
use hexorder_contracts::game_system::TypeId;
use hexorder_contracts::simulation::{RollRecord, TableResolution};

/// Fired when a die is rolled (for UI display, logging).
#[derive(Event, Debug, Clone)]
pub struct DieRolled {
    pub record: RollRecord,
}

/// Fired when a table resolution completes.
#[derive(Event, Debug, Clone)]
pub struct TableResolved {
    pub table_id: TypeId,
    pub resolution: TableResolution,
    pub roll: Option<RollRecord>,
}
```

**Step 3: Create systems (placeholder observers)**

Create `src/simulation/systems.rs`:

```rust
use bevy::prelude::*;

use super::events::{DieRolled, TableResolved};

/// Observer: log die roll events (future: update UI).
pub fn on_die_rolled(_trigger: Trigger<DieRolled>) {
    // Future: notify editor UI, update roll display panel.
}

/// Observer: log table resolution events (future: update UI).
pub fn on_table_resolved(_trigger: Trigger<TableResolved>) {
    // Future: notify editor UI, apply outcome effects.
}
```

**Step 4: Create plugin mod.rs**

Create `src/simulation/mod.rs`:

```rust
//! Simulation plugin.
//!
//! Hosts the `SimulationRng` resource and provides observer events
//! for die rolls and table resolutions. All types and pure functions
//! live in `hexorder_contracts::simulation`.

use bevy::prelude::*;
use hexorder_contracts::simulation::SimulationRng;

mod events;
mod systems;

#[cfg(test)]
mod tests;

/// Plugin that provides simulation primitives: seeded RNG and
/// table resolution runtime support.
#[derive(Debug)]
pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SimulationRng::new_random());
        app.add_observer(systems::on_die_rolled);
        app.add_observer(systems::on_table_resolved);
    }
}
```

**Step 5: Register in main.rs**

Add `mod simulation;` after the other `mod` declarations (between `rules_engine` and `scripting`).

Add plugin registration after `rules_engine::RulesEnginePlugin`:

```rust
.add_plugins(simulation::SimulationPlugin)
```

**Step 6: Run tests**

Run: `cargo test --lib simulation` Expected: simulation plugin tests pass

**Step 7: Run full check**

Run: `mise check` Expected: all checks pass (fmt, clippy, test, boundary, unwrap)

**Step 8: Commit**

```
feat(simulation): add simulation plugin with RNG resource and observer events
```

---

### Task 8: Integration test — RNG + table resolution end-to-end

**Files:**

- Modify: `src/simulation/tests.rs` (add integration test)

**Step 1: Write integration test**

Append to `src/simulation/tests.rs`:

```rust
use hexorder_contracts::simulation::{
    ColumnType, ResolutionTable, TableColumn, TableResult, TableRow,
    resolve_table, reset_rng,
};

/// End-to-end: seed RNG → roll → resolve table → verify deterministic.
#[test]
fn rng_table_resolution_deterministic() {
    let table = ResolutionTable {
        id: hexorder_contracts::game_system::TypeId::new(),
        name: "Test CRT".to_string(),
        columns: vec![
            TableColumn {
                label: "1:2".to_string(),
                column_type: ColumnType::Ratio,
                threshold: 0.5,
            },
            TableColumn {
                label: "1:1".to_string(),
                column_type: ColumnType::Ratio,
                threshold: 1.0,
            },
            TableColumn {
                label: "2:1".to_string(),
                column_type: ColumnType::Ratio,
                threshold: 2.0,
            },
        ],
        rows: vec![
            TableRow { label: "1-2".to_string(), value_min: 1, value_max: 2 },
            TableRow { label: "3-4".to_string(), value_min: 3, value_max: 4 },
            TableRow { label: "5-6".to_string(), value_min: 5, value_max: 6 },
        ],
        outcomes: vec![
            vec![
                TableResult::Text("AE".to_string()),
                TableResult::Text("NE".to_string()),
                TableResult::Text("DR".to_string()),
            ],
            vec![
                TableResult::Text("NE".to_string()),
                TableResult::Text("DR".to_string()),
                TableResult::Text("DE".to_string()),
            ],
            vec![
                TableResult::Text("EX".to_string()),
                TableResult::Text("SL".to_string()),
                TableResult::Text("DSL".to_string()),
            ],
        ],
    };

    // Run the same resolution twice with the same seed.
    let mut rng1 = SimulationRng::new(42);
    let roll1 = roll_die(&mut rng1, DieType::D6, "combat");
    let result1 = resolve_table(&table, 6.0, 2.0, roll1);

    let mut rng2 = SimulationRng::new(42);
    let roll2 = roll_die(&mut rng2, DieType::D6, "combat");
    let result2 = resolve_table(&table, 6.0, 2.0, roll2);

    assert_eq!(roll1, roll2, "Same seed should produce same roll");
    assert!(result1.is_some());
    assert!(result2.is_some());
    let r1 = result1.unwrap();
    let r2 = result2.unwrap();
    assert_eq!(r1.column_index, r2.column_index);
    assert_eq!(r1.row_index, r2.row_index);
    assert_eq!(r1.column_label, r2.column_label);
    assert_eq!(r1.row_label, r2.row_label);
}

/// Reset and re-roll produces the same sequence.
#[test]
fn reset_replays_same_sequence() {
    let mut rng = SimulationRng::new(42);
    let first_run: Vec<u32> = (0..10)
        .map(|_| roll_die(&mut rng, DieType::D6, ""))
        .collect();

    reset_rng(&mut rng, 42);
    let second_run: Vec<u32> = (0..10)
        .map(|_| roll_die(&mut rng, DieType::D6, ""))
        .collect();

    assert_eq!(first_run, second_run);
}
```

**Step 2: Run tests**

Run: `cargo test --lib simulation` Expected: all simulation tests pass

**Step 3: Run full audit**

Run: `mise check` Expected: all checks pass

**Step 4: Commit**

```
test(simulation): add RNG + table resolution integration tests
```

---

### Task 9: Create deferred GitHub Issues

**Step 1: Search for existing issues to avoid duplicates**

Run:

```bash
gh issue list --search "migrate CRT" --state all
gh issue list --search "roll display" --state all
gh issue list --search "table editor" --state all
```

**Step 2: Create deferred issues**

Create these GitHub Issues (adjust if duplicates found):

1. "Migrate CRT types to generic ResolutionTable" — `type:tech-debt`, `area:contracts`
2. "Roll display UI — show recent die rolls in editor" — `type:feature`, `area:editor-ui`
3. "Table editor UI — visual 2D grid editing for resolution tables" — `type:feature`,
   `area:editor-ui`

**Step 3: Commit** (no code changes, just issue creation)

No commit needed.

---

### Task 10: Plugin spec and log documentation

**Files:**

- Create: `docs/plugins/simulation/spec.md`
- Create: `docs/plugins/simulation/log.md`

**Step 1: Write plugin spec**

Follow the pattern of other plugin specs. Include:

- Plugin purpose (simulation primitives host)
- Dependencies (hexorder-contracts simulation types)
- Resources provided (SimulationRng)
- Events (DieRolled, TableResolved)
- Success criteria

**Step 2: Write initial log entry**

Record the initial implementation decisions.

**Step 3: Commit**

```
docs(simulation): add plugin spec and initial log
```

---

## Summary

| Task | What                        | Files touched                                        | Commit message                                                                  |
| ---- | --------------------------- | ---------------------------------------------------- | ------------------------------------------------------------------------------- |
| 1    | Workspace deps              | `Cargo.toml`, `crates/hexorder-contracts/Cargo.toml` | `chore(contracts): add rand and rand_chacha workspace dependencies`             |
| 2    | RNG types + failing tests   | `simulation.rs`, `lib.rs`                            | `test(contracts): add failing tests for simulation RNG types`                   |
| 3    | RNG implementations         | `simulation.rs`                                      | `feat(contracts): implement seeded RNG with deterministic replay`               |
| 4    | Table types + failing tests | `simulation.rs`                                      | `test(contracts): add failing tests for table resolution types`                 |
| 5    | Table implementations       | `simulation.rs`                                      | `feat(contracts): implement table resolution with 1D lookup and 2D grid`        |
| 6    | Contract spec doc           | `docs/contracts/simulation.md`                       | `docs(contracts): add simulation contract spec`                                 |
| 7    | Simulation plugin           | `src/simulation/`, `src/main.rs`                     | `feat(simulation): add simulation plugin with RNG resource and observer events` |
| 8    | Integration tests           | `src/simulation/tests.rs`                            | `test(simulation): add RNG + table resolution integration tests`                |
| 9    | Deferred issues             | GitHub Issues                                        | (no commit)                                                                     |
| 10   | Plugin docs                 | `docs/plugins/simulation/`                           | `docs(simulation): add plugin spec and initial log`                             |
