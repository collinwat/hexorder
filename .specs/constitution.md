# Hexorder Constitution

These principles are NON-NEGOTIABLE. Every agent, every session, every commit must respect them.

## Rust
- Edition 2024, stable toolchain
- `cargo clippy -- -D warnings` must pass (zero warnings)
- `cargo test` must pass before any feature is marked complete
- No `unsafe` without documented justification in the feature log
- No `unwrap()` in production code; use `?` or explicit error handling
- All public types: `#[derive(Debug)]` minimum

## Bevy 0.18
- Every feature is a `Plugin` implementing `fn build(&self, app: &mut App)`
- Plugins are registered in `main.rs` and nowhere else
- Components are data-only structs (no methods beyond trait impls)
- Logic lives in systems, not in component methods
- Cross-feature communication uses Events only
- Resources are for global singleton state; components for per-entity state
- Systems must specify their schedule (Startup, Update, FixedUpdate, etc.)
- Never use `World` directly in systems; use `Commands`, `Query`, `Res`, `EventReader`/`EventWriter`

## Project Identity
Hexorder is a **game system design tool**, not a consumer game. Its purpose is to help users define rules, develop aesthetics, run experiments, and export game system definitions. A separate application consumes the exported assets for distribution.

## Simulation & Game Systems
- Hex coordinates use axial (q, r) system (cube coordinates derived)
- The `hexx` crate is the canonical hex math library
- 3D rendering with hex grid on the ground plane
- Turn-based: game logic runs in discrete phases, not continuous
- All simulation entities exist on the hex grid (no off-grid entities except UI/tooling)
- Game systems (rules, units, terrain, phases) must be serializable for export
- Historical military setting — unit types, terrain, and mechanics should reflect this
- The tool must support defining, editing, and experimenting with rule sets at runtime

## Platform
- Primary target: macOS (development platform)
- Additional platforms will be added later
- No client-server split for now; single-user local application

## Architecture
- Feature boundaries align with plugin boundaries
- Shared types live in `src/contracts/` and are specified in `.specs/contracts/`
- No circular dependencies between feature plugins
- Features may depend on contracts but never on other features' internals
- Plugin load order is declared in `main.rs` and documented in coordination.md

## Coordination
- Spec before code: `.specs/features/<name>/spec.md` must exist before implementation
- Contracts before types: `.specs/contracts/<name>.md` must exist before `src/contracts/<name>.rs`
- Log everything: decisions, test results, blockers go in the feature log
- One owner per feature at a time (tracked in coordination.md)
