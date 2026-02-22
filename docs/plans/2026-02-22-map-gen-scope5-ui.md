# Map Generation UI — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan
> task-by-task.

**Goal:** Add a UI panel to the map_gen plugin with controls for seed, noise parameters, and a
"Generate" button — making heightmap generation usable from the editor.

**Architecture:** The map_gen plugin owns its own egui window system. The panel renders as a
floating egui window (not part of editor_ui's dock system) to respect plugin boundaries. The panel
reads/writes `MapGenParams` directly and inserts `GenerateMap` to trigger generation.

**Tech Stack:** Rust, Bevy 0.18, bevy_egui 0.39, egui widgets (DragValue, Slider, Button)

**Design doc:** `docs/plans/2026-02-21-map-gen-design.md`

---

### Task 1: Add MapGenPanelVisible resource and UI module

**Files:**

- Modify: `src/map_gen/components.rs`
- Create: `src/map_gen/ui.rs`
- Modify: `src/map_gen/mod.rs`

**Step 1: Add panel visibility resource to components.rs**

Add at the end of `src/map_gen/components.rs`:

```rust
/// Controls visibility of the map generation panel.
#[derive(Resource, Debug)]
pub struct MapGenPanelVisible(pub bool);

impl Default for MapGenPanelVisible {
    fn default() -> Self {
        Self(true)
    }
}
```

**Step 2: Create `src/map_gen/ui.rs` with the panel system**

```rust
//! Map generation UI panel rendered via egui.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use super::components::{GenerateMap, MapGenPanelVisible, MapGenParams};

/// Renders the map generation parameter panel as an egui window.
pub fn map_gen_panel(
    mut contexts: EguiContexts,
    mut params: ResMut<MapGenParams>,
    panel_visible: Res<MapGenPanelVisible>,
    generate: Option<Res<GenerateMap>>,
    mut commands: Commands,
) {
    if !panel_visible.0 {
        return;
    }

    let ctx = contexts.ctx_mut();
    let is_generating = generate.is_some();

    egui::Window::new("Map Generator")
        .default_width(260.0)
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            // Seed
            ui.horizontal(|ui| {
                ui.label("Seed:");
                ui.add(egui::DragValue::new(&mut params.seed).speed(1.0));
            });

            ui.add_space(4.0);

            // Noise parameters under a collapsible header
            egui::CollapsingHeader::new("Noise Parameters")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Octaves:");
                        let mut octaves_u32 = params.octaves as u32;
                        if ui
                            .add(
                                egui::DragValue::new(&mut octaves_u32)
                                    .speed(0.1)
                                    .range(1..=12),
                            )
                            .changed()
                        {
                            params.octaves = octaves_u32 as usize;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Frequency:");
                        ui.add(
                            egui::DragValue::new(&mut params.frequency)
                                .speed(0.001)
                                .range(0.001..=1.0)
                                .max_decimals(3),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Amplitude:");
                        ui.add(
                            egui::DragValue::new(&mut params.amplitude)
                                .speed(0.01)
                                .range(0.01..=5.0)
                                .max_decimals(2),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Lacunarity:");
                        ui.add(
                            egui::DragValue::new(&mut params.lacunarity)
                                .speed(0.01)
                                .range(1.0..=4.0)
                                .max_decimals(2),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Persistence:");
                        ui.add(
                            egui::DragValue::new(&mut params.persistence)
                                .speed(0.01)
                                .range(0.01..=1.0)
                                .max_decimals(2),
                        );
                    });
                });

            ui.add_space(8.0);

            // Reset to defaults
            if ui.button("Reset Defaults").clicked() {
                *params = MapGenParams::default();
            }

            ui.add_space(4.0);

            // Generate button (disabled while generation is in progress)
            ui.add_enabled_ui(!is_generating, |ui| {
                if ui
                    .button("Generate Map")
                    .on_hover_text("Generate terrain using current parameters")
                    .clicked()
                {
                    commands.insert_resource(GenerateMap);
                }
            });
        });
}
```

**Step 3: Register the UI module and system in mod.rs**

Update `src/map_gen/mod.rs`:

```rust
//! Procedural hex map generation plugin.
//!
//! Generates heightmap-based terrain using layered Perlin noise and
//! a configurable biome table that maps elevation ranges to cell types.

use bevy::prelude::*;

use crate::contracts::persistence::AppScreen;

mod biome;
mod components;
mod heightmap;
mod systems;
mod ui;

#[cfg(test)]
mod tests;

/// Plugin that provides procedural map generation.
#[derive(Debug)]
pub struct MapGenPlugin;

impl Plugin for MapGenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<components::MapGenParams>()
            .init_resource::<components::BiomeTable>()
            .init_resource::<components::MapGenPanelVisible>()
            .add_systems(
                Update,
                (
                    systems::run_generation,
                    ui::map_gen_panel,
                )
                    .run_if(in_state(AppScreen::Editor)),
            );
    }
}
```

**Step 4: Verify it compiles and runs**

Run: `cargo build`

Expected: Clean compilation.

**Step 5: Commit**

```bash
git add src/map_gen/components.rs src/map_gen/ui.rs src/map_gen/mod.rs
git commit -m "feat(map_gen): add generation parameter UI panel"
```

---

### Task 2: Run full checks and fix issues

**Files:**

- Possibly modify: any file with issues

**Step 1: Run clippy**

Run: `cargo clippy --all-targets`

Expected: Zero warnings. Fix any that appear.

**Step 2: Run full test suite**

Run: `cargo test`

Expected: All tests pass (existing + map_gen tests).

**Step 3: Run boundary and unwrap checks**

Run: `mise check:boundary && mise check:unwrap`

Expected: No violations.

**Step 4: Commit if any fixes were needed**

```bash
git add -A
git commit -m "fix(map_gen): resolve lint and test issues from UI scope"
```

---

### Task 3: Update spec, log, and post progress

**Files:**

- Modify: `docs/plugins/map-gen/spec.md`
- Modify: `docs/plugins/map-gen/log.md`

**Step 1: Update spec**

Mark SC-6 if applicable (editable after creation — should already work since we write EntityData
with no link back to the generator).

**Step 2: Update log**

Record the UI scope completion with test results and decision about egui window vs dock tab.

**Step 3: Post scope completion comment on pitch #102**

**Step 4: Commit**

```bash
git add docs/plugins/map-gen/spec.md docs/plugins/map-gen/log.md
git commit -m "docs(map_gen): update spec and log with scope 5 UI results"
```
