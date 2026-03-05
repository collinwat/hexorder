# Plugin: simulation

## Summary

Hosts simulation primitives: seeded RNG with deterministic replay and generic table resolution. All
data types and pure functions live in `hexorder_contracts::simulation`; this plugin provides the
Bevy resource lifecycle and observer events.

## Plugin

- Module: `src/simulation/`
- Plugin struct: `SimulationPlugin`
- Schedule: none (resource insertion at build, observers only)

## Dependencies

- **Contracts consumed**: simulation (SimulationRng, DieType, RollRecord, ResolutionTable,
  LookupTable, TableResolution, and all resolution functions), game_system (TypeId)
- **Contracts produced**: none (simulation contract is defined in hexorder-contracts)
- **Crate dependencies**: rand 0.9, rand_chacha 0.9 (via hexorder-contracts)

## Requirements

1. [REQ-1] Inserts `SimulationRng` resource (random seed) at plugin build
2. [REQ-2] Registers `DieRolled` observer for die roll notifications
3. [REQ-3] Registers `TableResolved` observer for table resolution notifications
4. [REQ-4] All simulation types and pure functions are domain-agnostic (ADR-005 space-game test)

## Success Criteria

- [x] [SC-1] `simulation_rng_resource_available` test — SimulationRng exists after plugin build
- [x] [SC-2] `die_rolled_event_fires` test — rolling a die increments roll count
- [x] [SC-3] `rng_table_resolution_deterministic` test — same seed produces same table resolution
- [x] [SC-4] `reset_replays_same_sequence` test — reset_rng replays identical roll sequence
- [x] [SC-BUILD] `cargo build` succeeds with this plugin registered
- [x] [SC-CLIPPY] `cargo clippy --all-targets` passes
- [x] [SC-TEST] `cargo test` passes (37 total simulation-related tests: 33 contract + 4 plugin)
- [x] [SC-BOUNDARY] No imports from other plugins' internals

## Constraints

- The plugin does NOT define simulation types — they live in `hexorder_contracts::simulation`
- Observer handlers are stubs for future UI integration (no side effects yet)
- All RNG operations are deterministic given the same seed (ChaCha8Rng)

## Deferred Items

- Migrate CRT types to generic ResolutionTable (#222)
- Roll display UI — show recent die rolls in editor (#223)
- Table editor UI — visual 2D grid editing for resolution tables (#224)

## Open Questions

- None
