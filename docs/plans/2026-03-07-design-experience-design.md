# Design Experience — Design Document

**Date**: 2026-03-07 **Status**: Approved **Research**: [[Game Design Process Phases and Tool UX]],
[[Design Tool Interface Patterns]], [[Game Mechanics Discovery and Design Experimentation]],
[[Hex Wargame Designer Community Analysis]]

## Purpose

Define Hexorder's approach to supporting the full game design process — not just editing artifacts,
but guiding designers through the cognitive phases of design, helping them navigate between
abstraction layers, and building connections between designers and their collaborators.

---

## 1. UX Design Framework

### 1.1 Design Personality: The Laboratory

Hexorder is a **design laboratory** — a place where game designers construct, observe, and refine
game systems through structured experimentation. The laboratory metaphor shapes the user-facing
design language.

| Lab Concept      | Hexorder Equivalent                          | UX Implication                                               |
| ---------------- | -------------------------------------------- | ------------------------------------------------------------ |
| **Workbench**    | Design surface                               | Where you construct — hands-on, precise, tool-rich           |
| **Test chamber** | Simulation surface                           | Where you observe — controlled, measurable, repeatable       |
| **Notebook**     | Design journal                               | Where you record — timestamped, searchable, always available |
| **Instruments**  | Panels (lenses, analyzers, dependency views) | Reveal what's not visible to the naked eye                   |
| **Hypothesis**   | Experience goal                              | What you expect to happen — tested against observations      |
| **Experiment**   | Simulation session                           | A controlled run with specific parameters                    |
| **Specimen**     | Game system file                             | The artifact under study — versioned, shareable, complete    |

The laboratory personality is a **design language layer**, not an engineering layer. Engineering
terms (Surface, Panel, Capability) are stable. User-facing labels ("Instrument," "Experiment,"
"Notebook") are provided by the theme system and are pluggable — a different personality plugin
could rename everything without touching the architecture.

### 1.2 Five UX Principles

**1. Focused surfaces, single truth.** Each surface serves one cognitive purpose — designing,
simulating, monitoring, coordinating. Surfaces are opinionated about what they show and how they
behave. All surfaces read from and write to the same underlying data. No sync, no export, no rebuild
between surfaces. Adding a new surface is an architectural pattern, not a special case.

**2. Observe, don't guess.** Every outcome in the simulation should be traceable to its causes.
Every mechanic should show its downstream effects. The tool replaces intuition with visibility —
like instruments in a lab.

**3. Record everything, require nothing.** The journal, experience goals, and annotations are always
available but never mandatory. Zero-friction capture. The designer decides what's worth recording.
No forms, no required fields, no workflow gates.

**4. Structure is stable, skin is pluggable.** Layout rules, interaction patterns, and information
architecture are consistent. Colors, terminology, density, and iconography are themeable and
eventually plugin-extensible. Nothing that affects personality is hardcoded.

**5. Show the phase, don't enforce it.** The designer always knows where they are (exploring,
building, testing) through visual indicators, but the tool never prevents cross-phase actions.
Traffic signs, not toll booths.

### 1.3 Layout Principles

**Design surface** (primary window):

- Follows the universal template: tools left, viewport center, properties right, status bottom
- Dock tabs (panels) are the unit of organization — all features ship as panels
- Workspace presets rearrange panels but never hide capabilities

**Simulation surface** (secondary window):

- Viewport dominant (80%+ of space) — observation-first
- Controls docked bottom or side — minimal, non-obstructing
- Trace/analysis panels expand on demand, collapse by default
- Editing controls disabled by default (Simulate intent lacks Edit capability) — visible but
  inactive so the designer knows they exist and can grant Edit capability temporarily

**Cross-surface coordination**:

- Selection in either surface highlights in both (select a hex in design, it highlights in
  simulation)
- Hovering a mechanic in the design surface can highlight affected hexes/units in the simulation
  surface
- Journal entries created in either surface are visible in both
- All coordination flows through shared Bevy ECS resources and events — no direct surface coupling

### 1.4 Interaction Patterns (Reused Across All Features)

| Pattern              | Description                                                                                                                     | Used By                                                        |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------- |
| **Tag-link**         | Freeform text + optional links to game elements by ID. Links display as clickable chips. Click navigates to the linked element. | Experience goals, journal entries, feedback annotations        |
| **Trace-expand**     | Click an outcome to expand the causal chain that produced it. Each step links to its source mechanic.                           | Causal tracing, dependency view, combat resolution             |
| **Lens-toggle**      | Toggle an analytical overlay on the hex grid. One active lens per surface.                                                      | Movement cost, influence, stacking, probability, goal coverage |
| **Context-annotate** | Right-click any element to add a journal note tagged to it. Pre-populates the tag link.                                         | Contextual feedback, playtest observations                     |
| **Snapshot-compare** | Save current state, make changes, compare before/after side by side.                                                            | Session replay with rule changes, "what if?" branching         |

### 1.5 Engineering Terms vs. Design Language

| Engineering Term | Description                               | Lab Theme Label |
| ---------------- | ----------------------------------------- | --------------- |
| `Surface`        | Logical container with intent and content | "Surface"       |
| `Panel`          | Content unit within a surface             | "Instrument"    |
| `Session`        | A recorded simulation run                 | "Experiment"    |
| `Goal`           | Experience goal                           | "Hypothesis"    |
| `Journal`        | Design journal                            | "Notebook"      |
| `Lens`           | Analytical overlay                        | "Lens"          |
| `Trace`          | Causal chain for an outcome               | "Trace"         |
| `Annotation`     | Feedback note tagged to an element        | "Observation"   |

The mapping is stored in the theme system. Plugins can override any mapping.

---

## 2. Architecture

### 2.1 Surface Model

A **Surface** is Hexorder's unit of focused UI. It is a logical concept independent of its physical
rendering target.

```
Surface (logical)
    id: SurfaceId
    intent: SurfaceIntent          — declares purpose, shapes defaults
    capabilities: CapabilitySet    — what operations are permitted (enforced)
    panels: Vec<Panel>             — the content units it contains
    state: SurfaceState            — active, suspended, transferred
    rendering: RenderingTarget     — where it's currently displayed

Panel (logical)
    id: PanelId
    kind: PanelKind                — viewport, inspector, log, control, etc.
    parent: SurfaceId              — which surface owns this
    content_key: ContentKey        — what data/view this renders

SurfaceIntent (enum, non-exhaustive)
    Design          — construct and edit game systems
    Simulation      — observe and interact with running game state
    Analysis        — statistical evaluation, lens views, reports
    Coordination    — agent/process monitoring (future)
    Custom(String)  — plugin-defined (future)

RenderingTarget (enum)
    Window(Entity)                  — own OS window
    DockTab(SurfaceId, TabId)       — tab within another surface
    SplitPane(SurfaceId, PaneId)    — pane within another surface
    Detached                        — floating, not yet placed
    Remote(DeviceId)                — another process/device (future)
    Suspended                       — exists logically, not rendered
```

**Key constraints**:

1. **No code assumes Surface = Window.** Systems receive a `SurfaceId` and query the
   `RenderingTarget` separately.
2. **Panels are the unit of content.** What we currently call "dock tabs" become Panels. A Panel can
   live inside any Surface regardless of that Surface's RenderingTarget.
3. **Promotion and demotion are target changes, not content changes.** "Break this panel out into a
   window" = create a new Surface with `RenderingTarget::Window`, move the Panel into it. Content
   and state unchanged.
4. **Intent governs defaults, capabilities govern enforcement.** Intent shapes which panels appear
   and what the phase indicator says. Capabilities determine what operations are permitted.
5. **All surfaces read from the shared World.** No surface owns data. The Bevy ECS World is the
   single truth.

### 2.2 Capability Model

Capabilities communicate what work a surface supports. The system enforces boundaries — a panel that
tries to execute an operation its surface lacks the capability for is denied.

```
Capability (enum, non-exhaustive for plugin extension)
    Observe     — view board state, inspect properties, read data
    Edit        — modify entity types, CRT, terrain, ontology, properties
    Simulate    — advance phases, resolve combat, roll dice, move units
    Annotate    — create journal entries, experience goals, feedback
    Analyze     — toggle lenses, view dependency graphs, run Monte Carlo
    Record      — capture and replay simulation sessions

CapabilitySet — on each Surface (HashSet<Capability> or bitflags)

Surface::has_capability(Capability) -> bool
Surface::grant(Capability)
Surface::revoke(Capability)
```

**Capabilities are dynamic** — grantable and revocable at runtime. Each surface starts with a
default set derived from its intent, but the designer can modify it.

**Default sets per intent**:

| Intent       | Default Capabilities                |
| ------------ | ----------------------------------- |
| Design       | Observe, Edit, Annotate, Analyze    |
| Simulation   | Observe, Simulate, Annotate, Record |
| Analysis     | Observe, Analyze, Annotate          |
| Coordination | Observe, Annotate                   |

**UI enforcement**: Controls requiring a capability the surface lacks render as **disabled**
(visible but inactive), not hidden. This preserves discoverability — the designer sees what's
possible and can grant the capability if needed.

**Extensibility path**: Categories can later decompose into granular sub-capabilities (e.g.,
`Edit.EntityType`, `Edit.CRT`, `Edit.Terrain`) without changing the enforcement interface. The check
is always `surface.has_capability(x)` where `x` is a category or a specific sub-capability.

### 2.3 New Contract Types

#### Surface Management

```
SurfaceId               — unique identifier for a surface instance
SurfaceIntent           — enum of surface purposes (non-exhaustive)
Capability              — enum of operation categories (non-exhaustive)
CapabilitySet           — set of capabilities on a surface
SurfaceState            — Active | Suspended
RenderingTarget         — where the surface is displayed
Surface                 — the logical surface definition
SurfaceRegistry         — resource tracking all open surfaces
PanelKind               — enum of panel types (extends current DockTab)
Panel                   — logical panel definition

Event: SurfaceOpenedEvent   — fired when a surface is created/activated
Event: SurfaceClosedEvent   — fired when a surface is closed/suspended
Event: CrossSurfaceHighlightEvent — one surface requests highlighting in others
```

#### Experience Goals

```
ExperienceGoal          — { id: TypeId, text: String, tags: Vec<TypeId> }
GoalRegistry            — resource: Vec<ExperienceGoal>
```

#### Design Journal

```
JournalEntry            — { id: TypeId, timestamp: DateTime, text: String,
                            tags: Vec<TypeId>, phase_hint: Option<Phase>,
                            surface_intent: Option<SurfaceIntent> }
JournalRegistry         — resource: Vec<JournalEntry>
Phase                   — enum: Explore | Build | Test
```

#### Feedback Annotations

```
Annotation              — { id: TypeId, text: String, target: AnnotationTarget,
                            turn_number: Option<u32>, phase_index: Option<usize> }
AnnotationTarget        — enum: Hex(HexPosition) | Unit(TypeId, HexPosition) |
                            Mechanic(TypeId) | General
AnnotationRegistry      — resource: Vec<Annotation>
```

#### Lens System

```
LensMode                — enum: Off | MovementCost | Influence | Stacking |
                            Probability | GoalCoverage | Custom(String)
ActiveLens              — resource per surface: LensMode
```

#### Session Recording

```
SessionAction           — enum: Move { entity, from, to } |
                            Combat { attacker, defender } |
                            PhaseAdvance | DiceRoll { pool, result } | ...
SessionRecord           — { id: TypeId, seed: u64,
                            actions: Vec<(Duration, SessionAction)> }
SessionRecordingState   — { is_recording: bool, current: Option<SessionRecord> }
```

#### Causal Tracing

```
TraceStep               — { description: String, source_id: Option<TypeId>,
                            input: String, output: String }
TraceEntry              — { outcome: String, steps: Vec<TraceStep> }
ActiveTrace             — resource: Option<TraceEntry>
```

#### Dependency Graph

```
DependencyEdge          — { from: TypeId, to: TypeId, relationship: String }
DependencyGraph         — computed: Vec<DependencyEdge> (not persisted)
```

### 2.4 Persistence Changes

`GameSystemFile` gains new fields. Format version bumps to **7**.

```rust
// New fields in GameSystemFile
pub experience_goals: GoalRegistry,
pub journal: JournalRegistry,
pub annotations: AnnotationRegistry,
pub session_records: Vec<SessionRecord>,  // optional, can be large
```

### 2.5 Cross-Surface Coordination

All surfaces share the Bevy `World`. Coordination happens through:

- **Shared resources**: `SelectedHex`, `SelectedUnit`, `ActiveLens`, `TurnState` — read by all
  surfaces
- **Events**: `HexSelectedEvent`, `CombatResolvedEvent`, etc. — observed by all surfaces
- **Cross-highlight event**: `CrossSurfaceHighlightEvent { target: AnnotationTarget }` — one surface
  requests highlighting in all others
- **No direct coupling**: surfaces never reference each other. They communicate through the World.

### 2.6 AppScreen Evolution

`AppScreen::Play` is **deprecated and removed**. The simulation surface replaces it.

```
AppScreen:
    Launcher  — project selection (unchanged)
    Editor    — surfaces are active (replaces both Editor and Play)
```

The simulation surface opens/closes dynamically within the Editor state. No screen transition needed
to start simulating.

### 2.7 Migration Path

| Current Concept       | Becomes                               | Notes                              |
| --------------------- | ------------------------------------- | ---------------------------------- |
| `AppScreen::Editor`   | Surface with Design intent            | Primary window                     |
| `AppScreen::Play`     | Surface with Simulation intent        | Secondary window, on demand        |
| `AppScreen::Launcher` | Stays as-is                           | Pre-surface state                  |
| `DockTab` enum        | `PanelKind` enum                      | Same variants plus new ones        |
| `DockState<DockTab>`  | `DockState<PanelKind>` per Surface    | Each surface has its own dock      |
| `WorkspacePreset`     | Preset per SurfaceIntent              | Design presets, Simulation presets |
| `ViewportRect`        | Per-surface viewport tracking         | Multiple viewports                 |
| `EditorState`         | Split into per-intent state resources | Design state, Simulation state     |

### 2.8 Multi-Window Implementation

Each surface with `RenderingTarget::Window` maps to:

- A Bevy `Window` entity (spawned via `commands.spawn(Window { ... })`)
- A `Camera3d` with `RenderTarget::Window(WindowRef::Entity(window_id))`
- An egui context via `EguiMultipassSchedule` with a per-surface schedule label
- An independent `DockState<PanelKind>` rendered in that egui context

The primary (Design) surface uses `EguiPrimaryContextPass`. Each additional surface uses a custom
schedule label (e.g., `SimulationContextPass`).

Shared state flows through normal Bevy `Resource` types — no custom sync.

Closing a secondary window fires `SurfaceClosedEvent` and transitions the surface to
`SurfaceState::Suspended`. The primary window close triggers the existing `CloseProjectEvent`.

---

## 3. Feature Design

### Feature 1: Dual-Window Surface Architecture

**What ships**: The Surface/Panel/RenderingTarget/Capability abstraction. Two surfaces — Design
(primary window, Intent::Design) and Simulation (secondary window, Intent::Simulation).

**Design surface** retains the existing editor dock with all current panels plus new panels
(ExperienceGoals, Journal). Default capabilities: Observe, Edit, Annotate, Analyze.

**Simulation surface** opens via "Open Simulation" button or keyboard shortcut. Contains:

- 3D hex viewport panel (same scene, different camera, read-only by default)
- Simulation Controls panel — turn/phase stepping, dice rolls, combat initiation
- Combat Log panel — rolling log of simulation events with trace-expand links
- Trace panel — causal trace, populated on "Why?" click
- Feedback panel — annotations filtered to current simulation state

Default capabilities: Observe, Simulate, Annotate, Record.

**Migration**: The existing `render_play.rs` panel system (turn controls, combat UI) moves into
simulation surface panels. `AppScreen::Play` is removed. Play-mode-gated systems become
simulation-surface-gated systems (run when a Simulation-intent surface exists).

### Feature 2: Simulation Integration (Hot-Reload Play-in-Editor)

**What ships**: The simulation engine wired into the simulation surface. Dice roll, CRT resolves,
combat produces outcomes, phases advance with effects. Changes in the design surface apply
immediately to the next simulation action — no restart.

**SimulationEngine system set**:

- Runs only when a Simulation-intent surface exists
- Reads TurnStructure, CRT, modifiers, constraints from the shared World
- Writes to TurnState, ActiveCombat, SimulationRng
- Fires events: DieRolled, TableResolved, CombatResolvedEvent, PhaseAdvancedEvent
- Hot-reload is automatic — systems read current resource values each frame

**Turn/phase controls**: Manual stepping — designer clicks "Advance Phase." No auto-play initially.

**Coordination with cycles 16-18**: The batched cycle sequence (simulation runtime, combat,
scenarios) builds the simulation primitives that this feature wires into the surface. This feature
may follow or run alongside those cycles.

### Feature 3: Experience Goals (Freeform with Tags)

**What ships**: Lightweight text-based system for recording design intent with optional links to
mechanics.

**UI**: New `ExperienceGoals` panel available in any workspace.

- List of goals, each a text block with optional tag-link chips
- "Add Goal" button, inline text editing
- Tag links via searchable dropdown (entity types, relations, constraints, CRT)
- Chips are clickable — navigate to the linked element
- Subtle status indicator: "untested" (no linked journal entries) vs. "tested" (has observations)

**Storage**: `GoalRegistry` in `GameSystemFile`. Persisted in `.hexorder` format v7.

### Feature 4: Design Journal (In-File)

**What ships**: Timestamped running narrative stored in the `.hexorder` file.

**UI**: New `Journal` panel available in any workspace.

- Chronological list of entries, newest first
- Each entry: timestamp, free text, optional tag-link chips, phase hint (auto-detected from active
  workspace preset, editable)
- "Add Entry" — single text field, submit with Enter or Cmd+Enter for multiline
- Entries created via context-annotate (right-click -> "Add Note") pre-populate the tag
- Filter by: phase, tag, date range, search text
- Exportable as Markdown

**Storage**: `JournalRegistry` in `GameSystemFile`. Persisted in `.hexorder` format v7.

### Feature 5: Phase-Aware Workspace Presets

**What ships**: Revised presets reflecting Explore/Build/Test phases. Phase indicator. Theme system
extended for design language mapping.

**Revised presets (Design surface)**:

| Preset          | Phase   | Key Panels                                                       | Shortcut |
| --------------- | ------- | ---------------------------------------------------------------- | -------- |
| **Explore**     | Explore | ExperienceGoals, MechanicReference, Journal, Viewport, Inspector | Cmd+1    |
| **Map & Units** | Build   | Palette, Viewport, Inspector, MapGenerator, Selection            | Cmd+2    |
| **Rules**       | Build   | Design, Rules, Viewport, Inspector, Validation                   | Cmd+3    |
| **Analysis**    | Test    | Journal, Validation, Inspector, Viewport                         | Cmd+4    |

**Phase indicator**: Subtle label in the status area showing current phase (Explore / Build / Test).
Derived from active preset. Not enforced. Styled with brand accent colors.

**All panels remain accessible** in any preset — presets change default layout only.

**Theme system extension**: Engineering term -> user-facing label mapping stored in theme
configuration. Plugins can override any mapping.

### Feature 6: Dependency View

**Phase 1 — Contextual list (Inspector integration)**:

When any mechanic is selected, the Inspector shows a **"Connections"** collapsible section:

- **Affects**: items this mechanic influences
- **Affected by**: items that influence this mechanic
- Each entry is clickable — navigates to that element

Dependency data computed by walking: entity types -> concept bindings -> relations -> constraints,
plus CRT -> modifiers -> terrain type filters.

**Phase 2 — Graph visualization**:

New `DependencyGraph` panel. Auto-generated node graph:

- Nodes: entity types, concepts, relations, constraints, CRT
- Edges: dependency relationships with labels
- Click a node to select it (syncs with Inspector)
- Zoom, pan, search
- Force-directed or hierarchical layout

### Feature 7: Contextual Feedback Capture

**What ships**: Right-click any element in either surface to create a journal entry tagged to it.

**In the design surface**: Right-click a hex, unit, entity type, rule -> "Add Note" -> journal entry
created with tag link.

**In the simulation surface**: Right-click a hex, unit, combat log entry, phase marker -> "Add
Observation" -> journal entry created with tag link + current turn/phase state.

**Annotations** are journal entries with simulation context (turn number, phase, board state). They
appear:

- In the Journal panel (filterable)
- In the Inspector's "Notes" section when the tagged element is selected
- In the simulation surface's Feedback panel

### Feature 8: Lens Filters

**What ships**: Togglable analytical overlays on the hex grid in both surfaces.

**Built-in lenses**:

| Lens              | What It Shows                                            | Color Scheme                                           |
| ----------------- | -------------------------------------------------------- | ------------------------------------------------------ |
| **Movement Cost** | Per-hex cost for selected unit type                      | Green (cheap) -> Red (expensive) -> Black (impassable) |
| **Influence/ZOC** | Hexes under zone of control                              | Faction colors with transparency                       |
| **Stacking**      | Unit count vs. stacking limit per hex                    | Green (room) -> Yellow (near) -> Red (full)            |
| **Probability**   | CRT outcome distribution for hexes in combat range       | Gradient with tooltip                                  |
| **Goal Coverage** | Hexes/units touched by mechanics linked to selected goal | Highlighted vs. dimmed                                 |

**Toggle UI**: Lens selector in each surface toolbar. One active lens per surface. Keyboard shortcut
to cycle. Extends the existing grid overlay system.

### Feature 9: Causal Tracing

**What ships**: "Why?" button on simulation outcomes. Trace panel showing the rule chain.

**Trace structure example**:

```
Outcome: DR (Defender Retreats)
  1. Base ratio: 18 vs 6 = 3:1 -> Column 4
  2. Modifier: Defender in Forest -> -1 shift -> Column 3
  3. Modifier: Attacker across river -> -1 shift -> Column 2
  4. Final column: 2:1
  5. Die roll: 4 (d6)
  6. Row 4, Column 2:1 -> DR
```

Each step links to its source mechanic. Clicking navigates to the design surface.

**For movement**: Clicking a blocked hex shows the trace — which constraint failed, which relation
blocked, budget calculation.

**Data sources**: Extends existing `RollRecord` log and `ValidMoveSet` blocked explanations to
produce `TraceEntry` structures.

### Feature 10: Session Recording and Replay

**What ships**: Record simulation actions, replay with modified rules, detect divergence.

**Recording**: Toggle in simulation surface captures all actions:

- Phase advances, unit movements, combat initiations, dice rolls
- Timestamps relative to session start
- Stored with RNG seed as `SessionRecord`

**Replay**:

- Load a session record, simulation replays actions in sequence
- Same seed = same dice rolls (if rules unchanged)
- Modified rules = outcomes may diverge
- Divergence points highlighted with side-by-side comparison
- Replay controls: play, pause, step forward, step back

**Storage**: `Vec<SessionRecord>` in `.hexorder` file. Designer can delete old sessions. "Max
sessions" setting for file size management.

---

## 4. Implementation Phases

Each phase is a buildable vertical slice that can become a pitch at the betting table.

### Phase 1: Surface Foundation

**What ships**: Surface/Panel/RenderingTarget/Capability abstraction. Two surfaces running (Design

- Simulation windows). Simulation surface has a 3D viewport and basic turn/phase controls migrated
  from `AppScreen::Play`. `AppScreen::Play` removed.

**Vertical slice**: Designer opens project, clicks "Open Simulation," second window appears showing
the same hex grid. Advance phases, see board in both windows. Close simulation window returns to
single-window editing.

**Contracts**: Surface, SurfaceId, SurfaceIntent, Capability, CapabilitySet, Panel, PanelKind,
RenderingTarget, SurfaceRegistry, SurfaceOpenedEvent, SurfaceClosedEvent

**Risk**: Multi-window bevy_egui integration, per-surface input handling, macOS Metal behavior.

**Depends on**: Nothing.

### Phase 2: Simulation Integration

**What ships**: Simulation engine wired into the simulation surface. Dice roll, CRT resolves, phases
advance. Hot-reload — edit CRT in design window, next combat uses new values.

**Vertical slice**: Define CRT and units in design window, open simulation, move unit, initiate
combat, see outcome. Change CRT value, initiate another combat — new table applies immediately.

**Contracts**: SimulationEngine system set wiring.

**Risk**: Coordination with cycles 16-18 (simulation runtime, combat, scenarios).

**Depends on**: Phase 1.

### Phase 3: Journal, Goals, and Annotations

**What ships**: Experience goals, design journal, contextual feedback capture. Tag-link interaction
pattern. New panels: ExperienceGoals, Journal. Format version 7.

**Vertical slice**: Write experience goal, tag it to a relation. During simulation, right-click
blocked hex and add observation. Open journal, see both entries with tags, click tag to navigate.

**Contracts**: ExperienceGoal, GoalRegistry, JournalEntry, JournalRegistry, Annotation,
AnnotationTarget, AnnotationRegistry

**Depends on**: Phase 1 (surfaces, new panels). Phase 2 helpful but not blocking.

### Phase 4: Phase-Aware Presets and Theme System

**What ships**: Revised presets (Explore, Map & Units, Rules, Analysis). Phase indicator.
Engineering-to-design-language mapping in theme system.

**Vertical slice**: Switch to Explore preset (Cmd+1), see goals and mechanic reference. Phase
indicator shows "Explore." Switch to Rules (Cmd+3), shows "Build." Simulation surface shows "Test."

**Contracts**: Extended WorkspacePreset with phase association. ThemeLabel mapping.

**Depends on**: Phase 1, Phase 3 (new panels to arrange).

### Phase 5: Dependency View

**What ships**: Inspector "Connections" section (list) and DependencyGraph panel (node graph).

**Vertical slice**: Select terrain type, Inspector shows "Affects: 3 relations, 2 constraints." Open
dependency graph, see full web of connections, click node to select.

**Contracts**: DependencyEdge, DependencyGraph (computed). Graph rendering.

**Depends on**: Phase 1 (panels). Stable ontology.

### Phase 6: Lens Filters

**What ships**: Five built-in lens overlays. Lens selector per surface. Extends grid overlay.

**Vertical slice**: Toggle Movement Cost lens — hexes color by cost. Switch to Influence — ZOC
lights up. In simulation, toggle Probability — combat range hexes show distributions.

**Contracts**: LensMode, ActiveLens per surface.

**Depends on**: Phase 1 (per-surface rendering). Phase 2 (probability lens). Grid overlay system.

### Phase 7: Causal Tracing

**What ships**: "Why?" on outcomes. Trace panel. Links to design surface mechanics.

**Vertical slice**: Combat resolves DR. Click "Why?" — trace shows ratio, modifiers, die roll,
result. Click "forest modifier" — design window navigates to definition.

**Contracts**: TraceEntry, TraceStep, ActiveTrace.

**Depends on**: Phase 2 (simulation producing outcomes).

### Phase 8: Session Recording and Replay

**What ships**: Record/replay toggle. Deterministic replay. Divergence detection. Side-by-side
comparison.

**Vertical slice**: Record 5-turn session. Change CRT. Replay — turn 3 diverges. Tool shows
"Original: DE, New: DR" with trace for each.

**Contracts**: SessionAction, SessionRecord, SessionRecordingState.

**Depends on**: Phase 2, Phase 7 (traces for understanding divergence).

### Phase Dependency Graph

```
Phase 1: Surface Foundation
  |
  +-- Phase 2: Simulation Integration
  |     |
  |     +-- Phase 6: Lens Filters (probability lens)
  |     |
  |     +-- Phase 7: Causal Tracing
  |     |     |
  |     |     +-- Phase 8: Session Recording
  |     |
  |     +-- (coordinates with cycles 16-18)
  |
  +-- Phase 3: Journal, Goals, Annotations
  |     |
  |     +-- Phase 4: Phase-Aware Presets & Theme
  |
  +-- Phase 5: Dependency View
```

Phases 3 and 5 can run in parallel with Phase 2. Phases 6-8 are sequential and depend on the
simulation engine.

---

## 5. Future Directions

These are captured for architectural awareness but are not part of this plan's implementation scope.

### 5.1 Panel Promotion/Demotion

A panel can be promoted from a dock tab to its own window (creating a new Surface with
`RenderingTarget::Window`) or demoted back. The Surface model supports this — it's a
`RenderingTarget` change, not a content change.

### 5.2 Remote Surfaces

A surface transferred to another device — two processes communicating surface state. The surface
definition is serializable; the data stays in the World (or syncs). Connects to agent coordination
research and multi-terminal workflows.

### 5.3 Granular Capabilities

Categories decompose into per-operation sub-capabilities (e.g., `Edit.EntityType`, `Edit.CRT`). The
enforcement interface (`has_capability`) is unchanged.

### 5.4 Plugin-Defined Surfaces

Plugins register new `SurfaceIntent` variants and `PanelKind` variants. A "Performance Monitor"
plugin could add an Intent::Performance surface with custom panels.

### 5.5 Plugin-Defined Lenses

Plugins register custom `LensMode` variants with rendering callbacks. A "Historical Comparison"
plugin could overlay historical engagement data on the hex grid.

### 5.6 Collaborative Surfaces

Multiple designers viewing the same game system simultaneously — each with their own surfaces, all
reading from a shared (potentially networked) World.

---

## 6. Sources

- [[Game Design Process Phases and Tool UX]] — design process models, phase-aware UX, cognitive
  support, connection patterns
- [[Design Tool Interface Patterns]] — layout patterns, progressive disclosure, UX KPIs
- [[Game Mechanics Discovery and Design Experimentation]] — mechanic discovery patterns, tool gap
  analysis, feature priority ordering
- [[Hex Wargame Designer Community Analysis]] — designer workflows, tool landscape, community needs
- [Bevy multiple_windows.rs example](https://raw.githubusercontent.com/bevyengine/bevy/refs/heads/main/examples/window/multiple_windows.rs)
  — multi-window rendering pattern
- [bevy_egui two_windows.rs example](https://github.com/vladbat00/bevy_egui/blob/v0.36.0/examples/two_windows.rs)
  — multi-window egui context pattern
