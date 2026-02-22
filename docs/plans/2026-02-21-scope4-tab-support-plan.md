# Scope 4: Tab Support — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan
> task-by-task.

**Goal:** Replace native egui panels with egui_dock DockArea for full drag-to-dock tab support with
9 content tabs across 4 zones.

**Architecture:** A single `editor_dock_system` renders the menu bar (native TopBottomPanel) then a
DockArea (via `DockState<DockTab>`). A `TabViewer` impl dispatches each tab to its `render_*`
function. A `ViewportRect` resource tracks the viewport tab's screen rect for input passthrough and
margin calculation.

**Tech Stack:** Rust, Bevy 0.18, bevy_egui 0.39, egui_dock 0.18

**Design doc:** `docs/plans/2026-02-21-scope4-tab-support-design.md`

---

## Task 1: Expand DockTab enum and update default layout

**Files:**

- Modify: `src/editor_ui/components.rs`

**Step 1: Update DockTab enum**

Add 5 new variants to `DockTab`. Keep existing 4, add `Palette`, `Design`, `Rules`, `Settings`,
`Selection`. The existing `ToolPalette` variant becomes `Palette` (rename). The result is 8 named
tabs plus `Viewport`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum DockTab {
    Viewport,
    Palette,
    Design,
    Rules,
    Inspector,
    Settings,
    Selection,
    Validation,
}
```

Update the `Display` impl to match:

```rust
impl std::fmt::Display for DockTab {
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
        }
    }
}
```

**Step 2: Update `create_default_dock_layout`**

Build a 4-zone layout with tabs distributed:

```rust
pub(crate) fn create_default_dock_layout() -> DockState<DockTab> {
    let mut state = DockState::new(vec![DockTab::Viewport]);
    let tree = state.main_surface_mut();
    let root = egui_dock::NodeIndex::root();

    // Left: Palette + Design + Rules tabs get 20% width.
    let [center, _left] = tree.split_left(
        root,
        0.20,
        vec![DockTab::Palette, DockTab::Design, DockTab::Rules],
    );

    // Right: Inspector + Settings + Selection tabs get 25% of remaining width.
    let [center, _right] = tree.split_right(
        center,
        0.75,
        vec![DockTab::Inspector, DockTab::Settings, DockTab::Selection],
    );

    // Bottom: Validation gets 15% of center height.
    let [_viewport, _bottom] = tree.split_below(center, 0.85, vec![DockTab::Validation]);

    state
}
```

**Step 3: Add `ViewportRect` resource**

Add a resource to store the viewport tab's screen rect each frame:

```rust
/// Screen rect of the Viewport dock tab, updated each frame.
/// Used by the custom input passthrough condition and viewport margin calculation.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub(crate) struct ViewportRect(pub(crate) Option<bevy_egui::egui::Rect>);
```

**Step 4: Run `cargo build` to verify compilation**

The `ToolPalette` variant is removed so existing references in tests will fail — that's expected and
fixed in Task 2.

**Step 5: Commit**

```
feat(editor_ui): expand DockTab to 8 content tabs with ViewportRect resource
```

---

## Task 2: Update tests for new DockTab variants

**Files:**

- Modify: `src/editor_ui/tests.rs`

**Step 1: Update `dock_tab_variants_are_distinct` test**

Replace the 4-variant distinctness test with an 8-variant version covering all `DockTab` values.

**Step 2: Update `dock_layout_creates_four_zones` test**

Rename to `dock_layout_creates_default_layout` and update assertion: the default layout should have
8 tabs total (Palette+Design+Rules in left, Viewport in center, Inspector+Settings+Selection in
right, Validation in bottom).

**Step 3: Add `viewport_tab_is_not_closeable` test**

Test that `DockTab::Viewport` is not closeable and all other tabs are. This tests the `is_closeable`
logic that will be implemented in the `TabViewer` — for now, test the function directly:

```rust
#[test]
fn viewport_tab_is_not_closeable() {
    assert!(!DockTab::Viewport.is_closeable());
    assert!(DockTab::Palette.is_closeable());
    assert!(DockTab::Inspector.is_closeable());
}
```

This requires adding a method `is_closeable()` on `DockTab` itself (in components.rs):

```rust
impl DockTab {
    pub(crate) fn is_closeable(&self) -> bool {
        !matches!(self, Self::Viewport)
    }
}
```

**Step 4: Run `cargo test --lib editor_ui`**

All editor_ui tests should pass.

**Step 5: Commit**

```
test(editor_ui): update dock tab tests for 8-tab layout (Scope 4)
```

---

## Task 3: Implement TabViewer and editor_dock_system

This is the core task. Replace the 4 separate zone systems with a single system that renders the
menu bar then delegates to `DockArea`.

**Files:**

- Modify: `src/editor_ui/systems.rs`
- Modify: `src/editor_ui/mod.rs`

**Step 1: Create the `EditorDockViewer` struct**

Define a struct that holds mutable references to all the data needed by the render functions. It
implements `egui_dock::TabViewer`. Place this near the top of `systems.rs` (after imports).

The struct borrows from the system's parameters for the duration of the `DockArea::show()` call:

```rust
struct EditorDockViewer<'a> {
    editor_tool: &'a mut EditorTool,
    editor_state: &'a mut EditorState,
    selection: &'a mut SelectionParams<'_>,  // check lifetime
    project: &'a ProjectParams<'_>,
    registry: &'a mut EntityTypeRegistry,
    enum_registry: &'a mut EnumRegistry,
    struct_registry: &'a mut StructRegistry,
    ontology: &'a mut OntologyParams<'_>,
    mechanics: &'a mut MechanicsParams<'_>,
    next_state: &'a mut NextState<AppScreen>,
    validation: &'a SchemaValidation,
    actions: &'a mut Vec<EditorAction>,
    viewport_rect: &'a mut ViewportRect,
}
```

Note: The exact lifetime annotations may need adjustment based on what the borrow checker accepts.
The `SelectionParams`, `OntologyParams`, `MechanicsParams`, and `ProjectParams` are `SystemParam`
bundles with `'w` lifetimes — the viewer struct borrows from their fields.

**Step 2: Implement `TabViewer` for `EditorDockViewer`**

```rust
impl egui_dock::TabViewer for EditorDockViewer<'_> {
    type Tab = DockTab;

    fn title(&mut self, tab: &mut DockTab) -> egui::WidgetText {
        tab.to_string().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut DockTab) {
        match tab {
            DockTab::Viewport => {
                // Store the viewport rect for input passthrough.
                self.viewport_rect.0 = Some(ui.max_rect());
            }
            DockTab::Palette => {
                render_workspace_header(
                    ui,
                    &self.project.workspace,
                    &self.project.game_system,
                );
                if self.editor_state.toolbar_visible {
                    render_tool_mode(ui, self.editor_tool);
                }
                if ui
                    .button(
                        egui::RichText::new("\u{25B6} Play")
                            .strong()
                            .color(BrandTheme::SUCCESS),
                    )
                    .on_hover_text("Enter play mode to test turns and combat")
                    .clicked()
                {
                    self.next_state.set(AppScreen::Play);
                }
                ui.separator();
                if *self.editor_tool == EditorTool::Paint {
                    render_cell_palette(ui, self.registry, &mut self.selection.active_board);
                }
                if *self.editor_tool == EditorTool::Place {
                    render_unit_palette(ui, self.registry, &mut self.selection.active_token);
                }
            }
            DockTab::Design => {
                // Sub-tab bar for Design tabs only.
                render_design_tab_bar(ui, self.editor_state);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    match self.editor_state.active_tab {
                        OntologyTab::Types => {
                            render_entity_type_editor(
                                ui, self.registry, self.editor_state,
                                self.actions, self.enum_registry, self.struct_registry,
                            );
                        }
                        OntologyTab::Enums => {
                            render_enums_tab(
                                ui, self.enum_registry, self.editor_state, self.actions,
                            );
                        }
                        OntologyTab::Structs => {
                            render_structs_tab(
                                ui, self.struct_registry, self.enum_registry,
                                self.editor_state, self.actions,
                            );
                        }
                        OntologyTab::Concepts => {
                            render_concepts_tab(
                                ui, &mut self.ontology.concept_registry, self.registry,
                                self.editor_state, self.actions,
                            );
                        }
                        OntologyTab::Relations => {
                            render_relations_tab(
                                ui, &mut self.ontology.relation_registry,
                                &self.ontology.concept_registry,
                                self.editor_state, self.actions,
                            );
                        }
                        // If user had a Rules sub-tab selected, show Types as fallback.
                        _ => {
                            self.editor_state.active_tab = OntologyTab::Types;
                            render_entity_type_editor(
                                ui, self.registry, self.editor_state,
                                self.actions, self.enum_registry, self.struct_registry,
                            );
                        }
                    }
                });
            }
            DockTab::Rules => {
                // Sub-tab bar for Rules tabs only.
                render_rules_tab_bar(ui, self.editor_state);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    match self.editor_state.active_tab {
                        OntologyTab::Constraints => {
                            render_constraints_tab(
                                ui, &mut self.ontology.constraint_registry,
                                &self.ontology.concept_registry,
                                self.editor_state, self.actions,
                            );
                        }
                        OntologyTab::Validation => {
                            render_validation_tab(ui, &self.ontology.schema_validation);
                        }
                        OntologyTab::Mechanics => {
                            render_mechanics_tab(
                                ui, &self.mechanics.turn_structure,
                                &self.mechanics.combat_results_table,
                                &self.mechanics.combat_modifiers,
                                self.editor_state, self.actions,
                            );
                        }
                        // If user had a Design sub-tab selected, show Constraints as fallback.
                        _ => {
                            self.editor_state.active_tab = OntologyTab::Constraints;
                            render_constraints_tab(
                                ui, &mut self.ontology.constraint_registry,
                                &self.ontology.concept_registry,
                                self.editor_state, self.actions,
                            );
                        }
                    }
                });
            }
            DockTab::Inspector => {
                ui.label(
                    egui::RichText::new("Inspector")
                        .strong()
                        .color(BrandTheme::ACCENT_AMBER),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new("Tile/unit inspector (coming soon)")
                        .color(BrandTheme::TEXT_SECONDARY),
                );
            }
            DockTab::Settings => {
                ui.label(
                    egui::RichText::new("Settings")
                        .strong()
                        .color(BrandTheme::ACCENT_AMBER),
                );
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Font size:");
                    if ui.button(" \u{2212} ").clicked()
                        && self.editor_state.font_size_base > 10.0
                    {
                        self.editor_state.font_size_base -= 1.0;
                    }
                    ui.monospace(format!("{}", self.editor_state.font_size_base as i32));
                    if ui.button(" + ").clicked()
                        && self.editor_state.font_size_base < 24.0
                    {
                        self.editor_state.font_size_base += 1.0;
                    }
                });
            }
            DockTab::Selection => {
                ui.label(
                    egui::RichText::new("Selection")
                        .strong()
                        .color(BrandTheme::ACCENT_AMBER),
                );
                ui.separator();
                let count = self.selection.multi.entities.len();
                if count > 0 {
                    ui.label(
                        egui::RichText::new(format!("{count} tiles selected"))
                            .color(BrandTheme::ACCENT_TEAL),
                    );
                } else {
                    ui.label(
                        egui::RichText::new("No selection")
                            .color(BrandTheme::TEXT_SECONDARY),
                    );
                }
            }
            DockTab::Validation => {
                render_validation_tab(ui, self.validation);
            }
        }
    }

    fn is_closeable(&self, tab: &DockTab) -> bool {
        tab.is_closeable()
    }

    fn clear_background(&self, tab: &DockTab) -> bool {
        !matches!(tab, DockTab::Viewport)
    }

    fn allowed_in_windows(&self, _tab: &mut DockTab) -> bool {
        false  // No floating windows (pitch no-go).
    }
}
```

**Step 3: Create `render_design_tab_bar` and `render_rules_tab_bar`**

Split the existing `render_tab_bar` into two functions. The Design bar shows Types, Enums, Structs,
Concepts, Relations. The Rules bar shows Constraints, Validation, Mechanics.

```rust
fn render_design_tab_bar(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    ui.horizontal_wrapped(|ui| {
        for tab in [
            OntologyTab::Types,
            OntologyTab::Enums,
            OntologyTab::Structs,
            OntologyTab::Concepts,
            OntologyTab::Relations,
        ] {
            let label = match tab {
                OntologyTab::Types => "Types",
                OntologyTab::Enums => "Enums",
                OntologyTab::Structs => "Structs",
                OntologyTab::Concepts => "Concepts",
                OntologyTab::Relations => "Relations",
                _ => unreachable!(),
            };
            if ui
                .selectable_label(editor_state.active_tab == tab, label)
                .clicked()
            {
                editor_state.active_tab = tab;
            }
        }
    });
    ui.separator();
}

fn render_rules_tab_bar(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    ui.horizontal_wrapped(|ui| {
        for tab in [
            OntologyTab::Constraints,
            OntologyTab::Validation,
            OntologyTab::Mechanics,
        ] {
            let label = match tab {
                OntologyTab::Constraints => "Constraints",
                OntologyTab::Validation => "Validation",
                OntologyTab::Mechanics => "Mechanics",
                _ => unreachable!(),
            };
            if ui
                .selectable_label(editor_state.active_tab == tab, label)
                .clicked()
            {
                editor_state.active_tab = tab;
            }
        }
    });
    ui.separator();
}
```

**Step 4: Create `editor_dock_system`**

This replaces the 4 separate zone systems (menu, validation, tool palette, inspector). It renders
the menu bar as a native TopBottomPanel, then the DockArea for everything else.

```rust
#[allow(clippy::too_many_arguments)]
pub fn editor_dock_system(
    mut contexts: EguiContexts,
    mut editor_tool: ResMut<EditorTool>,
    mut selection: SelectionParams,
    mut editor_state: ResMut<EditorState>,
    project: ProjectParams,
    mut registry: ResMut<EntityTypeRegistry>,
    mut enum_registry: ResMut<EnumRegistry>,
    mut struct_registry: ResMut<StructRegistry>,
    mut tile_data_query: Query<&mut EntityData, Without<UnitInstance>>,
    mut commands: Commands,
    mut ontology: OntologyParams,
    mut mechanics: MechanicsParams,
    mut next_state: ResMut<NextState<AppScreen>>,
    mut dock_layout: ResMut<DockLayoutState>,
    mut viewport_rect: ResMut<ViewportRect>,
    validation: Res<SchemaValidation>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // Menu bar as native TopBottomPanel (above dock area).
    egui::TopBottomPanel::top("editor_menu_bar").show(ctx, |ui| {
        // ... same menu bar content as current editor_menu_system ...
    });

    // About panel (modal, renders over everything).
    render_about_panel(ctx, &mut editor_state);

    let mut actions: Vec<EditorAction> = Vec::new();

    // DockArea for all tabbed content.
    let mut viewer = EditorDockViewer {
        editor_tool: &mut editor_tool,
        editor_state: &mut editor_state,
        selection: &mut selection,
        project: &project,
        registry: &mut registry,
        enum_registry: &mut enum_registry,
        struct_registry: &mut struct_registry,
        ontology: &mut ontology,
        mechanics: &mut mechanics,
        next_state: &mut next_state,
        validation: &validation,
        actions: &mut actions,
        viewport_rect: &mut viewport_rect,
    };

    // Configure dock area style to match brand theme.
    let mut style = egui_dock::Style::from_egui(ctx.style().as_ref());
    style.tab_bar.bg_fill = BrandTheme::BG_DEEP;
    style.tab.tab_body.bg_fill = BrandTheme::BG_PANEL;

    DockArea::new(&mut dock_layout.dock_state)
        .style(style)
        .draggable_tabs(true)
        .show_close_buttons(true)
        .show_leaf_close_all_buttons(false)
        .show_leaf_collapse_buttons(false)
        .show(ctx, &mut viewer);

    // Apply deferred actions.
    apply_actions(
        actions,
        &mut registry,
        &mut enum_registry,
        &mut struct_registry,
        &mut tile_data_query,
        &mut selection.active_board,
        &mut selection.active_token,
        &mut selection.selected_unit,
        &editor_state,
        &mut commands,
        &mut ontology.concept_registry,
        &mut ontology.relation_registry,
        &mut ontology.constraint_registry,
        &mut mechanics.turn_structure,
        &mut mechanics.combat_results_table,
        &mut mechanics.combat_modifiers,
    );
}
```

**Step 5: Update `mod.rs` to register the new system**

Replace the 4-system chain with the single `editor_dock_system`:

```rust
// Before (remove):
(
    systems::editor_menu_system,
    systems::editor_validation_system,
    systems::editor_tool_palette_system,
    systems::editor_inspector_system,
    systems::update_viewport_margins,
)
    .chain()
    .run_if(in_state(AppScreen::Editor)),

// After:
(
    systems::editor_dock_system,
    systems::update_viewport_margins,
)
    .chain()
    .run_if(in_state(AppScreen::Editor)),
```

Also register the `ViewportRect` resource: `app.init_resource::<components::ViewportRect>();`

The `#[cfg(feature = "inspector")]` variant needs the same update (with `debug_inspector_panel`
chained after `editor_dock_system` if needed, or folded into the dock as a tab).

**Step 6: Run `cargo build`**

Fix any borrow checker issues with the `EditorDockViewer` struct. The main risk is lifetime
annotations on SystemParam bundles — may need to destructure the bundles before passing fields to
the viewer.

**Step 7: Run `cargo test`**

All tests should pass. Some render tests in `ui_tests.rs` may need updates if they depend on the old
panel structure.

**Step 8: Commit**

```
feat(editor_ui): replace native panels with DockArea tab system (Scope 4)
```

---

## Task 4: Update ViewportMargins to use ViewportRect

**Files:**

- Modify: `src/editor_ui/systems.rs`

**Step 1: Rewrite `update_viewport_margins`**

Replace the `available_rect()` approach with viewport rect-based calculation:

```rust
pub fn update_viewport_margins(
    mut contexts: EguiContexts,
    mut margins: ResMut<ViewportMargins>,
    viewport_rect: Res<ViewportRect>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let Some(rect) = viewport_rect.0 else {
        return;
    };
    let screen = ctx.input(bevy_egui::egui::InputState::viewport_rect);
    margins.left = rect.left();
    margins.top = rect.top();
    margins.right = screen.right() - rect.right();
    margins.bottom = screen.bottom() - rect.bottom();
}
```

**Step 2: Run `cargo build && cargo test`**

**Step 3: Commit**

```
refactor(editor_ui): compute viewport margins from DockArea viewport rect
```

---

## Task 5: Custom input passthrough condition

**Files:**

- Modify: `src/editor_ui/systems.rs` (add the condition function)
- Modify: `src/editor_ui/mod.rs` (make it public)
- Modify: `src/hex_grid/mod.rs` (replace run condition)
- Modify: `src/camera/mod.rs` (replace run condition)

**Step 1: Add `pointer_over_ui_panel` function**

```rust
/// Returns `true` when the pointer is over a non-viewport UI panel.
/// Replacement for `egui_wants_any_pointer_input` which always returns true
/// when DockArea covers the full window.
pub fn pointer_over_ui_panel(
    contexts: EguiContexts,
    viewport_rect: Res<ViewportRect>,
) -> bool {
    let Some(vp_rect) = viewport_rect.0 else {
        // No viewport rect yet — allow game input (conservative).
        return false;
    };
    let Ok(ctx) = contexts.ctx_mut() else {
        return false;
    };
    let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) else {
        return false;
    };
    // If pointer is inside the viewport rect, game systems should run.
    !vp_rect.contains(pos)
}
```

Note: `EguiContexts` param here must be non-mutable — check if `ctx()` (immutable) is available, or
use a different approach. The function is a Bevy run condition so it takes system params. If
`EguiContexts` requires `&mut`, use `Res<bevy_egui::EguiContext>` or read pointer position from
Bevy's `Window` resource instead.

Alternative using Bevy's window cursor:

```rust
pub fn pointer_over_ui_panel(
    viewport_rect: Res<ViewportRect>,
    windows: Query<&Window, With<PrimaryWindow>>,
) -> bool {
    let Some(vp_rect) = viewport_rect.0 else {
        return false;
    };
    let Ok(window) = windows.single() else {
        return false;
    };
    let Some(cursor) = window.cursor_position() else {
        return false;
    };
    // Bevy cursor is (0,0) at top-left, Y increases downward — same as egui.
    let pos = bevy_egui::egui::Pos2::new(cursor.x, cursor.y);
    !vp_rect.contains(pos)
}
```

This avoids borrowing `EguiContexts` in a run condition.

**Step 2: Replace `egui_wants_any_pointer_input` in `hex_grid/mod.rs`**

Change the import and 2 run condition sites:

```rust
// Before:
use bevy_egui::input::egui_wants_any_pointer_input;
systems::update_hover.run_if(not(egui_wants_any_pointer_input)),
systems::handle_click.run_if(not(egui_wants_any_pointer_input)),

// After:
use crate::editor_ui::pointer_over_ui_panel;
systems::update_hover.run_if(not(pointer_over_ui_panel)),
systems::handle_click.run_if(not(pointer_over_ui_panel)),
```

**Step 3: Replace `egui_wants_any_pointer_input` in `camera/mod.rs`**

Same pattern, 2 sites:

```rust
// Before:
use bevy_egui::input::egui_wants_any_pointer_input;
.run_if(not(egui_wants_any_pointer_input)),

// After:
use crate::editor_ui::pointer_over_ui_panel;
.run_if(not(pointer_over_ui_panel)),
```

**Step 4: Run `cargo build && cargo test`**

Verify no boundary violations: `mise check:boundary`. The `pointer_over_ui_panel` function is in
`editor_ui` but consumed by `hex_grid` and `camera`. This is cross-plugin usage — it should be
exposed through contracts or made a public function in editor_ui. Check if this violates boundary
rules. If it does, add a `pointer_over_ui_panel` run condition to `src/contracts/editor_ui.rs`
instead.

**Step 5: Commit**

```
feat(editor_ui): add custom input passthrough for DockArea viewport (Scope 4)
```

---

## Task 6: Style the dock to match brand theme

**Files:**

- Modify: `src/editor_ui/systems.rs`

**Step 1: Refine dock style**

In `editor_dock_system`, expand the style configuration:

```rust
let mut style = egui_dock::Style::from_egui(ctx.style().as_ref());
// Tab bar background matches deep background.
style.tab_bar.bg_fill = BrandTheme::BG_DEEP;
// Tab body background matches panel fill.
style.tab.tab_body.bg_fill = BrandTheme::BG_PANEL;
// Active tab text.
style.tab.focused.text_color = BrandTheme::TEXT_PRIMARY;
style.tab.active.text_color = BrandTheme::TEXT_PRIMARY;
style.tab.active.bg_fill = BrandTheme::BG_PANEL;
style.tab.focused.bg_fill = BrandTheme::BG_PANEL;
// Inactive tab text.
style.tab.inactive.text_color = BrandTheme::TEXT_SECONDARY;
style.tab.inactive.bg_fill = BrandTheme::BG_DEEP;
style.tab.hovered.bg_fill = BrandTheme::BG_SURFACE;
style.tab.hovered.text_color = BrandTheme::TEXT_PRIMARY;
// Separator.
style.separator.color_idle = BrandTheme::BORDER_SUBTLE;
style.separator.color_hovered = BrandTheme::ACCENT_TEAL;
style.separator.color_dragged = BrandTheme::ACCENT_TEAL;
// Overlay (drag-to-dock indicators).
style.overlay.selection_color = egui::Color32::from_rgba_premultiplied(0, 92, 128, 80);
```

**Step 2: Run `cargo build`, verify visually with `cargo run`**

**Step 3: Commit**

```
style(editor_ui): apply brand theme to dock tab bar and separators
```

---

## Task 7: Final verification

**Step 1: Run full check suite**

```bash
mise check
```

This runs fmt, clippy, test, deny, typos, taplo, boundary, unwrap.

**Step 2: Run boundary check explicitly**

```bash
mise check:boundary
```

The `pointer_over_ui_panel` cross-plugin usage may be flagged. If so, move it to contracts or expose
it as a public module-level function that the boundary checker allows.

**Step 3: Manual visual test**

```bash
cargo run
```

Verify:

- 4-zone layout with tabs visible
- Drag a tab from left to right zone — it moves
- Viewport shows 3D scene, hex grid clicks work
- Camera orbit/pan works over viewport
- Menu bar (File, Help) works
- Each tab renders its content correctly
- Viewport tab cannot be closed
- Resize window — layout adjusts

**Step 4: Commit any remaining fixes**

**Step 5: Post progress comment on pitch issue**

```bash
gh issue comment 135 --body "Scope 4 complete (commit <SHA>): Full egui_dock DockArea with 8 content tabs, drag-to-dock, custom viewport input passthrough, and brand-themed styling. Viewport margins now derived from dock tab rect."
```
