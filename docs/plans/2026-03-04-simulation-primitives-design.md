# Simulation Primitives: Seeded RNG + Table Resolution

**Date**: 2026-03-04 **Issues**: #199 (Seeded RNG), #200 (Table Resolution System) **Approach**:
Contract-Heavy (Approach A) — types and pure functions in `hexorder-contracts`, thin runtime plugin
in `src/simulation/`

## Decisions

- **Delivery**: Single pitch, new cycle (parallel to Cycle 11 crate extraction)
- **Table generality**: Generalized 2D grid (CRT pattern) + 1D lookup tables
- **RNG history**: Full roll log (persistent, enables future Monte Carlo #57)
- **Plugin home**: New `simulation` plugin in `src/simulation/`
- **Naming**: PascalCase acronyms per existing convention (`Rng`, `Crt`)

## Contract Types (`crates/hexorder-contracts/src/simulation.rs`)

### Seeded RNG

```rust
/// Deterministic RNG resource wrapping ChaCha8Rng.
pub struct SimulationRng {
    seed: u64,
    rng: ChaCha8Rng,
    roll_log: Vec<RollRecord>,
}

pub struct RollRecord {
    pub roll_index: u64,
    pub die_type: DieType,
    pub result: u32,
    pub context: String,
}

pub enum DieType {
    D6,
    D10,
    D100,
    Custom { sides: u32 },
}
```

### RNG Pure Functions

```rust
pub fn roll_die(rng: &mut SimulationRng, die: DieType, context: &str) -> u32
pub fn roll_range(rng: &mut SimulationRng, min: u32, max: u32, context: &str) -> u32
pub fn reset_rng(rng: &mut SimulationRng, seed: u64)
pub fn replay_from_seed(seed: u64, count: u64) -> Vec<u32>
```

### 1D Lookup Table

```rust
pub struct LookupTable {
    pub id: TypeId,
    pub name: String,
    pub entries: Vec<LookupEntry>,
}

pub struct LookupEntry {
    pub label: String,
    pub threshold: f64,
    pub result: TableResult,
}
```

### 2D Resolution Table (Generalized CRT)

```rust
pub struct ResolutionTable {
    pub id: TypeId,
    pub name: String,
    pub columns: Vec<TableColumn>,
    pub rows: Vec<TableRow>,
    pub outcomes: Vec<Vec<TableResult>>,
}

pub struct TableColumn {
    pub label: String,
    pub column_type: ColumnType,
    pub threshold: f64,
}

pub enum ColumnType {
    Ratio,
    Differential,
    Direct,
}

pub struct TableRow {
    pub label: String,
    pub value_min: u32,
    pub value_max: u32,
}

pub enum TableResult {
    Text(String),
    Effect(OutcomeEffect),
    NumericValue(f64),
    PropertyModifier { property: String, delta: f64 },
}
```

### Table Resolution Pure Functions

```rust
pub fn resolve_lookup(table: &LookupTable, input: f64) -> Option<&LookupEntry>
pub fn find_table_column(input_a: f64, input_b: f64, columns: &[TableColumn]) -> Option<usize>
pub fn find_table_row(roll: u32, rows: &[TableRow]) -> Option<usize>
pub fn resolve_table(
    table: &ResolutionTable,
    input_a: f64,
    input_b: f64,
    roll: u32,
) -> Option<TableResolution>
pub fn evaluate_column_modifiers(
    modifiers: &[ColumnModifier],
    column_count: usize,
) -> (i32, Vec<(String, i32)>)

pub struct TableResolution {
    pub column_index: usize,
    pub row_index: usize,
    pub column_label: String,
    pub row_label: String,
    pub result: TableResult,
}
```

## Simulation Plugin (`src/simulation/`)

### Observer Events

```rust
#[derive(Event)]
pub struct TableResolved {
    pub table_id: TypeId,
    pub resolution: TableResolution,
    pub roll: Option<RollRecord>,
}

#[derive(Event)]
pub struct DieRolled {
    pub record: RollRecord,
}
```

### Plugin Registration

```rust
impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SimulationRng::new_random())
           .add_observer(on_die_rolled)
           .add_observer(on_table_resolved);
    }
}
```

Intentionally thin. Contract types and pure functions do the real work. The plugin hosts the
`SimulationRng` resource and emits events for other plugins to react to.

## Build Checklist

1. Contract types — `simulation.rs` in `hexorder-contracts` with all type definitions
2. RNG pure functions — `roll_die`, `roll_range`, `reset_rng`, `replay_from_seed` + tests
3. Table resolution pure functions — 1D lookup + 2D resolution + column modifiers + tests
4. Simulation plugin — `src/simulation/` with resource init, events, plugin registration
5. Integration tests — wire RNG into CRT resolution, verify deterministic replay

## Deferred Items (GitHub Issues during build)

- Migrate `mechanics.rs` CRT types to generic `ResolutionTable` (`type:tech-debt`)
- Roll display UI in editor (`type:feature`, `area:editor-ui`)
- Table editor UI — visual 2D grid editing (`type:feature`, `area:editor-ui`)
- Monte Carlo integration — iterate seeds over resolution tables (#57)

## Crate Dependencies

- `rand` + `rand_chacha` added to `hexorder-contracts/Cargo.toml`

## Conflict Analysis with Cycle 11 (#192)

| Area                      | This pitch                                           | Crate extraction                                      | Conflict          |
| ------------------------- | ---------------------------------------------------- | ----------------------------------------------------- | ----------------- |
| `hexorder-contracts/src/` | Adds `simulation.rs`, one `pub mod` line in `lib.rs` | Adds `system_sets.rs`, one `pub mod` line in `lib.rs` | Trivial merge     |
| `src/simulation/`         | New directory                                        | Not touched                                           | None              |
| `src/main.rs`             | Adds `SimulationPlugin` line                         | Refactors to thin assembly                            | One line, trivial |
| Workspace `Cargo.toml`    | Adds `rand`/`rand_chacha` to contracts               | Centralizes workspace deps                            | May need rebase   |

## Not in Scope

- No UI for roll display or table resolution
- No simulation mode phase execution (#194)
- No CombatSelect tool integration (#107)
- No Monte Carlo analysis (#57)
- No migration of existing CRT types (deferred)
