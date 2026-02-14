# Hexorder Roadmap

## Release 0.1.0 — "The World Exists"

**Goal**: Open Hexorder, see a hex world, and interact with it. The earliest point where you can
touch the tool and start forming opinions.

**Context**: This release operates entirely within a Game System editing context. No Games,
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

### Out of scope for 0.1.0

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

### 0.1.0 Checkpoint (2026-02-08)

**Status**: Complete. 44 tests, clippy clean, constitution audit passed.

**1. What did we learn?**

- We needed business and technical audit gates _before_ reaching the end of a release. The initial
  build passed all tests but had 5 contract boundary violations that only surfaced in a
  cross-feature audit. Added: release completion gate in CLAUDE.md, `SC-BOUNDARY` in the feature
  spec template, pre-checkpoint audit requirement in this roadmap, and a compile-time enforcement
  via private modules + `architecture_tests::feature_modules_are_private` test.
- Searching repeatedly for framework API patterns (Bevy 0.18, bevy_egui 0.39) was a significant time
  cost. Solved by creating `docs/guides/bevy-guide.md` and `docs/guides/bevy-egui-guide.md` as
  persistent references. Future releases should create guides for any new library before
  implementation begins.

**2. What felt right? What felt wrong or missing?**

- Right: Seeing the tool boot with a hex grid, working buttons, and paintable terrain — even
  bare-bones, it validated the vertical-slice approach.
- Missing: The editor's visual theme is plain/functional. A more engaging color palette and overall
  aesthetic would make the tool more motivating to use during long design sessions. Carry this as a
  desire into 0.2.0.

**3. Does 0.2.0 still make sense?** Yes.

**4. Reorder/insert/drop?** No changes.

**5. Domain model changes?** None yet.

**Carry-forward notes for 0.2.0:**

- Editor visual theme: invest in a cohesive color scheme (background, panel styling, terrain colors)
  early in 0.2.0 rather than deferring indefinitely.
- Create library reference guides before implementation (any new crate gets a guide first).

---

## Release 0.2.0 — "The World Has Properties"

**Goal**: The hex board becomes a Game System artifact. Cells are defined by the user, not
hardcoded. The property system lays the foundation for all future entity definitions.

**Context**: This release introduces the Game System container and shifts from hardcoded terrain
types to user-defined cell types with custom properties. The 0.1.0 terrain system (hardcoded enum)
is replaced entirely. The editor gets a dark theme and an inspector panel.

### Terminology shift from 0.1.0

- "Terrain type" → "Cell type" (a Game System definition describing what a board position is)
- "Terrain painting" → "Cell painting" (applying a cell type to a hex tile)
- Hex tiles on the board are **cells** — their meaning is defined by the Game System

### What the user can do

- Create, edit, and delete cell types (name, color, custom properties)
- Define properties on cell types using 6 data types: Bool, Int, Float, String, Color, Enum
- Paint cell types onto the hex grid (replaces 0.1.0 terrain painting)
- Select a cell and inspect/edit its property values in an inspector panel
- Work within a Game System context (id + version displayed, enforced as container)

### Technical scope

| Feature                 | Plugin                      | Notes                                                                                                                                              |
| ----------------------- | --------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| Game System container   | `game_system` (NEW)         | Top-level resource with `id` and `version`. Owns cell type registry.                                                                               |
| Property system         | `game_system`               | PropertyType enum (Bool, Int, Float, String, Color, Enum), PropertyDefinition, PropertyValue. Entity-agnostic — will be reused for units in 0.3.0. |
| Cell type definitions   | `game_system`               | CellType (name, color, property defs). CellTypeRegistry resource. Replaces TerrainPalette.                                                         |
| Cell painting + visuals | Refactor `terrain` → `cell` | Painting applies cell types to hex tiles. Visual sync reads cell type color. Replaces hardcoded terrain system.                                    |
| Inspector panel         | `editor_ui` (evolve)        | Right-side panel. Shows selected cell's type and property values. Editable fields per property type.                                               |
| Cell type editor        | `editor_ui` (evolve)        | UI for creating/editing/deleting cell types and their property definitions.                                                                        |
| Editor dark theme       | `editor_ui` (evolve)        | Dark color scheme, system fonts, clear delineation between editor controls and game view.                                                          |

### Contracts needed

- `game_system` (NEW — GameSystem, PropertyType, PropertyDefinition, PropertyValue, EnumDefinition)
- `cell` (NEW — replaces `terrain` — CellTypeId, CellType, CellTypeRegistry, CellData,
  ActiveCellType)
- `editor_ui` (evolve — EditorTool gains new modes if needed)
- `hex_grid` (unchanged)

### 0.1.0 types being retired

| 0.1.0 Type                     | Replaced By               | Reason                                 |
| ------------------------------ | ------------------------- | -------------------------------------- |
| `TerrainType` (hardcoded enum) | `CellTypeId` (dynamic ID) | User-defined, not hardcoded            |
| `Terrain` (component)          | `CellData` (component)    | References cell type + property values |
| `TerrainEntry`                 | `CellType`                | Richer definition with properties      |
| `TerrainPalette`               | `CellTypeRegistry`        | Lives inside GameSystem                |
| `ActiveTerrain`                | `ActiveCellType`          | Same role, new type                    |
| `terrain` plugin               | `cell` plugin             | Renamed to reflect new abstraction     |

### Out of scope for 0.2.0

- Persistence / save / load (0.6.0)
- Units or movable entities (0.3.0)
- Rules, constraints, calculated properties (0.4.0)
- Undo/redo
- EntityRef, List, Map, Struct, Formula property types (future releases)

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

### 0.2.0 Checkpoint (2026-02-09)

**Status**: Complete. 53 tests, clippy clean, constitution audit passed.

**1. What did we learn?**

- GPU rendering impacts window lifecycle. Bevy's render pipeline causes a white flash on the
  OS-default window surface before the first GPU frame lands. We solved this with a hidden-window
  pattern (start `visible: false`, reveal after 3 frames once the GPU has rendered dark content).
  This is now documented in `docs/guides/bevy-guide.md` Section 19. Future releases should account
  for GPU pipeline timing when adding new windows or render targets.
- Brand palette enforcement via architecture tests (`editor_ui_colors_match_brand_palette`) catches
  color drift at compile time. This worked well and should be extended to any future UI surfaces.
- Library reference guides (`docs/guides/bevy-guide.md`, `docs/guides/bevy-egui-guide.md`) continue
  to pay off — created at 0.1.0 and expanded throughout 0.2.0. Any new crate dependency should get a
  guide before implementation.

**2. What felt right? What felt wrong or missing?**

- Missing: A lot of functionality is still absent (expected at this stage).
- Wrong: The editor theme feels off — the dark palette is functional but not yet visually engaging
  for long design sessions.
- Missing: Brand logo is not visible in the application. Would like it showing for brand recognition
  (e.g., in a title bar, about panel, or watermark).

**3. Does 0.3.0 still make sense?** Yes — units on the grid is the natural next step.

**4. Reorder/insert/drop?** No changes at this time.

**5. Domain model changes?** Taxonomy models (hierarchical type classification) feel like they'll be
needed eventually, but premature to add now. Note for future consideration — likely relevant when
the number of cell/unit types grows large enough to need categorization.

**6. Revised sketches?** None yet.

**Carry-forward notes for 0.3.0:**

- Editor theme: still needs polish. Consider revisiting the visual design when adding the unit
  palette UI.
- Brand logo: find an appropriate place to display the hexorder logo in the application (title bar
  area, splash, or persistent watermark).
- Taxonomy models: keep in mind for 0.4.0+ when type counts grow.

---

## Release 0.3.0 — "Things Live in the World"

**Goal**: Define unit types with stats and properties. Place unit tokens on the hex grid. Basic
movement — click a unit, click a destination, it moves (respecting grid bounds). No rule enforcement
yet, just placement and relocation.

### 0.3.0 Checkpoint (2026-02-10)

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

**3. Does 0.4.0 still make sense?** Tentatively yes, but the user's priority is seeing visual
rendering of created types in the viewport before adding rules. May need to reorder.

**4. Reorder/insert/drop?** The user wants to get to rendering of created cells sooner. Cell
painting (0.2.0) and unit placement (0.3.0) already render in the viewport, but the workflow may not
be discoverable enough — or the viewport may need better separation from the panel.

**5. Domain model changes?** None.

**6. Revised sketches?** Not yet.

**Carry-forward notes for 0.4.0:**

- Viewport experience: the user needs to see the connection between type creation and world
  rendering. May need viewport adjustment (push 3D view right of panel), visual affordances, or
  workflow guidance.
- Discoverability: Paint mode paints cells, Place mode places units — but this may not be obvious
  without trying it.
- Input absorb pattern: documented and working. Apply to any future text input surfaces.

---

## Release 0.4.0 — "Rules Shape the World"

**Goal**: Transform Hexorder from a placement tool into a game ontology editor. The designer defines
entity types, concepts, relations, and constraints. The tool validates the design and renders
entities according to the defined rules. No hardcoded game terms — everything is designer-defined.

**Context**: 0.3.0 delivered unit placement and free movement with no rules. 0.4.0 introduces the
conceptual framework that lets a game designer express _how_ their entities interact. The designer
creates abstract concepts (e.g., "Motion"), binds entity types to concept roles, defines relations
between those roles, and adds constraints. The tool validates the design for consistency and shows
the implications visually (e.g., highlighting reachable hexes based on movement constraints).

This release also unifies CellType and UnitType into a single EntityType with a designer-assigned
role (BoardPosition or Token). This eliminates code duplication, simplifies the editor, and enables
the relation system to work across all entity categories uniformly.

### Terminology

- **Entity type**: A designer-defined type (replaces CellType and UnitType). Classified by role.
- **Entity role**: How a type participates in the world — BoardPosition (hex tile) or Token (game
  piece).
- **Property**: A named, typed field on an entity type (existing from 0.2.0).
- **Attribute**: The value assigned to a property for a specific entity instance.
- **Concept**: An abstract category that groups related behaviors across entity types (e.g.,
  "Motion", "Defense"). Has named role slots that entity types can bind to.
- **Relation**: How entity types interact through a concept — defines a trigger (enter, exit,
  coexist) and an effect (modify a property, block movement).
- **Constraint**: A validation rule that must hold for the game design to be consistent. Can be
  auto-generated from relations or manually defined.
- **Schema validation**: Checking whether the game system definition itself is internally consistent
  (design-time).
- **State validation**: Checking whether a board state satisfies all constraints (placement/movement
  time).

### What the user can do

- Create entity types with a role (BoardPosition or Token) — unified editor replaces separate
  cell/unit type editors
- Create concepts with named role slots and bind entity types to them
- Define relations between concept roles (trigger + effect)
- Define constraints using a structured expression builder
- See schema validation results — which parts of their game design are inconsistent and why
- Select a unit and see which hexes it can reach based on the defined constraints
- Attempt to move a unit and get feedback when the move is rejected and why
- Inspect constraint details on hovered hexes (why is this blocked?)

### Technical scope

| Feature                | Plugin                   | Notes                                                                                                                                      |
| ---------------------- | ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------ |
| EntityType unification | `game_system` (refactor) | Replace CellType/UnitType with EntityType + EntityRole. Single EntityTypeRegistry. EntityData replaces CellData/UnitData.                  |
| Concept system         | `ontology` (NEW)         | Concepts, ConceptRoles, ConceptBindings. Designer creates concepts and binds entity types to role slots with property mappings.            |
| Relation system        | `ontology` (NEW)         | Relations between concept roles. Triggers: OnEnter, OnExit, WhilePresent. Effects: ModifyProperty (with operation), Block, Allow.          |
| Constraint system      | `ontology` (NEW)         | Constraints with structured expressions. Auto-generated constraints from relations. ConceptRegistry, RelationRegistry, ConstraintRegistry. |
| Schema validation      | `rules_engine` (NEW)     | Validates game system definition consistency. Reports errors with categories and human-readable explanations.                              |
| State validation       | `rules_engine` (NEW)     | Evaluates constraints against board state. Computes valid moves via BFS. Produces ValidMoveSet resource.                                   |
| Move overlay rendering | `hex_grid` (extend)      | Reads ValidMoveSet, renders green/red overlays on reachable/blocked hexes. Separate overlay entities above tiles.                          |
| Unified entity editor  | `editor_ui` (refactor)   | Single type editor with role selector. Replaces separate cell/unit type editors.                                                           |
| Ontology editor panels | `editor_ui` (extend)     | Tabbed layout: Types, Concepts, Relations, Constraints, Validation. Concept binding UI, relation editor, constraint expression builder.    |
| Validation feedback    | `editor_ui` (extend)     | Schema error panel. Inspector shows constraint annotations. Rejection reasons on hover.                                                    |
| Cell migration         | `cell` (refactor)        | Use EntityTypeRegistry (BoardPosition filter) and EntityData instead of CellTypeRegistry and CellData.                                     |
| Unit migration         | `unit` (refactor)        | Use EntityTypeRegistry (Token filter) and EntityData instead of UnitTypeRegistry and UnitData. Movement consults ValidMoveSet.             |

### Contracts needed

- `game_system` (EVOLVE — EntityType, EntityRole, EntityTypeRegistry, EntityData, ActiveBoardType,
  ActiveTokenType replace CellType/UnitType types)
- `ontology` (NEW — Concept, ConceptRole, ConceptBinding, PropertyBinding, Relation,
  RelationTrigger, RelationEffect, Constraint, ConstraintExpr, registries)
- `validation` (NEW — ValidationResult, ValidMoveSet, SchemaValidation, SchemaError)
- `hex_grid` (EXTEND — MoveOverlay, MoveOverlayState)
- `editor_ui` (unchanged)

### Out of scope for 0.4.0

- Persistence / save / load (0.6.0)
- Turn phases / action phases (deferred — no actions exist yet)
- Formula or computed properties
- Multi-select or group operations
- Taxonomy / type classification hierarchies
- Undo/redo
- Path visualization (optimal path highlighting — just valid/invalid for 0.4.0)

### Success criteria

- Entity types are unified: one editor, one registry, role-based filtering works
- Designer can create at least one concept with two role slots
- Designer can bind entity types to concept roles with property mappings
- Designer can create a relation between concept roles
- Designer can create constraints (at least PropertyCompare and PathBudget types)
- Schema validation catches and reports at least: dangling references, role mismatches, missing
  bindings
- Selecting a unit shows reachable hexes as green overlays, blocked hexes as red
- Moving to a blocked hex shows a rejection message explaining why
- Auto-generated constraints appear when creating Subtract relations
- All existing 0.3.0 functionality preserved (painting, placing, moving — now through unified types)
- `cargo test` and `cargo clippy --all-targets` pass
- Constitution audit passes (no contract boundary violations)

### 0.4.0 Checkpoint (2026-02-13)

**Status**: Complete. 92 tests, clippy clean, constitution audit passed (all checks green).

**1. What did we learn?**

- The ontology framework works — concepts, relations, constraints, and schema validation all
  function as designed. The auto-constraint generation for Subtract relations is a clean pattern.
- The OntologyParams SystemParam bundle was necessary to stay under Bevy's 16-parameter limit. This
  is a sign the editor_ui system is accumulating too many dependencies — future releases should
  consider splitting the monolithic `editor_panel_system` into per-tab systems.
- A UI architecture research effort (documented in `docs/research/ui-architecture-survey.md`)
  surveyed how Unity, Autodesk (Maya), Unreal, and Blender build their editor UIs and test
  interactions. Key findings:
    - **egui has no built-in test driver for UI interactions.** Every major design tool platform has
      one. egui_kittest + AccessKit now exists and should be adopted.
    - **Reflection-driven form generation** (Unreal's UPROPERTY pattern) eliminates most manual form
      building. Bevy's `Reflect` system enables this via bevy-inspector-egui patterns.
    - **Embedded scripting** (Lua/Python) is the industry-standard test driver and user automation
      layer for native design tools (Maya, Blender, Houdini).
    - **Dioxus Native (Blitz)** is the most promising long-term migration target — HTML/CSS forms,
      GPU-rendered via wgpu, AccessKit testable, shared lower crates with Bevy. Pre-alpha today.

**2. What felt right? What felt wrong or missing?**

- Right: The ontology editor tabs (Concepts, Relations, Constraints, Validation) organize complexity
  well. The tabbed layout keeps the sidebar manageable.
- Right: Schema validation with immediate visual feedback ("Schema Valid" / error list) gives the
  designer confidence.
- Wrong: **Editor forms are hand-built egui code.** Every new entity type, property, or concept
  requires manual widget construction. This does not scale — future releases will add more data
  types and the editor code will grow linearly.
- Wrong: **No UI interaction tests exist.** All 92 tests exercise logic and ECS systems. None test
  that clicking a button or filling a form actually produces the correct state change through the
  UI. We cannot verify the editor works without manual testing.
- Missing: **Scripting layer.** Game rule definitions are structured data entered through forms.
  Designers will eventually want to script rules, batch-process definitions, and automate
  experiments. A scripting layer (Lua) would also serve as an integration test driver.
- Missing: Persistence is still absent (expected — scoped for 0.6.0).

**3. Does 0.6.0 still make sense?**

Yes, but it should be preceded by a testing and infrastructure release. Persistence adds complexity
(file I/O, serialization, migration) that will be hard to verify without UI interaction tests.
Adding testability _before_ persistence means 0.6.0's save/load workflows can be tested from day
one.

**4. Reorder/insert/drop?**

**Insert Release 0.5.0 — "The World Is Testable"** before 0.6.0. This is infrastructure, not
features — but it directly enables testing every future release's UI.

**5. Domain model changes?**

No changes to the domain model. However, the _implementation architecture_ assumption has evolved:
the editor UI should be driven by type reflection rather than hand-built forms, and a scripting
layer is a product feature (not just tooling).

**6. Revised sketches?**

0.6.0 stays as sketched. 0.5.0 inserted before it. 0.7.0+ gains "Embedded scripting (Lua)" as a
known future need.

**Carry-forward notes for 0.5.0:**

- egui_kittest is the entry point — validate it works with bevy_egui before committing to the rest
- `Reflect` derives should be additive (don't break existing code)
- mlua integration should expose read-only access to registries first, write access later
- The UI architecture research (`docs/research/ui-architecture-survey.md`) documents the full
  strategy and long-term migration path toward Dioxus Native/Blitz

---

## Release 0.5.0 — "The World Is Testable"

**Goal**: Testing infrastructure and embedded scripting. No new user-facing features — this release
makes the existing editor verifiable and reduces the cost of building future UI.

### What was delivered

- egui_kittest for AccessKit-based UI interaction testing (26 UI tests)
- `Reflect` derives on ~43 game system data types
- Editor_ui render function extraction (testable pure functions)
- mlua (LuaJIT) embedded scripting layer with read-only registry access (11 tests)

### 0.5.0 Checkpoint (2026-02-13)

**Status**: Complete. 129 tests, clippy clean, constitution audit passed.

_Note: 0.5.0 shipped as a pure infrastructure release. Checkpoint recorded retroactively during the
0.6.0 checkpoint process._

**Carry-forward notes for 0.6.0:**

- egui_kittest pattern established — use for all future UI test coverage
- Lua scripting is read-only; write access deferred to backlog (#15)
- editor_panel_system splitting deferred to backlog (#22)
- bevy-inspector-egui exploration deferred to backlog (#28)

---

## Release 0.6.0 — "The World Remembers"

**Goal**: Persistence layer. Save and load Game System definitions. Launcher screen for creating and
opening projects. File menu with keyboard shortcuts.

### What was delivered

- Serialize/Deserialize + Clone on all persistent types (registries, HexPosition, PropertyValue)
- RON file I/O with `.hexorder` file extension
- AppScreen state machine (Launcher → Editor)
- PersistencePlugin with save/load systems and rfd native file dialogs
- Keyboard shortcuts (Ctrl+S save, Ctrl+O open, Ctrl+N new)
- Launcher UI with create-new and open-existing workflows
- 10 new tests (3 serde round-trip, 4 file I/O, 3 persistence plugin)

### 0.6.0 Checkpoint (2026-02-14)

**Status**: Complete. 139 tests, clippy clean, constitution audit passed (all automated checks
green).

**1. What did we learn?**

- Need a shortcut customization mechanism — hardcoded shortcuts don't scale.
- Need action confirmation feedback (e.g., visual confirmation that Cmd+S actually saved).
- The menu bar is not styled well — need a ribbon-like experience similar to AutoDesk tools.
- No way to exit the game system editor back to the launcher screen.
- No default save location — the tool asks the user where to save every time.
- No way to set the workspace/project/game system name when starting a new project.

**2. What felt right? What felt wrong or missing?**

- Wrong: Font choice and font size still feel off. Want user-configurable font size but an
  opinionated default UI/form font choice.
- Otherwise, feedback was captured in Q1 above.

**3. Does the next planned release still make sense?**

Not directly. Before defining the next code release, we need a process for curating issues into
value-add buckets and a release cadence mapping those buckets to releases. The loose sketch model
has served well through 0.1.0-0.6.0, but with 39+ backlog items and growing UX feedback, structured
prioritization is needed.

**4. Reorder/insert/drop?**

Insert **Cycle 1 — "The Process Matures"** before Release 0.7.0. This is a process cycle (no code)
to define the curation and release cadence systems.

**5. Domain model changes?**

Introduce **Workspace as an application-level concept** — the design tool's project container (name,
save path, recent files, open/close lifecycle). This is distinct from the domain model Workspace
concept. Tracked as a GitHub Issue.

**6. Revised sketches?**

Cycle 1 inserted. Release 0.7.0 scope deferred until the bucketing process is defined.

**Carry-forward notes for Cycle 1:**

- Create GitHub Issues for all 0.6.0 checkpoint feedback (shortcut customization, action
  confirmation, ribbon menu, return-to-launcher, default save location, project naming, font size
  config, opinionated font choice, Workspace application concept)
- Triage and bucket all 39+ backlog issues using the new process
- The process should produce a clear Release 0.7.0 scope

---

## Cycle 1 — "The Process Matures"

**Type**: Process cycle (no code). **Appetite**: Small Batch (1-2 weeks).

**Problem**: After shipping 6 releases (0.1.0-0.6.0), the project has 48+ raw ideas in GitHub Issues
and growing UX feedback. The loose sketch model served well for bootstrapping, but structured
prioritization is needed. Without a deliberate process for deciding what to build next, the project
risks building low-value work or losing important ideas.

**Solution**: Adopt Shape Up methodology adapted for solo dev + AI agents. Rewrite all workflow
documentation to use Shape Up terminology and process. Define the pitch template, the cool-down
protocol, and the betting table process.

**Deliverables**:

- Rewrite roadmap strategy to Shape Up cycle model
- Rewrite CLAUDE.md workflow sections for Shape Up
- Rewrite git-guide.md to frame branches within cycles
- Update coordination.md with current bets
- Create pitch Issue template (`type:pitch`)
- Update spec and log templates with Shape Up terminology
- Update constitution.md coordination section
- Update glossary with Shape Up terms

**No gos**: No triage of existing 48+ raw idea Issues in this cycle. That happens in the cool-down
after Cycle 1 ships.

---

## Release 0.7.0 (sketch)

Scope determined by the first betting table after Cycle 1 ships. Shaped pitches from the cool-down
retrospective will compete for this release.
