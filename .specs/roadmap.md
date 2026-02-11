# Hexorder Roadmap

## Strategy

This roadmap follows a **vertical-slice, learn-and-adapt** model:

- Only the current milestone is fully specced. Future milestones are loose sketches.
- After each milestone ships, we run a checkpoint: what did we learn? What changed? Then we
  re-sketch what comes next.
- Milestones will be reordered, merged, split, or dropped based on what we discover by using the
  tool.
- No code or specs are written for future milestones until they become current.

**One milestone at a time is in-flight. The rest stay loose until it's their turn.**

---

## Domain Model

These are the core concepts the product is built around. They emerged from early design
conversations and will be refined as we build.

### Game System (versioned)

The abstract design artifact. Defines how the world works: rules, constraints, unit type
definitions, terrain type definitions, combat mechanics, movement rules, turn phase structure,
theme/aesthetics. This is what gets exported. Multiple games can share one system.

### Game (pinned to a Game System version)

A concrete game built on a specific Game System version. Contains map(s), unit rosters, and playable
configurations. Cannot exist without a Game System.

### Scenario / Campaign / Situation

Different ways to experience a Game. Same rules and content, different setups or progressions. These
"skin" or "configure" the Game to provide distinct play experiences.

### Workspace

The user's persistent design-time context. Remembers which Game System or Game the user was working
on, camera state, open panels, and tool state. The user resumes a workspace when they open Hexorder.

### Game Session

Play-test runtime. The game is running, but the user has extra tooling — note-taking, insight
capture, logging — to feed observations back into the design process.

### Change Isolation Model

- Game Systems are immutable at a given version (v1, v2, v3...).
- Games pin to a specific Game System version.
- A Game can fork/duplicate Game System content to experiment with changes in isolation.
- Integration back into the Game System is deliberate, with impact analysis across all consuming
  Games.
- Each Game opts in to upgrading to a new Game System version.

---

## Milestone 1 — "The World Exists"

**Goal**: Open Hexorder, see a hex world, and interact with it. The earliest point where you can
touch the tool and start forming opinions.

**Context**: This milestone operates entirely within a Game System editing context. No Games,
scenarios, workspace persistence, or launcher exist yet. The app boots directly into the sandbox.

### What the user can do

- See a hex grid rendered on the ground plane
- Navigate with a top-down orthographic camera (pan and zoom only, locked perpendicular to the
  ground plane — 2D thinking, 3D code)
- Select hex tiles by clicking
- Paint terrain types onto tiles from a small fixed palette
- See terrain visually differentiated on the grid (color per terrain type)

### Technical scope

| Feature             | Plugin      | Notes                                                                                                                                         |
| ------------------- | ----------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| Hex grid rendering  | `hex_grid`  | Uses `hexx` crate. Renders a hex map of configurable radius on the XZ plane.                                                                  |
| Camera              | `camera`    | Orthographic, top-down, locked to Y-axis. Pan (middle-click drag or WASD) and zoom (scroll). No rotation.                                     |
| Hex selection       | `hex_grid`  | Raycast from cursor to grid, highlight selected tile, fire `HexSelectedEvent`.                                                                |
| Terrain painting    | `terrain`   | Fixed palette of 4-5 terrain types (plains, forest, water, mountain, road). Click to paint. Terrain stored as component on hex tile entities. |
| Terrain visuals     | `terrain`   | Color-coded hex tiles based on terrain type. Simple flat colors or minimal materials.                                                         |
| Editor UI (minimal) | `editor_ui` | bevy_egui panel showing: current tool (select/paint), terrain palette, selected tile info.                                                    |

### Contracts needed

- `hex_grid` (exists as spec, needs implementation)
- `terrain` (new — terrain type enum, terrain component, terrain palette resource)

### Out of scope for M1

- Persistence / save / load
- Game or Game System data model
- Units or entities on the grid
- Rule definitions or constraints
- Undo/redo
- Workspace launcher

### Success criteria

- App launches, hex grid is visible
- Camera pans and zooms smoothly, stays locked top-down
- Clicking a hex tile selects it (visual highlight + event fired)
- Selecting a terrain type and clicking paints that terrain
- Terrain is visually distinguishable on the grid
- `cargo test` and `cargo clippy -- -D warnings` pass

### M1 Checkpoint (2026-02-08)

**Status**: Complete. 44 tests, clippy clean, constitution audit passed.

**1. What did we learn?**

- We needed business and technical audit gates _before_ reaching the end of a milestone. The initial
  build passed all tests but had 5 contract boundary violations that only surfaced in a
  cross-feature audit. Added: milestone completion gate in CLAUDE.md, `SC-BOUNDARY` in the feature
  spec template, pre-checkpoint audit requirement in this roadmap, and a compile-time enforcement
  via private modules + `architecture_tests::feature_modules_are_private` test.
- Searching repeatedly for framework API patterns (Bevy 0.18, bevy_egui 0.39) was a significant time
  cost. Solved by creating `docs/bevy-guide.md` and `docs/bevy-egui-guide.md` as persistent
  references. Future milestones should create guides for any new library before implementation
  begins.

**2. What felt right? What felt wrong or missing?**

- Right: Seeing the tool boot with a hex grid, working buttons, and paintable terrain — even
  bare-bones, it validated the vertical-slice approach.
- Missing: The editor's visual theme is plain/functional. A more engaging color palette and overall
  aesthetic would make the tool more motivating to use during long design sessions. Carry this as a
  desire into M2.

**3. Does M2 still make sense?** Yes.

**4. Reorder/insert/drop?** No changes.

**5. Domain model changes?** None yet.

**Carry-forward notes for M2:**

- Editor visual theme: invest in a cohesive color scheme (background, panel styling, terrain colors)
  early in M2 rather than deferring indefinitely.
- Create library reference guides before implementation (any new crate gets a guide first).

---

## Milestone 2 — "The World Has Properties"

**Goal**: The hex board becomes a Game System artifact. Cells are defined by the user, not
hardcoded. The property system lays the foundation for all future entity definitions.

**Context**: This milestone introduces the Game System container and shifts from hardcoded terrain
types to user-defined cell types with custom properties. The M1 terrain system (hardcoded enum) is
replaced entirely. The editor gets a dark theme and an inspector panel.

### Terminology shift from M1

- "Terrain type" → "Cell type" (a Game System definition describing what a board position is)
- "Terrain painting" → "Cell painting" (applying a cell type to a hex tile)
- Hex tiles on the board are **cells** — their meaning is defined by the Game System

### What the user can do

- Create, edit, and delete cell types (name, color, custom properties)
- Define properties on cell types using 6 data types: Bool, Int, Float, String, Color, Enum
- Paint cell types onto the hex grid (replaces M1 terrain painting)
- Select a cell and inspect/edit its property values in an inspector panel
- Work within a Game System context (id + version displayed, enforced as container)

### Technical scope

| Feature                 | Plugin                      | Notes                                                                                                                                           |
| ----------------------- | --------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| Game System container   | `game_system` (NEW)         | Top-level resource with `id` and `version`. Owns cell type registry.                                                                            |
| Property system         | `game_system`               | PropertyType enum (Bool, Int, Float, String, Color, Enum), PropertyDefinition, PropertyValue. Entity-agnostic — will be reused for units in M3. |
| Cell type definitions   | `game_system`               | CellType (name, color, property defs). CellTypeRegistry resource. Replaces TerrainPalette.                                                      |
| Cell painting + visuals | Refactor `terrain` → `cell` | Painting applies cell types to hex tiles. Visual sync reads cell type color. Replaces hardcoded terrain system.                                 |
| Inspector panel         | `editor_ui` (evolve)        | Right-side panel. Shows selected cell's type and property values. Editable fields per property type.                                            |
| Cell type editor        | `editor_ui` (evolve)        | UI for creating/editing/deleting cell types and their property definitions.                                                                     |
| Editor dark theme       | `editor_ui` (evolve)        | Dark color scheme, system fonts, clear delineation between editor controls and game view.                                                       |

### Contracts needed

- `game_system` (NEW — GameSystem, PropertyType, PropertyDefinition, PropertyValue, EnumDefinition)
- `cell` (NEW — replaces `terrain` — CellTypeId, CellType, CellTypeRegistry, CellData,
  ActiveCellType)
- `editor_ui` (evolve — EditorTool gains new modes if needed)
- `hex_grid` (unchanged)

### M1 types being retired

| M1 Type                        | Replaced By               | Reason                                 |
| ------------------------------ | ------------------------- | -------------------------------------- |
| `TerrainType` (hardcoded enum) | `CellTypeId` (dynamic ID) | User-defined, not hardcoded            |
| `Terrain` (component)          | `CellData` (component)    | References cell type + property values |
| `TerrainEntry`                 | `CellType`                | Richer definition with properties      |
| `TerrainPalette`               | `CellTypeRegistry`        | Lives inside GameSystem                |
| `ActiveTerrain`                | `ActiveCellType`          | Same role, new type                    |
| `terrain` plugin               | `cell` plugin             | Renamed to reflect new abstraction     |

### Out of scope for M2

- Persistence / save / load (M5)
- Units or movable entities (M3)
- Rules, constraints, calculated properties (M4)
- Undo/redo
- EntityRef, List, Map, Struct, Formula property types (future milestones)

### Success criteria

- Game System container exists with id and version
- User can create at least one cell type with custom properties
- All 6 property types work: Bool, Int, Float, String, Color, Enum
- Painting applies user-defined cell types to hex tiles
- Inspector panel shows and allows editing of property values
- Editor uses a dark theme with system fonts
- No hardcoded terrain types remain
- `cargo test` and `cargo clippy -- -D warnings` pass
- Constitution audit passes (no contract boundary violations)

### M2 Checkpoint (2026-02-09)

**Status**: Complete. 53 tests, clippy clean, constitution audit passed.

**1. What did we learn?**

- GPU rendering impacts window lifecycle. Bevy's render pipeline causes a white flash on the
  OS-default window surface before the first GPU frame lands. We solved this with a hidden-window
  pattern (start `visible: false`, reveal after 3 frames once the GPU has rendered dark content).
  This is now documented in `docs/bevy-guide.md` Section 19. Future milestones should account for
  GPU pipeline timing when adding new windows or render targets.
- Brand palette enforcement via architecture tests (`editor_ui_colors_match_brand_palette`) catches
  color drift at compile time. This worked well and should be extended to any future UI surfaces.
- Library reference guides (`docs/bevy-guide.md`, `docs/bevy-egui-guide.md`) continue to pay off —
  created at M1 and expanded throughout M2. Any new crate dependency should get a guide before
  implementation.

**2. What felt right? What felt wrong or missing?**

- Missing: A lot of functionality is still absent (expected at this stage).
- Wrong: The editor theme feels off — the dark palette is functional but not yet visually engaging
  for long design sessions.
- Missing: Brand logo is not visible in the application. Would like it showing for brand recognition
  (e.g., in a title bar, about panel, or watermark).

**3. Does M3 still make sense?** Yes — units on the grid is the natural next step.

**4. Reorder/insert/drop?** No changes at this time.

**5. Domain model changes?** Taxonomy models (hierarchical type classification) feel like they'll be
needed eventually, but premature to add now. Note for future consideration — likely relevant when
the number of cell/unit types grows large enough to need categorization.

**6. Revised sketches?** None yet.

**Carry-forward notes for M3:**

- Editor theme: still needs polish. Consider revisiting the visual design when adding the unit
  palette UI.
- Brand logo: find an appropriate place to display the hexorder logo in the application (title bar
  area, splash, or persistent watermark).
- Taxonomy models: keep in mind for M4+ when type counts grow.

---

## Milestone 3 — "Things Live in the World"

**Goal**: Define unit types with stats and properties. Place unit tokens on the hex grid. Basic
movement — click a unit, click a destination, it moves (respecting grid bounds). No rule enforcement
yet, just placement and relocation.

### M3 Checkpoint (2026-02-10)

**Status**: Complete. 71 tests, clippy clean, constitution audit passed (9/9 checks).

**1. What did we learn?**

- bevy_egui text input silently fails without `enable_absorb_bevy_input_system = true`. Run
  conditions on game systems are not sufficient — Bevy's internal systems consume keyboard events
  before egui processes them. Fixed and documented in bevy-egui-guide.md (Section 7 + Pitfall #10)
  and editor_ui log.

**2. What felt right? What felt wrong or missing?**

- The interaction is still entirely panel-based. Creating cell types and unit types in the editor
  doesn't produce immediate visual feedback in the 3D viewport. The user wants to see the things
  they create rendered in the viewing portal — the connection between "I defined something" and "I
  can see it in the world" needs to be more direct.

**3. Does M4 still make sense?** Tentatively yes, but the user's priority is seeing visual rendering
of created types in the viewport before adding rules. May need to reorder.

**4. Reorder/insert/drop?** The user wants to get to rendering of created cells sooner. Cell
painting (M2) and unit placement (M3) already render in the viewport, but the workflow may not be
discoverable enough — or the viewport may need better separation from the panel.

**5. Domain model changes?** None.

**6. Revised sketches?** Not yet.

**Carry-forward notes for M4:**

- Viewport experience: the user needs to see the connection between type creation and world
  rendering. May need viewport adjustment (push 3D view right of panel), visual affordances, or
  workflow guidance.
- Discoverability: Paint mode paints cells, Place mode places units — but this may not be obvious
  without trying it.
- Input absorb pattern: documented and working. Apply to any future text input surfaces.

---

## Milestone 4 — "Rules Shape the World" (sketch)

Introduce constraints and rules. Movement costs based on terrain. Impassable terrain. Turn phases
(move phase, action phase). The simulation enforces rules defined by the user. The user can see "why
can't this unit move there?" feedback.

---

## Milestone 5 — "The World Remembers" (sketch)

Persistence layer. Save and load Game System definitions. Workspace state (camera, open panels,
last-edited context). The launcher screen: resume workspace, load previous, create new. Game System
and Game data model in storage.

---

## Milestone 6+ — Future (sketch)

These are known needs, unordered and unrefined:

- Game Sessions (play-test runtime with insight capture)
- Scenarios, campaigns, situations within a Game
- Game System versioning and change isolation model
- Impact analysis for Game System changes across Games
- Export pipeline for game system definitions
- Combat resolution systems
- Line of sight / visibility
- Rich rule authoring UI
- Undo/redo

---

## Checkpoint Template

After each milestone, answer these before speccing the next:

1. What did we learn by using the tool at this stage?
2. What felt right? What felt wrong or missing?
3. Does the next planned milestone still make sense?
4. Do we need to insert, reorder, or drop anything?
5. Have any domain model assumptions changed?
6. Update this file with revised sketches.

### Pre-Checkpoint: Constitution Audit

Before running the checkpoint, the milestone must pass the **Milestone Completion Gate** defined in
CLAUDE.md. This is a full-codebase audit covering tests, lint, contract boundaries, and
architectural rules. No milestone is complete until the audit passes with zero violations.
