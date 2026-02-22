# Plugin Log: map_gen

## Status: speccing

## Decision Log

### 2026-02-21 — Plugin naming and structure

**Context**: Pitch #102 targets hex map generation as a new capability. Need to decide whether this
extends `hex_grid` or becomes a new plugin. **Decision**: New plugin `map_gen` — procedural
generation is a separate concern from grid rendering/selection. **Rationale**: hex_grid owns the
spatial foundation (grid spawning, selection, hover). Map generation is a design tool feature that
operates on the grid. Separation keeps hex_grid focused and allows map_gen to evolve independently.
**Alternatives rejected**: Extending hex_grid (would bloat a foundational plugin with optional
design-tool features).

### 2026-02-21 — Noise library selection

**Context**: Pitch suggests noise-rs as the noise library. Need to confirm before adding dependency.
**Decision**: Use `noise` crate (noise-rs) — well-maintained, supports Perlin/simplex, no unsafe.
**Rationale**: Pitch explicitly recommends it. Supports multiple noise types needed for terrain
generation. Pure Rust with no system dependencies. **Alternatives rejected**: simdnoise (less
maintained), bracket-noise (more game-focused, less flexible).

## Test Results

_No test runs yet._

## Blockers

| Blocker | Waiting On | Raised | Resolved |
| ------- | ---------- | ------ | -------- |
|         |            |        |          |

## Deferred / Future Work

- [None yet]

## Status Updates

| Date       | Status   | Notes                |
| ---------- | -------- | -------------------- |
| 2026-02-21 | speccing | Initial spec created |
