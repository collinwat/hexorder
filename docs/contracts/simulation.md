# Contract: simulation

## Purpose

Defines generic simulation primitives per ADR-005: seeded RNG with deterministic replay, die types,
roll logging, 1D lookup tables, 2D resolution tables, and column modifiers. These primitives are
domain-agnostic — they pass the space-game test and can express any tabletop resolution mechanic
when composed through the ontology.

## Consumers

- `simulation` — hosts `SimulationRng` resource, fires observer events
- `rules_engine` — future: combat resolution using `ResolutionTable` + `roll_die`
- `editor_ui` — future: roll display panel, table editor UI
- `persistence` — future: save/load resolution tables and lookup tables

## Producers

- `simulation` — inserts `SimulationRng` resource at startup

## Types

### Dice Pool

```rust
/// A pool of dice to roll together: count dice of sides sides, plus a flat modifier.
pub struct DicePool {
    pub count: u8,
    pub sides: u8,
    pub modifier: i8,
}

/// The result of rolling a DicePool.
pub struct DiceRoll {
    pub pool: DicePool,
    pub values: Vec<u8>,
    pub total: i16,
}
```

### Seeded RNG

```rust
/// The type of die being rolled.
pub enum DieType {
    D6,
    D10,
    D100,
    Custom { sides: u32 },
}

/// A single recorded die roll.
pub struct RollRecord {
    pub roll_index: u64,
    pub die_type: DieType,
    pub result: u32,
    pub context: String,
}

/// Deterministic RNG resource wrapping ChaCha8Rng.
pub struct SimulationRng {
    seed: u64,
    rng: ChaCha8Rng,
    roll_log: Vec<RollRecord>,
    next_roll_index: u64,
}
```

### Table Resolution

```rust
/// How a resolution table column input is calculated.
pub enum ColumnType {
    Ratio,
    Differential,
    Direct,
}

/// A column header in a resolution table.
pub struct TableColumn {
    pub label: String,
    pub column_type: ColumnType,
    pub threshold: f64,
}

/// A row header in a resolution table (die roll range).
pub struct TableRow {
    pub label: String,
    pub value_min: u32,
    pub value_max: u32,
}

/// The result of a table lookup.
pub enum TableResult {
    Text(String),
    NumericValue(f64),
    PropertyModifier { property: String, delta: f64 },
}

/// A column shift modifier applied during table resolution.
pub struct ColumnModifier {
    pub name: String,
    pub column_shift: i32,
    pub cap: Option<i32>,
    pub priority: u32,
}

/// A 2D resolution table: columns x rows -> outcomes.
pub struct ResolutionTable {
    pub id: TypeId,
    pub name: String,
    pub columns: Vec<TableColumn>,
    pub rows: Vec<TableRow>,
    pub outcomes: Vec<Vec<TableResult>>,
}

/// A 1D lookup table: input threshold -> result.
pub struct LookupTable {
    pub id: TypeId,
    pub name: String,
    pub entries: Vec<LookupEntry>,
}

/// A single entry in a lookup table.
pub struct LookupEntry {
    pub label: String,
    pub threshold: f64,
    pub result: TableResult,
}

/// Result of a full 2D table resolution.
pub struct TableResolution {
    pub column_index: usize,
    pub row_index: usize,
    pub column_label: String,
    pub row_label: String,
    pub result: TableResult,
}
```

## Functions

### RNG

```rust
/// Roll a die, logging the result. Returns [1, die.sides()].
pub fn roll_die(rng: &mut SimulationRng, die: DieType, context: &str) -> u32;

/// Roll a value in [min, max], logging the result.
pub fn roll_range(rng: &mut SimulationRng, min: u32, max: u32, context: &str) -> u32;

/// Reset the RNG with a new seed, clearing the roll log.
pub fn reset_rng(rng: &mut SimulationRng, seed: u64);

/// Roll a DicePool, logging each individual die. Returns a DiceRoll with values and total.
pub fn roll_pool(rng: &mut SimulationRng, pool: DicePool, context: &str) -> DiceRoll;

/// Replay rolls from a seed — returns the first `count` d6 results.
pub fn replay_from_seed(seed: u64, count: u64) -> Vec<u32>;
```

### Table Resolution

```rust
/// Resolve a 1D lookup: rightmost entry whose threshold the input meets.
pub fn resolve_lookup(table: &LookupTable, input: f64) -> Option<usize>;

/// Find the best matching column for given inputs.
pub fn find_table_column(input_a: f64, input_b: f64, columns: &[TableColumn]) -> Option<usize>;

/// Find the row matching a die roll value.
pub fn find_table_row(roll: u32, rows: &[TableRow]) -> Option<usize>;

/// Resolve a full 2D table lookup.
pub fn resolve_table(
    table: &ResolutionTable, input_a: f64, input_b: f64, roll: u32,
) -> Option<TableResolution>;

/// Evaluate column modifiers in priority order. Returns final shift and display list.
pub fn evaluate_column_modifiers(
    modifiers: &[ColumnModifier], column_count: usize,
) -> (i32, Vec<(String, i32)>);

/// Apply a column shift to a base index, clamping to bounds.
pub fn apply_column_shift(base_column: usize, shift: i32, column_count: usize) -> usize;
```

## Source

`crates/hexorder-contracts/src/simulation.rs`
