# Design Experience — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan
> task-by-task.

**Goal:** Implement the Surface/Panel/Capability architecture and 10 design experience features
described in `docs/plans/2026-03-07-design-experience-design.md`.

**Architecture:** Eight vertical-slice phases. Phase 1 (Surface Foundation) is fully decomposed into
TDD tasks. Phases 2-8 are decomposed into task groups with file paths and key code — they will be
expanded into full TDD tasks when their cycle begins at the betting table.

**Tech Stack:** Rust (edition 2024), Bevy 0.18, bevy_egui 0.39, egui_dock, serde/RON persistence

**Design document:** `docs/plans/2026-03-07-design-experience-design.md`

---

## Phase 1: Surface Foundation

**Pitch scope:** Surface/Panel/RenderingTarget/Capability contracts. Two OS windows (Design +
Simulation). Migrate `AppScreen::Play` into Simulation surface. Cross-surface selection
highlighting.

**Vertical slice:** Designer opens project, clicks "Open Simulation," second window appears showing
the same hex grid. Close simulation window returns to single-window editing.

---

### Task 1.1: Surface Contract Types

**Files:**

- Create: `crates/hexorder-contracts/src/surface.rs`
- Modify: `crates/hexorder-contracts/src/lib.rs`
- Create: `docs/contracts/surface.md`

**Step 1: Write the failing test — SurfaceIntent, Capability, CapabilitySet**

```rust
// crates/hexorder-contracts/src/surface.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_intent_default_is_design() {
        assert_eq!(SurfaceIntent::default(), SurfaceIntent::Design);
    }

    #[test]
    fn surface_intent_variants_are_distinct() {
        assert_ne!(SurfaceIntent::Design, SurfaceIntent::Simulation);
        assert_ne!(SurfaceIntent::Simulation, SurfaceIntent::Analysis);
        assert_ne!(SurfaceIntent::Analysis, SurfaceIntent::Coordination);
    }

    #[test]
    fn capability_variants_are_distinct() {
        let caps = [
            Capability::Observe,
            Capability::Edit,
            Capability::Simulate,
            Capability::Annotate,
            Capability::Analyze,
            Capability::Record,
        ];
        for (i, a) in caps.iter().enumerate() {
            for (j, b) in caps.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b);
                }
            }
        }
    }

    #[test]
    fn capability_set_default_for_design() {
        let set = CapabilitySet::for_intent(SurfaceIntent::Design);
        assert!(set.has(Capability::Observe));
        assert!(set.has(Capability::Edit));
        assert!(set.has(Capability::Annotate));
        assert!(set.has(Capability::Analyze));
        assert!(!set.has(Capability::Simulate));
        assert!(!set.has(Capability::Record));
    }

    #[test]
    fn capability_set_default_for_simulation() {
        let set = CapabilitySet::for_intent(SurfaceIntent::Simulation);
        assert!(set.has(Capability::Observe));
        assert!(set.has(Capability::Simulate));
        assert!(set.has(Capability::Annotate));
        assert!(set.has(Capability::Record));
        assert!(!set.has(Capability::Edit));
        assert!(!set.has(Capability::Analyze));
    }

    #[test]
    fn capability_set_grant_and_revoke() {
        let mut set = CapabilitySet::for_intent(SurfaceIntent::Design);
        assert!(!set.has(Capability::Simulate));
        set.grant(Capability::Simulate);
        assert!(set.has(Capability::Simulate));
        set.revoke(Capability::Simulate);
        assert!(!set.has(Capability::Simulate));
    }

    #[test]
    fn capability_set_default_for_analysis() {
        let set = CapabilitySet::for_intent(SurfaceIntent::Analysis);
        assert!(set.has(Capability::Observe));
        assert!(set.has(Capability::Analyze));
        assert!(set.has(Capability::Annotate));
        assert!(!set.has(Capability::Edit));
    }

    #[test]
    fn capability_set_default_for_coordination() {
        let set = CapabilitySet::for_intent(SurfaceIntent::Coordination);
        assert!(set.has(Capability::Observe));
        assert!(set.has(Capability::Annotate));
        assert!(!set.has(Capability::Edit));
        assert!(!set.has(Capability::Simulate));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p hexorder-contracts surface` Expected: FAIL — module `surface` does not exist

**Step 3: Write minimal implementation**

```rust
// crates/hexorder-contracts/src/surface.rs

//! Shared surface types. See `docs/contracts/surface.md`.

use std::collections::HashSet;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Unique identifier for a surface instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub struct SurfaceId(pub u64);

impl SurfaceId {
    /// Create a new unique surface ID.
    #[must_use]
    pub fn new() -> Self {
        Self(rand::random())
    }
}

impl Default for SurfaceId {
    fn default() -> Self {
        Self::new()
    }
}

/// Declares a surface's purpose. Shapes default panels and capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Reflect)]
#[non_exhaustive]
pub enum SurfaceIntent {
    /// Construct and edit game systems.
    #[default]
    Design,
    /// Observe and interact with running game state.
    Simulation,
    /// Statistical evaluation, lens views, reports.
    Analysis,
    /// Agent/process monitoring (future).
    Coordination,
}

/// Operation category that a surface can permit or deny.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
#[non_exhaustive]
pub enum Capability {
    /// View board state, inspect properties, read data.
    Observe,
    /// Modify entity types, CRT, terrain, ontology, properties.
    Edit,
    /// Advance phases, resolve combat, roll dice, move units.
    Simulate,
    /// Create journal entries, experience goals, feedback.
    Annotate,
    /// Toggle lenses, view dependency graphs, run Monte Carlo.
    Analyze,
    /// Capture and replay simulation sessions.
    Record,
}

/// Set of capabilities on a surface. Supports grant/revoke at runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub struct CapabilitySet {
    #[reflect(ignore)]
    caps: HashSet<Capability>,
}

impl Default for CapabilitySet {
    fn default() -> Self {
        Self {
            caps: HashSet::new(),
        }
    }
}

impl CapabilitySet {
    /// Create the default capability set for a given intent.
    #[must_use]
    pub fn for_intent(intent: SurfaceIntent) -> Self {
        let caps = match intent {
            SurfaceIntent::Design => {
                vec![
                    Capability::Observe,
                    Capability::Edit,
                    Capability::Annotate,
                    Capability::Analyze,
                ]
            }
            SurfaceIntent::Simulation => {
                vec![
                    Capability::Observe,
                    Capability::Simulate,
                    Capability::Annotate,
                    Capability::Record,
                ]
            }
            SurfaceIntent::Analysis => {
                vec![
                    Capability::Observe,
                    Capability::Analyze,
                    Capability::Annotate,
                ]
            }
            SurfaceIntent::Coordination => {
                vec![Capability::Observe, Capability::Annotate]
            }
        };
        Self {
            caps: caps.into_iter().collect(),
        }
    }

    /// Check whether this set contains a capability.
    #[must_use]
    pub fn has(&self, cap: Capability) -> bool {
        self.caps.contains(&cap)
    }

    /// Grant a capability to this set.
    pub fn grant(&mut self, cap: Capability) {
        self.caps.insert(cap);
    }

    /// Revoke a capability from this set.
    pub fn revoke(&mut self, cap: Capability) {
        self.caps.remove(&cap);
    }
}

/// Whether the surface is active or suspended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Reflect)]
pub enum SurfaceState {
    #[default]
    Active,
    Suspended,
}

/// Where a surface is currently displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum RenderingTarget {
    /// Own OS window (Entity is the Bevy Window entity).
    Window(Entity),
    /// Exists logically but is not rendered.
    Suspended,
}

impl Default for RenderingTarget {
    fn default() -> Self {
        Self::Suspended
    }
}

/// Fired when a surface is created or activated.
#[derive(Event, Debug, Clone)]
pub struct SurfaceOpenedEvent {
    pub surface_id: SurfaceId,
    pub intent: SurfaceIntent,
}

/// Fired when a surface is closed or suspended.
#[derive(Event, Debug, Clone)]
pub struct SurfaceClosedEvent {
    pub surface_id: SurfaceId,
}

/// One surface requests highlighting an element in all other surfaces.
#[derive(Event, Debug, Clone)]
pub struct CrossSurfaceHighlightEvent {
    pub source_surface: SurfaceId,
    pub target: HighlightTarget,
}

/// What element to highlight across surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HighlightTarget {
    Hex(super::hex_grid::HexPosition),
    Unit(super::game_system::TypeId, super::hex_grid::HexPosition),
}
```

**Step 4: Register the module**

Add to `crates/hexorder-contracts/src/lib.rs`:

```rust
#[allow(dead_code)]
pub mod surface;
```

**Step 5: Run test to verify it passes**

Run: `cargo test -p hexorder-contracts surface` Expected: PASS — all 9 tests green

**Step 6: Write contract spec**

Create `docs/contracts/surface.md` documenting all types (follow existing spec format in
`docs/contracts/editor-ui.md`).

**Step 7: Commit**

```bash
git add crates/hexorder-contracts/src/surface.rs crates/hexorder-contracts/src/lib.rs docs/contracts/surface.md
git commit -m "feat(contracts): add surface, capability, and rendering target types"
```

---

### Task 1.2: SurfaceDefinition and SurfaceRegistry

**Files:**

- Modify: `crates/hexorder-contracts/src/surface.rs`

**Step 1: Write the failing test — SurfaceDefinition and SurfaceRegistry**

```rust
// Add to surface.rs tests

#[test]
fn surface_definition_construction() {
    let def = SurfaceDefinition {
        id: SurfaceId(1),
        intent: SurfaceIntent::Design,
        capabilities: CapabilitySet::for_intent(SurfaceIntent::Design),
        state: SurfaceState::Active,
        rendering: RenderingTarget::Suspended,
    };
    assert_eq!(def.intent, SurfaceIntent::Design);
    assert!(def.capabilities.has(Capability::Edit));
}

#[test]
fn surface_registry_insert_and_get() {
    let mut reg = SurfaceRegistry::default();
    let id = SurfaceId(42);
    let def = SurfaceDefinition {
        id,
        intent: SurfaceIntent::Simulation,
        capabilities: CapabilitySet::for_intent(SurfaceIntent::Simulation),
        state: SurfaceState::Active,
        rendering: RenderingTarget::Suspended,
    };
    reg.insert(def);
    assert!(reg.get(id).is_some());
    assert_eq!(reg.get(id).map(|s| s.intent), Some(SurfaceIntent::Simulation));
}

#[test]
fn surface_registry_remove() {
    let mut reg = SurfaceRegistry::default();
    let id = SurfaceId(42);
    let def = SurfaceDefinition {
        id,
        intent: SurfaceIntent::Design,
        capabilities: CapabilitySet::for_intent(SurfaceIntent::Design),
        state: SurfaceState::Active,
        rendering: RenderingTarget::Suspended,
    };
    reg.insert(def);
    assert!(reg.remove(id).is_some());
    assert!(reg.get(id).is_none());
}

#[test]
fn surface_registry_find_by_intent() {
    let mut reg = SurfaceRegistry::default();
    let d = SurfaceDefinition {
        id: SurfaceId(1),
        intent: SurfaceIntent::Design,
        capabilities: CapabilitySet::for_intent(SurfaceIntent::Design),
        state: SurfaceState::Active,
        rendering: RenderingTarget::Suspended,
    };
    let s = SurfaceDefinition {
        id: SurfaceId(2),
        intent: SurfaceIntent::Simulation,
        capabilities: CapabilitySet::for_intent(SurfaceIntent::Simulation),
        state: SurfaceState::Active,
        rendering: RenderingTarget::Suspended,
    };
    reg.insert(d);
    reg.insert(s);
    let sims: Vec<_> = reg.find_by_intent(SurfaceIntent::Simulation).collect();
    assert_eq!(sims.len(), 1);
    assert_eq!(sims[0].id, SurfaceId(2));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p hexorder-contracts surface` Expected: FAIL — `SurfaceDefinition` and
`SurfaceRegistry` not found

**Step 3: Write minimal implementation**

```rust
// Add to surface.rs

/// A logical surface definition.
#[derive(Debug, Clone, Reflect)]
pub struct SurfaceDefinition {
    pub id: SurfaceId,
    pub intent: SurfaceIntent,
    pub capabilities: CapabilitySet,
    pub state: SurfaceState,
    pub rendering: RenderingTarget,
}

impl SurfaceDefinition {
    /// Check whether this surface has a capability.
    #[must_use]
    pub fn has_capability(&self, cap: Capability) -> bool {
        self.capabilities.has(cap)
    }
}

/// Resource tracking all open surfaces.
#[derive(Resource, Debug, Default)]
pub struct SurfaceRegistry {
    surfaces: Vec<SurfaceDefinition>,
}

impl SurfaceRegistry {
    /// Insert a surface definition.
    pub fn insert(&mut self, def: SurfaceDefinition) {
        self.surfaces.push(def);
    }

    /// Get a surface by ID.
    #[must_use]
    pub fn get(&self, id: SurfaceId) -> Option<&SurfaceDefinition> {
        self.surfaces.iter().find(|s| s.id == id)
    }

    /// Get a mutable reference to a surface by ID.
    #[must_use]
    pub fn get_mut(&mut self, id: SurfaceId) -> Option<&mut SurfaceDefinition> {
        self.surfaces.iter_mut().find(|s| s.id == id)
    }

    /// Remove a surface by ID. Returns the removed definition.
    pub fn remove(&mut self, id: SurfaceId) -> Option<SurfaceDefinition> {
        let pos = self.surfaces.iter().position(|s| s.id == id)?;
        Some(self.surfaces.remove(pos))
    }

    /// Find all surfaces with a given intent.
    pub fn find_by_intent(&self, intent: SurfaceIntent) -> impl Iterator<Item = &SurfaceDefinition> {
        self.surfaces.iter().filter(move |s| s.intent == intent)
    }

    /// Check whether any surface with the given intent is active.
    #[must_use]
    pub fn has_active(&self, intent: SurfaceIntent) -> bool {
        self.surfaces
            .iter()
            .any(|s| s.intent == intent && s.state == SurfaceState::Active)
    }
}
```

**Step 4: Run tests**

Run: `cargo test -p hexorder-contracts surface` Expected: PASS

**Step 5: Commit**

```bash
git add crates/hexorder-contracts/src/surface.rs
git commit -m "feat(contracts): add SurfaceDefinition and SurfaceRegistry"
```

---

### Task 1.3: PanelKind Contract Type

**Files:**

- Modify: `crates/hexorder-contracts/src/surface.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn panel_kind_includes_all_current_dock_tabs() {
    // Verify all current DockTab variants have PanelKind equivalents
    let kinds = [
        PanelKind::Viewport,
        PanelKind::Palette,
        PanelKind::Design,
        PanelKind::Rules,
        PanelKind::Inspector,
        PanelKind::Settings,
        PanelKind::Selection,
        PanelKind::Validation,
        PanelKind::MechanicReference,
        PanelKind::MapGenerator,
        PanelKind::Shortcuts,
    ];
    // Each has a display name
    for kind in &kinds {
        assert!(!format!("{kind}").is_empty());
    }
}

#[test]
fn panel_kind_new_variants_exist() {
    let new_kinds = [
        PanelKind::ExperienceGoals,
        PanelKind::Journal,
        PanelKind::SimulationControls,
        PanelKind::CombatLog,
        PanelKind::Trace,
        PanelKind::Feedback,
    ];
    for kind in &new_kinds {
        assert!(!format!("{kind}").is_empty());
    }
}

#[test]
fn panel_kind_viewport_is_not_closeable() {
    assert!(!PanelKind::Viewport.is_closeable());
}

#[test]
fn panel_kind_others_are_closeable() {
    assert!(PanelKind::Palette.is_closeable());
    assert!(PanelKind::Journal.is_closeable());
    assert!(PanelKind::SimulationControls.is_closeable());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p hexorder-contracts surface::tests::panel_kind` Expected: FAIL

**Step 3: Write implementation**

```rust
// Add to surface.rs

/// Which logical panel occupies a dock tab. Extends the former `DockTab` enum
/// with new panels for journal, goals, simulation controls, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
#[non_exhaustive]
pub enum PanelKind {
    // -- Existing (from DockTab) --
    Viewport,
    Palette,
    Design,
    Rules,
    Inspector,
    Settings,
    Selection,
    Validation,
    MechanicReference,
    MapGenerator,
    Shortcuts,
    // -- New panels --
    ExperienceGoals,
    Journal,
    SimulationControls,
    CombatLog,
    Trace,
    Feedback,
}

impl PanelKind {
    /// Whether this panel can be closed by the user.
    #[must_use]
    pub fn is_closeable(self) -> bool {
        !matches!(self, Self::Viewport)
    }
}

impl std::fmt::Display for PanelKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Viewport => write!(f, "Viewport"),
            Self::Palette => write!(f, "Palette"),
            Self::Design => write!(f, "Design"),
            Self::Rules => write!(f, "Rules"),
            Self::Inspector => write!(f, "Inspector"),
            Self::Settings => write!(f, "Settings"),
            Self::Selection => write!(f, "Selection"),
            Self::Validation => write!(f, "Validation"),
            Self::MechanicReference => write!(f, "Mechanic Reference"),
            Self::MapGenerator => write!(f, "Map Generator"),
            Self::Shortcuts => write!(f, "Shortcuts"),
            Self::ExperienceGoals => write!(f, "Experience Goals"),
            Self::Journal => write!(f, "Journal"),
            Self::SimulationControls => write!(f, "Simulation Controls"),
            Self::CombatLog => write!(f, "Combat Log"),
            Self::Trace => write!(f, "Trace"),
            Self::Feedback => write!(f, "Feedback"),
        }
    }
}
```

**Step 4: Run tests**

Run: `cargo test -p hexorder-contracts surface` Expected: PASS

**Step 5: Commit**

```bash
git add crates/hexorder-contracts/src/surface.rs
git commit -m "feat(contracts): add PanelKind enum with new panel variants"
```

---

### Task 1.4: Migrate DockTab to PanelKind

**Files:**

- Modify: `src/editor_ui/components.rs` — change `DockTab` → `PanelKind`, update `DockLayoutState`
- Modify: `src/editor_ui/systems.rs` — update `render_dock_tab`, `editor_dock_system`, layout
  functions
- Modify: `src/editor_ui/mod.rs` — update imports
- Modify: `src/editor_ui/tests.rs` — update test references

This is a mechanical rename. The `DockTab` enum in `components.rs` becomes a re-export of
`PanelKind` from contracts (for the 11 existing variants). The internal `DockState<DockTab>` becomes
`DockState<PanelKind>`.

**Step 1: Update imports and type aliases**

In `src/editor_ui/components.rs`:

- Remove the `DockTab` enum definition entirely
- Import `PanelKind` from contracts: `use hexorder_contracts::surface::PanelKind;`
- Replace all `DockTab` references with `PanelKind`
- Update `DockLayoutState` to use `DockState<PanelKind>`
- Update `DockLayoutFile` to use `DockState<PanelKind>`
- Update all `create_*_layout()` functions to use `PanelKind::*` variants

**Step 2: Update systems.rs**

In `src/editor_ui/systems.rs`:

- Replace all `DockTab::*` with `PanelKind::*`
- Update `render_dock_tab` signature and match arms
- Update `EditorDockViewer` TabViewer impl

**Step 3: Run all tests**

Run: `cargo test --lib editor_ui` Expected: PASS — all existing tests pass with renamed type

Run: `mise check:clippy` Expected: PASS

**Step 4: Commit**

```bash
git add src/editor_ui/ crates/hexorder-contracts/src/surface.rs
git commit -m "refactor(editor_ui): migrate DockTab to PanelKind contract type"
```

---

### Task 1.5: Register SurfaceRegistry and Create Design Surface on Editor Entry

**Files:**

- Modify: `src/editor_ui/mod.rs` — register resource, add system
- Create: `src/editor_ui/surface.rs` — surface management systems
- Modify: `src/editor_ui/components.rs` — add `ActiveSurfaceId` resource

**Step 1: Write the failing test**

```rust
// src/editor_ui/surface.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;
    use hexorder_contracts::surface::*;

    #[test]
    fn create_design_surface_populates_registry() {
        let mut app = App::new();
        app.insert_resource(SurfaceRegistry::default());
        app.add_systems(Startup, create_design_surface);
        app.update();

        let reg = app.world().resource::<SurfaceRegistry>();
        let designs: Vec<_> = reg.find_by_intent(SurfaceIntent::Design).collect();
        assert_eq!(designs.len(), 1);
        assert!(designs[0].has_capability(Capability::Edit));
        assert_eq!(designs[0].state, SurfaceState::Active);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib editor_ui::surface` Expected: FAIL

**Step 3: Write implementation**

```rust
// src/editor_ui/surface.rs

use bevy::prelude::*;
use hexorder_contracts::surface::*;

/// Resource tracking which surface the primary window belongs to.
#[derive(Resource, Debug)]
pub(crate) struct PrimarySurfaceId(pub(crate) SurfaceId);

/// System that creates the Design surface on editor entry.
/// Runs once on `OnEnter(AppScreen::Editor)`.
pub(super) fn create_design_surface(mut registry: ResMut<SurfaceRegistry>) {
    let id = SurfaceId::new();
    let def = SurfaceDefinition {
        id,
        intent: SurfaceIntent::Design,
        capabilities: CapabilitySet::for_intent(SurfaceIntent::Design),
        state: SurfaceState::Active,
        rendering: RenderingTarget::Suspended, // Will be set to Window after spawn
    };
    registry.insert(def);
    // PrimarySurfaceId inserted separately after window entity is known
}
```

**Step 4: Register in mod.rs**

In `src/editor_ui/mod.rs`:

- Add `mod surface;`
- In plugin `build()`:
    - `app.init_resource::<SurfaceRegistry>()`
    - `app.add_systems(OnEnter(AppScreen::Editor), surface::create_design_surface)`

**Step 5: Run tests**

Run: `cargo test --lib editor_ui::surface` Expected: PASS

**Step 6: Commit**

```bash
git add src/editor_ui/surface.rs src/editor_ui/mod.rs
git commit -m "feat(editor_ui): register SurfaceRegistry, create Design surface on editor entry"
```

---

### Task 1.6: Open Simulation Window (Multi-Window)

**Files:**

- Modify: `src/editor_ui/surface.rs` — add `open_simulation_surface` system
- Modify: `src/editor_ui/mod.rs` — register observer for open event
- Modify: `crates/hexorder-contracts/src/surface.rs` — add `OpenSimulationEvent`

This is the highest-risk task. Reference:

- Bevy `multiple_windows.rs` example (spawns Window entity + Camera3d with RenderTarget)
- bevy_egui `two_windows.rs` example (EguiMultipassSchedule per window)

**Step 1: Add the trigger event**

```rust
// Add to crates/hexorder-contracts/src/surface.rs

/// Request to open a simulation surface.
#[derive(Event, Debug)]
pub struct OpenSimulationEvent;

/// Request to close a simulation surface.
#[derive(Event, Debug)]
pub struct CloseSimulationEvent;
```

**Step 2: Write the system**

```rust
// Add to src/editor_ui/surface.rs

/// Observer: opens a second OS window with a simulation surface.
pub(super) fn open_simulation_surface(
    _trigger: Trigger<OpenSimulationEvent>,
    mut commands: Commands,
    mut registry: ResMut<SurfaceRegistry>,
) {
    // Don't open if one already exists
    if registry.has_active(SurfaceIntent::Simulation) {
        return;
    }

    let id = SurfaceId::new();

    // Spawn a secondary window
    let window_entity = commands
        .spawn(Window {
            title: "Hexorder — Simulation".to_string(),
            resolution: bevy::window::WindowResolution::new(1280.0, 720.0),
            ..default()
        })
        .id();

    // Spawn a camera targeting the new window
    commands.spawn((
        Camera3d::default(),
        Camera {
            target: bevy::render::camera::RenderTarget::Window(
                bevy::window::WindowRef::Entity(window_entity),
            ),
            ..default()
        },
    ));

    let def = SurfaceDefinition {
        id,
        intent: SurfaceIntent::Simulation,
        capabilities: CapabilitySet::for_intent(SurfaceIntent::Simulation),
        state: SurfaceState::Active,
        rendering: RenderingTarget::Window(window_entity),
    };
    registry.insert(def);

    commands.trigger(SurfaceOpenedEvent {
        surface_id: id,
        intent: SurfaceIntent::Simulation,
    });
}

/// Observer: closes the simulation surface and its window.
pub(super) fn close_simulation_surface(
    _trigger: Trigger<CloseSimulationEvent>,
    mut commands: Commands,
    mut registry: ResMut<SurfaceRegistry>,
) {
    let sim_id = registry
        .find_by_intent(SurfaceIntent::Simulation)
        .next()
        .map(|s| (s.id, s.rendering));

    if let Some((id, RenderingTarget::Window(entity))) = sim_id {
        commands.entity(entity).despawn();
        registry.remove(id);
        commands.trigger(SurfaceClosedEvent { surface_id: id });
    }
}
```

**Step 3: Register observers in mod.rs**

```rust
app.add_observer(surface::open_simulation_surface);
app.add_observer(surface::close_simulation_surface);
```

**Step 4: Add "Open Simulation" button to the menu bar**

In `src/editor_ui/systems.rs`, in the menu bar rendering code, add a menu item:

```rust
if ui.button("Open Simulation").clicked() {
    commands.trigger(OpenSimulationEvent);
}
```

**Step 5: Run cargo check and manual test**

Run: `cargo check` Expected: PASS

Run: `cargo run` — manually verify: click Open Simulation, second window appears.

**Step 6: Commit**

```bash
git add src/editor_ui/surface.rs src/editor_ui/mod.rs src/editor_ui/systems.rs crates/hexorder-contracts/src/surface.rs
git commit -m "feat(editor_ui): open simulation window with multi-window support"
```

---

### Task 1.7: Simulation Window egui Context and Dock Layout

**Files:**

- Modify: `src/editor_ui/surface.rs` — add simulation dock system
- Modify: `src/editor_ui/components.rs` — add `SimulationDockState` resource

**Step 1: Add simulation dock state**

````rust
// Add to src/editor_ui/components.rs

/// Dock layout state for the simulation surface.
#[derive(Resource)]
pub(crate) struct SimulationDockState {
    pub(crate) dock_state: DockState<PanelKind>,
}

impl Default for SimulationDockState {
    fn default() -> Self {
        Self {
            dock_state: create_simulation_layout(),
        }
    }
}

/// Creates the simulation surface dock layout.
///
/// ```text
/// +-----------------------------------+
/// |            Viewport               |
/// +-----------------------------------+
/// | SimulationControls | CombatLog    |
/// +-----------------------------------+
/// ```
pub(crate) fn create_simulation_layout() -> DockState<PanelKind> {
    let mut state = DockState::new(vec![PanelKind::Viewport]);
    let tree = state.main_surface_mut();
    let root = egui_dock::NodeIndex::root();
    let [_top, _bottom] = tree.split_below(
        root,
        0.75,
        vec![PanelKind::SimulationControls, PanelKind::CombatLog],
    );
    state
}
````

**Step 2: Write the simulation dock rendering system**

This system runs on the secondary window's egui context. The exact scheduling depends on bevy_egui
0.39 multi-window API — check `docs/guides/bevy-egui.md` §8 for `EguiMultipassSchedule` usage.

```rust
// src/editor_ui/surface.rs

/// System that renders the simulation surface's egui dock in the secondary window.
/// Scheduled via EguiMultipassSchedule for the simulation window.
pub(super) fn simulation_dock_system(
    // Parameters TBD based on bevy_egui multi-window API
    // At minimum: EguiContext for the secondary window, SimulationDockState
) {
    // Render DockArea with SimulationDockState
    // Similar structure to editor_dock_system but with fewer panels
}
```

**Note:** The exact system parameters depend on how bevy_egui 0.39 exposes per-window `EguiContext`.
Read `docs/guides/bevy-egui.md` §8 before implementing. The key pattern is:

1. Query `EguiContext` filtered by the simulation window entity
2. Call `DockArea::show()` with `SimulationDockState`
3. Render only simulation-relevant panels in the tab viewer

**Step 3: Run tests and manual verification**

Run: `cargo check` Run: `cargo run` — verify simulation window shows a dock layout

**Step 4: Commit**

```bash
git add src/editor_ui/surface.rs src/editor_ui/components.rs
git commit -m "feat(editor_ui): add simulation surface dock layout and rendering"
```

---

### Task 1.8: Remove AppScreen::Play

**Files:**

- Modify: `crates/hexorder-contracts/src/persistence.rs` — remove `Play` variant from `AppScreen`
- Modify: `src/editor_ui/render_play.rs` — migrate play mode UI into simulation panels
- Modify: `src/editor_ui/systems.rs` — remove play-mode gating, gate on simulation surface
- Modify: `src/editor_ui/mod.rs` — remove `OnEnter(AppScreen::Play)` / `OnExit(AppScreen::Play)`
- Update any systems across the codebase that reference `AppScreen::Play`

**Step 1: Search for all AppScreen::Play references**

Run: `rg "AppScreen::Play" --type rust`

This will identify every file that needs updating.

**Step 2: Replace Play-mode gating with simulation-surface gating**

Systems that previously ran `in_state(AppScreen::Play)` should instead check:

```rust
fn simulation_surface_active(registry: Res<SurfaceRegistry>) -> bool {
    registry.has_active(SurfaceIntent::Simulation)
}
```

Use as a run condition: `.run_if(simulation_surface_active)`

**Step 3: Remove the `Play` variant**

In `crates/hexorder-contracts/src/persistence.rs`:

```rust
pub enum AppScreen {
    #[default]
    Launcher,
    Editor,
    // Play variant removed — simulation is now a surface within Editor
}
```

**Step 4: Fix all compilation errors**

Work through each file identified in step 1. Key changes:

- `render_play.rs`: Convert the play panel system into a simulation panel renderer that runs within
  the simulation dock system
- Systems with `in_state(AppScreen::Play)` → `run_if(simulation_surface_active)`
- Remove any `NextState::set(AppScreen::Play)` calls — replace with
  `commands.trigger(OpenSimulationEvent)`
- Update tests that reference `AppScreen::Play`

**Step 5: Run full test suite**

Run: `mise check` Expected: PASS

**Step 6: Commit**

```bash
git add -u
git commit -m "refactor(persistence): remove AppScreen::Play, gate on simulation surface"
```

---

### Task 1.9: Cross-Surface Selection Highlighting

**Files:**

- Modify: `src/editor_ui/surface.rs` — add cross-highlight system
- Modify: `src/editor_ui/mod.rs` — register observer

**Step 1: Write the failing test**

```rust
#[test]
fn cross_highlight_event_fires_on_hex_selection() {
    let mut app = App::new();
    app.insert_resource(SurfaceRegistry::default());
    app.insert_resource(SelectedHex::default());
    // ... setup surfaces ...
    // Verify that changing SelectedHex in one surface fires CrossSurfaceHighlightEvent
}
```

**Step 2: Implement the cross-highlight system**

Selection state (`SelectedHex`, `SelectedUnit`) is already shared via ECS resources. Both surfaces
read from the same resource, so selection automatically syncs. The `CrossSurfaceHighlightEvent` is
for additional visual emphasis (e.g., pulsing highlight).

```rust
/// System that fires cross-surface highlight events when selection changes.
pub(super) fn sync_cross_surface_highlights(
    selected_hex: Res<SelectedHex>,
    registry: Res<SurfaceRegistry>,
    mut commands: Commands,
) {
    if !selected_hex.is_changed() {
        return;
    }
    if let Some(pos) = selected_hex.0 {
        // Find the design surface ID (source)
        if let Some(design) = registry.find_by_intent(SurfaceIntent::Design).next() {
            commands.trigger(CrossSurfaceHighlightEvent {
                source_surface: design.id,
                target: HighlightTarget::Hex(pos),
            });
        }
    }
}
```

**Step 3: Run tests**

Run: `cargo test --lib editor_ui` Expected: PASS

**Step 4: Commit**

```bash
git add src/editor_ui/surface.rs src/editor_ui/mod.rs
git commit -m "feat(editor_ui): sync cross-surface selection highlighting"
```

---

### Task 1.10: Auto-Close Simulation on Window Close

**Files:**

- Modify: `src/editor_ui/surface.rs`

**Step 1: Write system to detect window close**

When the user closes the simulation OS window, Bevy despawns the `Window` entity. Detect this and
clean up the surface registry.

```rust
/// System that detects when a simulation window is closed by the OS and cleans up.
pub(super) fn detect_simulation_window_close(
    mut registry: ResMut<SurfaceRegistry>,
    windows: Query<Entity, With<Window>>,
    mut commands: Commands,
) {
    // Find simulation surfaces whose window entity no longer exists
    let window_entities: HashSet<Entity> = windows.iter().collect();
    let orphaned: Vec<SurfaceId> = registry
        .find_by_intent(SurfaceIntent::Simulation)
        .filter(|s| {
            matches!(s.rendering, RenderingTarget::Window(e) if !window_entities.contains(&e))
        })
        .map(|s| s.id)
        .collect();

    for id in orphaned {
        if let Some(_removed) = registry.remove(id) {
            commands.trigger(SurfaceClosedEvent { surface_id: id });
        }
    }
}
```

**Step 2: Register system**

Run in `Update` schedule with a run condition that checks if any simulation surface exists.

**Step 3: Run tests and manual verification**

Run: `cargo test --lib editor_ui` Run: `cargo run` — open simulation, close it via OS close button,
verify no crash

**Step 4: Commit**

```bash
git add src/editor_ui/surface.rs src/editor_ui/mod.rs
git commit -m "feat(editor_ui): detect simulation window close and clean up registry"
```

---

### Task 1.11: Architecture Test — Surface Contract Boundary

**Files:**

- Modify: `tests/architecture.rs`

**Step 1: Add boundary test**

Verify that `surface.rs` contract types are only imported through `hexorder_contracts::surface`, not
via internal paths.

```rust
#[test]
fn surface_types_imported_through_contracts() {
    // Follow existing architecture test patterns in tests/architecture.rs
    // Verify no src/ file imports from crates/hexorder-contracts/src/surface.rs directly
}
```

**Step 2: Run**

Run: `cargo test --test architecture` Expected: PASS

**Step 3: Commit**

```bash
git add tests/architecture.rs
git commit -m "test(project): add surface contract boundary architecture test"
```

---

## Phase 2: Simulation Integration (Task Outlines)

**Depends on:** Phase 1 + Cycles 16-18 (simulation runtime, combat, scenarios)

**Coordination note:** This phase wires the simulation engine (dice, CRT, phase sequencer) from
cycles 16-18 into the simulation surface. It may run alongside or after those cycles. Do not
duplicate simulation primitives — consume what cycles 16-18 build.

### Task 2.1: SimulationEngine System Set Wiring

- Modify: `src/editor_ui/surface.rs` — add `simulation_surface_active` run condition
- Modify: `crates/hexorder-simulation/src/lib.rs` — gate systems on `simulation_surface_active`
- Systems: `SimulationEngine` system set runs only when a Simulation-intent surface exists
- Test: open simulation surface → systems activate; close → systems deactivate

### Task 2.2: Turn/Phase Controls in Simulation Surface

- Modify: `src/editor_ui/surface.rs` — add simulation panel rendering for turn controls
- Render `SimulationControls` panel: "Advance Phase" button, current phase display, turn counter
- Reads `TurnState`, `TurnStructure` from shared World
- Triggers `PhaseAdvanceEvent` on click

### Task 2.3: Combat Initiation and Resolution Display

- Render `CombatLog` panel in simulation surface
- Display rolling log of combat events from `CombatResolvedEvent` observer
- Each entry shows attacker, defender, ratio, die roll, result
- Clicking an entry populates the Trace panel (Phase 7 dependency — stub for now)

### Task 2.4: Hot-Reload Verification

- No new code needed — hot-reload is automatic (systems read current resource values each frame)
- Test: edit CRT in design window while simulation is open, initiate combat, verify new table used
- Add integration test proving resource changes propagate within same frame

---

## Phase 3: Journal, Goals, and Annotations (Task Outlines)

**Depends on:** Phase 1

### Task 3.1: Experience Goal Contract Types

- Create: `crates/hexorder-contracts/src/experience.rs`
- Types: `ExperienceGoal`, `GoalRegistry`
- Register module in `lib.rs`
- Tests: construction, registry CRUD, tag linking

### Task 3.2: Journal Contract Types

- Create: `crates/hexorder-contracts/src/journal.rs`
- Types: `JournalEntry`, `JournalRegistry`, `Phase` enum
- Register module in `lib.rs`
- Tests: construction, chronological ordering, filtering

### Task 3.3: Annotation Contract Types

- Create: `crates/hexorder-contracts/src/annotation.rs`
- Types: `Annotation`, `AnnotationTarget`, `AnnotationRegistry`
- Register module in `lib.rs`
- Tests: construction, target matching

### Task 3.4: ExperienceGoals Panel

- Modify: `src/editor_ui/systems.rs` — add `PanelKind::ExperienceGoals` rendering
- Render: goal list, "Add Goal" button, inline text editing, tag-link chips
- Actions: `EditorAction::AddGoal`, `EditorAction::RemoveGoal`
- Test: adding a goal persists in GoalRegistry

### Task 3.5: Journal Panel

- Modify: `src/editor_ui/systems.rs` — add `PanelKind::Journal` rendering
- Render: chronological entries, "Add Entry" field, filter controls (phase, tag, search)
- Actions: `EditorAction::AddJournalEntry`
- Test: adding entry with auto-detected phase hint

### Task 3.6: Tag-Link Interaction Pattern

- Create: `src/editor_ui/tag_link.rs` — reusable tag-link widget
- egui widget: text field + tag autocomplete dropdown + clickable chips
- Used by: ExperienceGoals, Journal, Feedback panels
- Test: tag search, chip click navigation

### Task 3.7: Contextual Feedback (Right-Click → Add Note)

- Modify: `src/editor_ui/systems.rs` — add context menu on hex/unit right-click
- Right-click → "Add Note" → creates journal entry with tag-link to clicked element
- In simulation surface: also captures turn number and phase index
- Test: right-click hex, journal entry created with correct tag

### Task 3.8: Persistence — Format Version 7

- Modify: `crates/hexorder-contracts/src/persistence.rs` — bump `FORMAT_VERSION` to 7
- Add fields to `GameSystemFile`: `experience_goals`, `journal`, `annotations`
- All with `#[serde(default)]` for backward compatibility
- Modify: `crates/hexorder-persistence/src/storage.rs` — serialize/deserialize new fields
- Test: load v6 file (missing new fields) → defaults; save → v7; round-trip

---

## Phase 4: Phase-Aware Presets and Theme System (Task Outlines)

**Depends on:** Phase 1, Phase 3

### Task 4.1: Revised WorkspacePreset Enum

- Modify: `src/editor_ui/components.rs` — replace 4 presets with new 4: `Explore`, `MapAndUnits`,
  `Rules`, `Analysis`
- Each preset associated with a `Phase` enum value
- Update `as_id()`, `from_id()`, `Display`
- Migration: map old preset IDs to new ones

### Task 4.2: Phase Indicator in Status Bar

- Modify: `src/editor_ui/systems.rs` — in status bar rendering, show current phase
- Phase derived from active preset: Explore→Explore, MapAndUnits/Rules→Build, Analysis→Test
- Simulation surface always shows "Test"
- Styled with brand accent color

### Task 4.3: Theme Label Mapping

- Create: `src/editor_ui/theme_labels.rs`
- HashMap<&'static str, &'static str> mapping engineering→design language labels
- e.g., `"Panel"` → `"Instrument"`, `"Journal"` → `"Notebook"`
- Used by panel title rendering
- Initially hardcoded; future: load from theme plugin config

### Task 4.4: Updated Preset Layouts

- Modify: `src/editor_ui/components.rs` — update `create_*_layout()` functions
- `create_explore_layout()`: ExperienceGoals, MechanicReference, Journal, Viewport, Inspector
- `create_map_and_units_layout()`: Palette, Viewport, Inspector, MapGenerator, Selection
- `create_rules_layout()`: Design, Rules, Viewport, Inspector, Validation
- `create_analysis_layout()`: Journal, Validation, Inspector, Viewport

---

## Phase 5: Dependency View (Task Outlines)

**Depends on:** Phase 1, stable ontology

### Task 5.1: Dependency Edge Computation

- Create: `src/editor_ui/dependency.rs`
- Walk ontology: entity types → concept bindings → relations → constraints
- Walk mechanics: CRT → modifiers → terrain type filters
- Produce `Vec<DependencyEdge>` (computed, not persisted)
- Test: given known ontology, verify correct edges

### Task 5.2: Inspector "Connections" Section

- Modify: `src/editor_ui/systems.rs` — in Inspector rendering, add collapsible "Connections"
- Show "Affects" and "Affected by" lists with clickable entries
- Uses dependency edges filtered to selected element

### Task 5.3: DependencyGraph Panel

- Modify: `src/editor_ui/systems.rs` — add `PanelKind::DependencyGraph` rendering (add variant to
  `PanelKind` first)
- Node graph: entities, concepts, relations, constraints, CRT as nodes
- Edges with relationship labels
- Click node → select in Inspector
- Layout: force-directed (use egui canvas or a Rust graph layout crate)
- This is the most complex UI task — may need a dedicated egui graph widget

---

## Phase 6: Lens Filters (Task Outlines)

**Depends on:** Phase 1, Phase 2, grid overlay system

### Task 6.1: LensMode and ActiveLens Contracts

- Add to `crates/hexorder-contracts/src/surface.rs`: `LensMode` enum (Off, MovementCost, Influence,
  Stacking, Probability, GoalCoverage) `ActiveLens` resource (per-surface)

### Task 6.2: Lens Selector UI

- Per-surface toolbar dropdown: lens mode selector
- Keyboard shortcut to cycle lenses
- One active lens per surface

### Task 6.3: Movement Cost Lens

- Hex overlay: color by cost for selected unit type
- Green (cheap) → Red (expensive) → Black (impassable)
- Reads `MovementCostMatrix` and selected entity type

### Task 6.4: Influence/ZOC Lens

- Hex overlay: ZOC hexes colored by faction
- Reads `InfluenceRuleRegistry` and unit positions

### Task 6.5: Stacking Lens

- Hex overlay: unit count vs stacking limit
- Green (room) → Yellow (near) → Red (full)

### Task 6.6: Probability Lens

- Hex overlay: CRT outcome distribution for hexes in combat range
- Requires simulation engine (Phase 2)

### Task 6.7: Goal Coverage Lens

- Hex overlay: highlight hexes/units touched by mechanics linked to selected goal
- Requires experience goals (Phase 3)

---

## Phase 7: Causal Tracing (Task Outlines)

**Depends on:** Phase 2

### Task 7.1: TraceEntry and TraceStep Contracts

- Add to `crates/hexorder-contracts/src/surface.rs`: `TraceStep`, `TraceEntry`, `ActiveTrace`

### Task 7.2: Combat Trace Generation

- Modify simulation combat resolution to produce `TraceEntry`
- Each step: base ratio, modifier application, column shift, die roll, lookup
- Links to source mechanic TypeIds

### Task 7.3: Movement Trace Generation

- Extend `ValidMoveSet` blocked explanations to produce `TraceEntry`
- Each step: constraint name, why it blocked, source relation/constraint

### Task 7.4: Trace Panel Rendering

- Render `PanelKind::Trace` in simulation surface dock
- Show trace steps with indentation
- Each step clickable → navigate to source mechanic in design surface
- "Why?" button on combat log entries populates this panel

---

## Phase 8: Session Recording and Replay (Task Outlines)

**Depends on:** Phase 2, Phase 7

### Task 8.1: SessionAction and SessionRecord Contracts

- Add to `crates/hexorder-contracts/src/surface.rs`: `SessionAction`, `SessionRecord`,
  `SessionRecordingState`

### Task 8.2: Recording System

- Toggle in simulation surface: start/stop recording
- Observer captures `PhaseAdvancedEvent`, movement events, combat events
- Timestamps relative to session start
- Stores RNG seed

### Task 8.3: Replay System

- Load a `SessionRecord`, replay actions in sequence
- Same seed → same dice rolls (if rules unchanged)
- Replay controls: play, pause, step forward, step back

### Task 8.4: Divergence Detection

- During replay with modified rules: detect when outcomes differ
- Highlight divergence points
- Show side-by-side: original outcome vs. new outcome with traces for each

### Task 8.5: Persistence

- Add `session_records: Vec<SessionRecord>` to `GameSystemFile`
- "Max sessions" setting for file size management
- Delete old sessions UI in settings

---

## Dependency Summary

```
Phase 1: Surface Foundation          ← START HERE
  |
  +-- Phase 2: Simulation Integration (also needs cycles 16-18)
  |     |
  |     +-- Phase 6: Lens Filters
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

## Notes for Implementers

1. **Read the design document first**: `docs/plans/2026-03-07-design-experience-design.md`
2. **Read bevy-egui guide**: `docs/guides/bevy-egui.md` — especially §8 (multi-window), §9 (input
   passthrough), §16 (deprecations)
3. **Read bevy guide**: `docs/guides/bevy.md` — especially §3 (observers), §5 (system ordering)
4. **Shared target directory issue**: `CARGO_TARGET_DIR` is shared across worktrees. Run
   `cargo clean -p hexorder-contracts` if you see stale type errors from another worktree.
5. **Pre-commit hooks**: Commit messages must use scope from allowed list (see `lefthook.yml`).
   `contracts` for contract changes, `editor_ui` for plugin changes, `project` for docs.
6. **`DockTab` → `PanelKind` migration**: This is a breaking internal change. All layout persistence
   files (`dock_layout.ron`) will need re-creation after migration. Add a fallback that defaults to
   `create_default_dock_layout()` on deserialization failure.
7. **Multi-window Metal risk**: macOS Metal has known issues with multiple windows. The
   `MallocStackLogging=lite` workaround (#230) may be needed. Test on macOS early.
