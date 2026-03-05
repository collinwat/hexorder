# Plugin Log: simulation

## 2026-03-04 — 0.16.1 Initial implementation (#216)

**Scope**: Seeded RNG with deterministic replay, generic table resolution (1D lookup + 2D grid),
thin Bevy plugin with observer events.

**Design decisions**:

- **Contract-heavy architecture**: All types and pure functions in `hexorder_contracts::simulation`,
  plugin only hosts the `SimulationRng` resource and observer stubs. This maximizes testability — 33
  of 37 tests run without Bevy.
- **ChaCha8Rng over ChaCha20Rng**: ChaCha8 is faster and sufficient for game simulation (not
  cryptographic). Deterministic replay requires only seed consistency, not cryptographic strength.
- **Generic table resolution (ADR-005)**: `ResolutionTable` uses `ColumnType` (Ratio, Differential,
  Direct) and `TableRow` ranges instead of CRT-specific vocabulary. Passes the space-game test — any
  domain can define tables with these primitives.
- **Observer stubs**: `on_die_rolled` and `on_table_resolved` are empty — they exist so future UI
  plugins can subscribe without changing the simulation plugin.
- **`On<E>` not `Trigger<E>`**: Bevy 0.18 observer systems use `On<E>` as the first parameter, not
  `Trigger<E>`.

### Test results

37 simulation-related tests pass:

- 33 contract tests (RNG: seeding, rolling, range, replay, reset; Table: column types, row lookup,
  2D resolution, modifiers, lookup tables, edge cases)
- 4 plugin tests (resource availability, die roll counting, deterministic resolution, reset replay)

Full suite passes. Zero clippy warnings.

### Deferred

- Migrate CRT types to generic ResolutionTable (#222)
- Roll display UI (#223)
- Table editor UI (#224)
