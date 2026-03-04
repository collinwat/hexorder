# ADR-005: Simulation Primitives Boundary

**Status:** accepted\
**Date:** 2026-03-03

## Context

Hexorder's constitution states it is a "game system design tool, not a consumer game" that should
"help users define rules, develop aesthetics, run experiments, and export game system definitions."
The constitution also states the tool "never provides game mechanics" and uses a space-game test:
"If a feature would make no sense in a space hex game, it is a game mechanic, not a primitive."

During the Traces of Victory canary game analysis, a roadmap was proposed to close 22 gaps between
Hexorder's current capabilities and what a mid-complexity hex wargame requires. Initial gap analysis
produced items like "ZOC projection system," "combat execution runtime," "chit-pull activation
system," and "supply line tracing" — all game-specific mechanics that fail the space-game test.

The tension: the constitution says "run experiments," which requires some simulation capability. A
designer cannot validate their rule set without executing rules against a board state. But building
a wargame-specific combat resolution pipeline violates game neutrality.

## Decision

Hexorder provides **simulation primitives** — generic, game-neutral execution capabilities that
evaluate user-authored rules. These primitives pass the space-game test. The designer composes
primitives with ontology concepts to create game-specific mechanics.

### Primitives (game-neutral simulation capabilities)

| Primitive                    | Description                                                                                                  | Space-game example                                |
| ---------------------------- | ------------------------------------------------------------------------------------------------------------ | ------------------------------------------------- |
| Spatial influence evaluator  | Projects effects from entities into adjacent hexes based on user-defined concepts                            | Radiation field damages adjacent hexes            |
| Table resolution system      | Resolves any user-defined lookup table: compute input → find column → apply modifiers → roll → output result | Scan interference table                           |
| Entity state machine         | Generic state transitions on entities with user-defined states and triggers                                  | Ship hull integrity: full → damaged → destroyed   |
| Phase sequencer              | Executes user-defined phase sequences with action gating per phase                                           | Any turn-based game has phases                    |
| Random draw pool             | Draws from a user-defined pool of items, resolves per-item effects, refills per rules                        | Draw mission cards, random event tokens           |
| Spatial proximity constraint | Entities must be within N hexes of a linked entity to qualify for an action                                  | Ships within range of command vessel              |
| Cascading table lookup       | Chain of table lookups where each result feeds the next                                                      | Solar flare → radiation level → movement modifier |
| Constrained path finding     | Find a path satisfying multiple user-defined conditions                                                      | Emergency jump to hex without enemy presence      |
| Graph reachability evaluator | BFS/DFS from user-defined sources, blocked by user-defined conditions                                        | Fuel pipeline from station to fleet               |
| Accumulation tracker         | Score that changes based on user-defined trigger conditions                                                  | Control points in any game                        |
| Area-effect modifiers        | Markers that modify resolution tables and costs within a radius                                              | Orbital bombardment zone                          |
| Scheduled entity spawning    | Entities enter the grid on user-defined turns at user-defined positions                                      | Fleet reinforcements arrive at jump point         |
| Off-grid entity zones        | Named holding areas outside the hex grid                                                                     | Reserve fleet, scrap yard, mission deck           |

### What primitives must NOT do

- Embed game-specific vocabulary (no "ZOC," "CRT," "chit," "supply" in type names or APIs)
- Hardcode game-specific logic (no "retreat must move away from attacker" — the designer defines the
  path constraints)
- Assume a genre (every primitive must pass the space-game test)

### Simulation mode

Hexorder has two modes:

- **Editor mode** (existing): Author types, rules, board state. Design-time previews like
  `ValidMoveSet` evaluate rules on the current static board state.
- **Simulation mode** (new): Execute user-authored phase sequences. The phase sequencer advances
  through phases, and within each phase, the relevant primitives evaluate. The designer observes
  results and iterates on their rules.

Simulation mode is a design validation tool, not a consumer game runtime. It executes the designer's
rules so they can verify correctness. The exported game system definitions are consumed by a
separate application for actual gameplay.

### The composition pattern

Game-specific mechanics emerge from composing primitives with ontology concepts:

| Game mechanic        | Composed from                                                             |
| -------------------- | ------------------------------------------------------------------------- |
| Zone of Control      | Spatial influence evaluator + movement cost matrix                        |
| Combat resolution    | Table resolution system + entity state machine + constrained path finding |
| Chit-pull activation | Random draw pool + spatial proximity constraint                           |
| Supply lines         | Graph reachability evaluator + entity state machine                       |
| Weather effects      | Cascading table lookup + movement cost matrix                             |
| Reinforcements       | Scheduled entity spawning + off-grid entity zones                         |

The ontology (concepts, relations, constraints) provides the wiring between primitives. The designer
authors the wiring; the tool executes it.

## Consequences

- Every new simulation feature must pass the space-game test before implementation
- Primitive APIs use structural vocabulary (influence, resolution, reachability), not domain
  vocabulary (ZOC, combat, supply)
- The ontology system becomes the primary composition mechanism — primitives are building blocks,
  ontology concepts are the glue
- The canary game (Traces of Victory) validates that primitives compose correctly for one genre;
  future canaries should validate other genres
- The exported game system definition must include the full ontology wiring so the consumer
  application can execute the same compositions
- Simulation mode requires a new `AppScreen::Simulation` state (or `AppScreen::Play` extended) with
  phase sequencer integration
