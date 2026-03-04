# Traces of Victory Roadmap Design

**Date**: 2026-03-03\
**Status**: Approved\
**ADR**: 005 (Simulation Primitives Boundary)

## Goal

Build the simulation primitives and authoring capabilities needed to define and simulate Traces of
Victory's mechanics in Hexorder. Validates the tool's core value proposition: if Hexorder can handle
a mid-complexity hex wargame, it can handle the genre.

**Success bar**: Define all ToV rules, units, terrain, and scenarios in Hexorder AND simulate
individual mechanics (run a combat, trace supply, execute a turn). Not a full connected playthrough.

**Approach**: Vertical slices with integrated validation. Each slice delivers one end-to-end
simulatable primitive set. Within each slice, fundamentals (code) and canary validation
(ToV-specific data authoring + testing) are explicitly separated.

**Key constraint (ADR-005)**: All simulation capabilities are generic primitives that pass the
space-game test. No game-specific vocabulary in APIs or type names.

## Current State (v0.15.0)

| Capability                                  | Status                    |
| ------------------------------------------- | ------------------------- |
| Hex grid rendering + interaction            | Working                   |
| Unit placement, selection, movement         | Working                   |
| BFS movement with ontology constraints      | Working                   |
| Move overlay visualization                  | Working                   |
| Entity type registry with properties        | Working                   |
| CRT data + pure resolution functions        | Math complete, no runtime |
| Turn structure data + advance_phase()       | Data defined, dead code   |
| Ontology (concepts, relations, constraints) | Data model complete       |
| Scripting (Lua)                             | Scaffolding only          |

## Slice Decomposition

### Slice 1: Spatial Rules

**Theme**: Movement, adjacency effects, and placement constraints.

#### Fundamentals

| Primitive                   | Description                                                                                              | Foundation              |
| --------------------------- | -------------------------------------------------------------------------------------------------------- | ----------------------- |
| Hex-side data model         | Edge annotations between adjacent hexes. Generic `EdgeFeature` type                                      | New (issue #150 exists) |
| Spatial influence evaluator | Project effects from entities into adjacent hexes via ontology concepts. Plugs into BFS as cost modifier | New                     |
| Stacking constraint         | Per-hex entity limits with type exemptions. Validates at placement + simulation                          | New                     |
| Movement cost matrix        | Costs vary by (terrain type × entity classification). Extends BFS budget                                 | Extend existing BFS     |

#### Canary Validation

- Define ToV terrain types with TEC movement costs (non-mech and mech columns)
- Define ToV hex-side features (Moselle River, bridges, road network) as edge annotations
- Define ToV stacking (max 2 combat units, HQs exempt) as stacking constraint
- Define ToV ZOC (+2 MP enter/leave) as spatial influence concept
- **Test**: Infantry (4 MP) through forest (2 MP) adjacent to enemy influence (+2 MP) = 4 MP spent,
  cannot continue
- **Test**: Mechanized uses road for reduced cost
- **Test**: 3rd combat unit rejected from occupied hex; HQ stacks freely

#### Dependencies

- Slice 1 has no dependencies; it extends existing foundations.
- Slices 2 and 3 depend on Slice 1.

---

### Slice 2: Table Resolution and Entity States

**Theme**: Resolving authored tables, state transitions, and post-resolution effects.

#### Fundamentals

| Primitive                    | Description                                                          | Foundation                  |
| ---------------------------- | -------------------------------------------------------------------- | --------------------------- |
| Seeded RNG                   | Deterministic random numbers with replay. Die display                | New                         |
| Table resolution system      | Resolve any lookup table: input → column → modifiers → roll → result | Build on existing CRT math  |
| Entity state machine         | Generic state transitions with user-defined states and triggers      | New                         |
| Constrained path finding     | Find path satisfying multiple user-defined conditions                | New                         |
| Post-resolution movement     | After table resolution, optionally move entities per result          | New                         |
| Per-hex resolution modifiers | Hex properties modify resolution inputs (column shifts, multipliers) | Build on existing modifiers |

#### Canary Validation

- Enter ToV CRT (7 columns: 1:2 through 6:1, d6 rows, outcomes AE/AR/EX/DR/DS/DE/NE)
- Define ToV combat modifiers (terrain defense, river crossing)
- Define fortifications (Metz x3/x4, Siegfried x2, Maginot -1 col) as per-hex modifiers
- Define Combat Intensity as optional secondary table resolution
- Define unit strength states (full → reduced → eliminated) as entity state machine
- **Test**: Table resolution with 3:1 input, roll 4 = DR result
- **Test**: Metz x3 multiplier reduces effective input ratio from 6:1 to 2:1
- **Test**: DR result triggers constrained path finding for defender movement
- **Test**: Attacker moves into vacated position via post-resolution movement
- **Test**: Entity transitions from full → reduced state on step loss result

#### Dependencies

- Depends on Slice 1 (spatial influence for constrained path conditions, hex-side features for
  crossing modifiers)
- Can run in parallel with Slice 3

---

### Slice 3: Phase Sequencing and Activation

**Theme**: Turn structure, randomized activation, and time-based events.

#### Fundamentals

| Primitive                    | Description                                                         | Foundation                |
| ---------------------------- | ------------------------------------------------------------------- | ------------------------- |
| Phase sequencer              | Execute user-defined phase sequence with action gating              | Rework existing dead code |
| Random draw pool             | Draw from user-defined pool, resolve per-item effects, refill rules | New                       |
| Spatial proximity constraint | Entities within N hexes of linked entity qualify for action         | New                       |
| Cascading table lookup       | Chain of lookups: result feeds next as input                        | New                       |
| Scheduled entity spawning    | Entities enter grid on defined turns at defined positions           | New                       |
| Off-grid entity zones        | Named holding areas outside hex grid                                | New                       |

#### Canary Validation

- Define ToV phase sequence: Weather → Chit Selection → Action → Turn End
- Define ToV weather as cascading lookup: d6 → weather state → mud track → cost modifier
- Define chit pool (US/German HQ chits + special chits) as random draw pool
- Define HQ command ranges as spatial proximity constraints
- Define reinforcement schedule as scheduled spawning entries
- Define off-grid zones: chit pool, reserve box, eliminated box
- **Test**: Phase sequencer advances Weather → Chit Selection → Action → Turn End → next turn
- **Test**: Weather cascading lookup: roll 5 with rain modifier = mud, mud increases movement costs
- **Test**: Draw from pool returns random item; pool refills at turn boundary
- **Test**: Only entities within proximity of drawn HQ can act
- **Test**: Entity spawns at designated position on scheduled turn
- **Test**: Entity moves to off-grid zone when eliminated

#### Dependencies

- Depends on Slice 1 (spatial proximity uses hex distance calculations)
- Can run in parallel with Slice 2

---

### Slice 4: Graph Evaluation and Configuration

**Theme**: Reachability analysis, scoring, and scenario management.

#### Fundamentals

| Primitive                         | Description                                                            | Foundation            |
| --------------------------------- | ---------------------------------------------------------------------- | --------------------- |
| Graph reachability evaluator      | BFS/DFS from sources, blocked by conditions. Per-entity status         | New (shares BFS code) |
| Accumulation tracker              | Score changes on trigger conditions (entity at position, state change) | New                   |
| Area-effect modifiers             | Markers modify resolution tables and costs within radius               | New                   |
| Configuration subsets (scenarios) | Named configs: entity subset, grid area, rules, positions, victory     | New                   |

#### Canary Validation

- Define supply as graph reachability from map-edge source hexes, blocked by enemy influence
- Define VP as accumulation: +5 when entity at Metz hex, -1 per step loss
- Define air support as area-effect: ground support = column shift, interdiction = +MP cost
- Define 3 scenarios: Patton's Assault, Patch's Advance, Combined Assault
- **Test**: Cut reachability path → entity marked unreachable → triggers state transition (step
  loss)
- **Test**: Entity occupies objective hex → accumulation increases by defined amount
- **Test**: Area-effect marker within radius → table resolution input modified
- **Test**: Scenario loads correct entity subset and grid area

#### Dependencies

- Depends on Slices 2 and 3 (table resolution for area-effect modifiers, phase sequencer for
  per-phase supply evaluation, entity state machine for out-of-supply effects)

---

### Slice 5: Presentation

**Theme**: Visual representation and organizational views.

#### Fundamentals

| Primitive               | Description                                                                 | Foundation              |
| ----------------------- | --------------------------------------------------------------------------- | ----------------------- |
| Symbol rendering system | Configurable symbol libraries for entity display. NATO APP-6 as one library | New (pitch #101 exists) |
| Hierarchy tree view     | Visualize parent-child entity relationships                                 | New (pitch #59 exists)  |
| Off-grid visual layouts | Visual arrangement of off-grid zones per faction                            | Extends Slice 3 zones   |

#### Canary Validation

- Render ToV units with NATO symbols (infantry cross, armor oval, artillery dot)
- Build ToV formation hierarchy: Army → Corps → Division → Regiment
- Layout ToV player boards: arrival track, eliminated box, reserve box per faction
- **Test**: Entity displays correct symbol based on type and classification
- **Test**: Hierarchy view shows correct parent-child relationships
- **Test**: Off-grid layout matches authored zone arrangement

#### Dependencies

- Can begin after Slice 1 (symbol rendering is independent of simulation)
- Off-grid layouts depend on Slice 3 (off-grid zones)

---

## Dependency Graph

```
Slice 1 (Spatial Rules)
  ├──→ Slice 2 (Table Resolution)  ──┐
  ├──→ Slice 3 (Phase Sequencing)  ──┤
  │                                   ├──→ Slice 4 (Graph Eval & Config)
  └──→ Slice 5 (Presentation) ◄──────┘
```

Slices 2 and 3 run in parallel after Slice 1. Slice 4 follows both. Slice 5 overlaps.

## Issue Mapping

### Existing issues that map to primitives

| Issue | Title                                  | Slice | Primitive                                  |
| ----- | -------------------------------------- | ----- | ------------------------------------------ |
| #150  | Hex-edge contract                      | 1     | Hex-side data model                        |
| #69   | Turn structure definition system       | 3     | Phase sequencer                            |
| #101  | NATO military symbol generator (pitch) | 5     | Symbol rendering                           |
| #59   | Order of Battle management (pitch)     | 5     | Hierarchy tree view                        |
| #97   | Advanced simulation mechanics (pitch)  | 3, 4  | Cascading table lookup, graph reachability |
| #107  | CombatSelect tool mode                 | 2     | Table resolution (UI)                      |
| #57   | Monte Carlo CRT visualization (pitch)  | 2     | Table resolution (analysis)                |

### Existing closed issues (idea captured, not implemented)

| Issue | Title                             | Slice | Primitive              |
| ----- | --------------------------------- | ----- | ---------------------- |
| #6    | Combat resolution systems         | 2     | Table resolution       |
| #70   | Supply and logistics              | 4     | Graph reachability     |
| #72   | Weather and environmental effects | 3     | Cascading table lookup |
| #2    | Scenarios / campaigns             | 4     | Configuration subsets  |
| #104  | Combat execution panel            | 2     | Table resolution (UI)  |

### New issues needed

| Slice | Primitive                    | Proposed Issue Title                                                        |
| ----- | ---------------------------- | --------------------------------------------------------------------------- |
| 1     | Spatial influence evaluator  | Spatial influence evaluator — project entity effects into adjacent hexes    |
| 1     | Stacking constraint          | Stacking constraint — configurable per-hex entity limits                    |
| 1     | Movement cost matrix         | Movement cost matrix — terrain × classification cost tables                 |
| 2     | Seeded RNG                   | Seeded RNG — deterministic random number generation with replay             |
| 2     | Table resolution system      | Table resolution system — generic lookup table execution engine             |
| 2     | Entity state machine         | Entity state machine — user-defined state transitions on entities           |
| 2     | Constrained path finding     | Constrained path finding — find paths satisfying user-defined conditions    |
| 2     | Post-resolution movement     | Post-resolution entity movement — move entities based on resolution results |
| 3     | Random draw pool             | Random draw pool — draw from user-defined item pools                        |
| 3     | Spatial proximity constraint | Spatial proximity constraint — distance-based action eligibility            |
| 3     | Cascading table lookup       | Cascading table lookup — chained resolution with result forwarding          |
| 3     | Scheduled entity spawning    | Scheduled entity spawning — time-based entity arrival                       |
| 3     | Off-grid entity zones        | Off-grid entity zones — named holding areas outside hex grid                |
| 4     | Graph reachability evaluator | Graph reachability evaluator — BFS/DFS from sources with blocking           |
| 4     | Accumulation tracker         | Accumulation tracker — score tracking with trigger conditions               |
| 4     | Area-effect modifiers        | Area-effect modifiers — radius-based resolution and cost modifiers          |
| 4     | Configuration subsets        | Configuration subsets — scenario definitions within a game system           |
| —     | Simulation mode              | Simulation mode — phase-driven rule execution for design validation         |
| —     | Canary: Traces of Victory    | Canary validation — define and simulate ToV mechanics in Hexorder           |

### Issues to close or merge

| Issue                      | Recommendation                                                                 |
| -------------------------- | ------------------------------------------------------------------------------ |
| #6 (Combat resolution)     | Superseded by "Table resolution system" — reopen with new framing or reference |
| #70 (Supply and logistics) | Superseded by "Graph reachability evaluator" — reference in new issue          |
| #72 (Weather effects)      | Superseded by "Cascading table lookup" — reference in new issue                |
| #2 (Scenarios)             | Superseded by "Configuration subsets" — reference in new issue                 |

## Estimated Scope

| Slice                  | Estimated Cycles | Notes                                                       |
| ---------------------- | ---------------- | ----------------------------------------------------------- |
| 1: Spatial Rules       | 1-2              | Hex-side model is the largest piece                         |
| 2: Table Resolution    | 1-2              | CRT math exists; runtime wiring + state machine are new     |
| 3: Phase Sequencing    | 1-2              | All new; chit-pull and cascading lookups are novel          |
| 4: Graph Eval & Config | 1                | Algorithms are straightforward; scenarios are data modeling |
| 5: Presentation        | 1                | NATO rendering is the main effort; pitches already shaped   |
| **Total**              | **5-8 cycles**   | Slices 2+3 parallel reduces wall clock to 4-6               |

## Key Risks

1. **Ontology as composition glue**: The plan assumes the ontology system can wire primitives
   together. If concept/relation/constraint expressiveness is insufficient, the ontology needs
   extension before primitives can compose into game mechanics.

2. **Property bags vs structured types**: ToV units need attack/defense/MP/strength stats. The
   current property bag system can store these, but simulation primitives may need typed access (not
   string-keyed lookups). May need a "well-known property" concept.

3. **Simulation mode architecture**: A new AppScreen state with phase sequencer integration is a
   cross-cutting change. Needs careful design before Slice 3.

4. **Export completeness**: The exported game system must include ontology wiring and primitive
   configurations so the consumer app can execute the same mechanics. Export format may need
   significant extension.
