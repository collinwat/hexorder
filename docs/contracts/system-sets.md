# Contract: system-sets

## Purpose

Shared system set definitions for cross-plugin scheduling. Plugin crates use these sets to declare
ordering constraints without depending on each other. The main binary wires set ordering in
`main.rs`.

## Types

### `HexorderPhase` (SystemSet)

Top-level execution phases within a single frame:

| Variant          | Description                                        |
| ---------------- | -------------------------------------------------- |
| `Input`          | Process user input (keyboard, mouse, UI)           |
| `Simulation`     | Run simulation logic (game system, rules, scripts) |
| `PostSimulation` | Post-simulation bookkeeping (persistence, undo)    |
| `Render`         | Visual updates (camera, mesh sync, materials)      |

Ordering: `Input` → `Simulation` → `PostSimulation` → `Render`

### `SimulationSet` (SystemSet)

Fine-grained sub-sets within `HexorderPhase::Simulation`:

| Variant      | Description                                |
| ------------ | ------------------------------------------ |
| `Grid`       | Hex grid spatial operations                |
| `GameSystem` | Game system definition and entity registry |
| `Ontology`   | Property definitions and categories        |
| `Cell`       | Board position data assignment and sync    |
| `Unit`       | Unit placement, movement, selection        |
| `Rules`      | Rules engine evaluation                    |
| `Scripting`  | Lua scripting execution                    |
| `MapGen`     | Procedural map generation                  |

## Consumers

All extracted plugin crates and in-tree plugins that need cross-plugin ordering.

## Changelog

| Date       | Change             | Reason                                              |
| ---------- | ------------------ | --------------------------------------------------- |
| 2026-03-04 | Initial definition | Wave 0 SDK foundation — crate extraction pitch #192 |
