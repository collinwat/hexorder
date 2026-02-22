# Hexorder Constitution

These principles are NON-NEGOTIABLE. Every agent, every session, every commit must respect them.

## Rust

- Edition 2024, stable toolchain
- `cargo clippy -- -D warnings` must pass (zero warnings)
- `cargo test` must pass before any plugin is marked complete
- No `unsafe` without documented justification in the plugin log
- No `unwrap()` in production code; use `?` or explicit error handling
- All public types: `#[derive(Debug)]` minimum

## Bevy 0.18

- Every plugin is a Bevy `Plugin` implementing `fn build(&self, app: &mut App)`
- Plugins are registered in `main.rs` and nowhere else
- Components are data-only structs (no methods beyond trait impls)
- Logic lives in systems, not in component methods
- Cross-plugin communication uses Events only
- Resources are for global singleton state; components for per-entity state
- Systems must specify their schedule (Startup, Update, FixedUpdate, etc.)
- Never use `World` directly in systems; use `Commands`, `Query`, `Res`, `EventReader`/`EventWriter`

## Project Identity

Hexorder is a **game system design tool**, not a consumer game. Its purpose is to help users define
rules, develop aesthetics, run experiments, and export game system definitions. A separate
application consumes the exported assets for distribution.

The tool is **game-neutral**. It must serve any hex-based game system — historical wargames, sci-fi
skirmishes, fantasy campaigns, abstract strategy — without favoring one genre. The first game system
being designed happens to be historical military, but that is a user choice, not a tool assumption.

## Tool / Game Boundary

The tool provides **primitives** and optional **scaffolding**. It never provides game mechanics.

### Primitives (game-neutral, never polluted with game-specific concepts)

Infrastructure types that any hex-based game system needs. These use neutral, structural vocabulary:

- Hex grid: cells, edges, positions, adjacency
- Spatial properties: elevation, regions, layers
- Entity types: user-defined names and attributes (via `EntityTypeRegistry`)
- Rule authoring: phases, actions, conditions — the grammar for expressing rules
- Serialization: export/import of complete game system definitions

Primitives must **never** embed game-specific terminology (no "river," "road," "infantry,"
"forest"). A primitive named `BiomeEntry` holds a `terrain_name` that resolves against user-defined
types — it does not hardcode what those types are.

### Scaffolding (genre-specific starter content, layered on top)

Templates, presets, and example configurations that help designers get started quickly with a
particular genre. Scaffolding is always:

- **Optional** — the tool works without it
- **Labeled by genre** — e.g., "Historical Wargame Starter," "Sci-Fi Skirmish Template"
- **Separate from primitives** — stored as loadable presets or example projects, never as default
  values in core types
- **User-editable** — scaffolding is a starting point, not a constraint

Examples of scaffolding: a historical wargame biome table (Water, Plains, Forest, Hills, Mountains),
a sci-fi terrain set (Void, Asteroid, Nebula, Station), a set of movement cost rules.

### Game mechanics (user-defined, never hardcoded)

What terrain costs to cross, how combat resolves, what edges mean for movement — these are rules the
designer authors. The tool provides the grammar for expressing them, not the rules themselves.

**Test**: If a feature would make no sense in a space hex game, it is a game mechanic, not a
primitive. It belongs in scaffolding or the user's rule set, not in core infrastructure.

## Simulation & Game Systems

- Hex coordinates use axial (q, r) system (cube coordinates derived)
- The `hexx` crate is the canonical hex math library
- 3D rendering with hex grid on the ground plane
- Turn-based: game logic runs in discrete phases, not continuous
- All simulation entities exist on the hex grid (no off-grid entities except UI/tooling)
- Game systems (rules, units, terrain, phases) must be serializable for export
- The tool must support defining, editing, and experimenting with rule sets at runtime

## Platform

- Primary target: macOS (development platform)
- Additional platforms will be added later
- No client-server split for now; single-user local application

## Documentation

- Markdown filenames are lowercase with hyphens as word separators (e.g., `game-system.md`, not
  `game_system.md`) — enforced by `mise check:filenames`

## Architecture

- Plugin boundaries align with module boundaries
- Shared types live in `src/contracts/` and are specified in `docs/contracts/`
- No circular dependencies between plugins
- Plugins may depend on contracts but never on other plugins' internals
- Plugin load order is declared in `main.rs` and documented in `docs/architecture.md`

## Coordination

- Shape before schedule: promising ideas are shaped into pitches before entering a build cycle
- Spec before code: `docs/plugins/<name>/spec.md` must exist before implementation
- Contracts before types: `docs/contracts/<name>.md` must exist before `src/contracts/<name>.rs`
- Log everything: decisions, test results, blockers go in the plugin log
- One owner per plugin at a time (tracked via issue assignees)
- Circuit breaker: unfinished cycles are cancelled by default, not extended
