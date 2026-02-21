# Plugin Log: editor_ui

## Status: building (0.11.0 dockable panels — pitch #135)

## Decision Log

### 2026-02-20 — 0.11.0: Scope 1 — egui_dock evaluation prototype (#135)

**Result: GO** — all three unknowns resolved favorably.

| #   | Unknown               | Result | Evidence                                                                                                                                                                                                              |
| --- | --------------------- | ------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | Compilation           | PASS   | `cargo build` clean, `cargo deny check` passes — no version splits. egui_dock 0.18 pulls 4 new crates (duplicate, egui_dock, heck, proc-macro2-diagnostics).                                                          |
| 2   | Viewport transparency | PASS   | `clear_background(false)` + `SidePanel::left` with `Frame::NONE` — 3D hex grid visible through viewport tab. User confirmed visually.                                                                                 |
| 3   | Floating suppression  | PASS   | `draggable_tabs(false)` + `show_close_buttons(false)` + `show_leaf_close_all_buttons(false)` + `show_leaf_collapse_buttons(false)` — no close buttons, no tab dragging, no floating windows. User confirmed visually. |

**Additional finding — input passthrough**: Full-width `SidePanel::left` causes
`egui_wants_any_pointer_input()` to always return true, blocking hex_grid and camera input systems.
3D scene is visible but not interactive. **Workaround for Scope 2**: custom run condition that
checks pointer position against viewport tab rect from `DockLayoutState`, or restructure layout to
exclude viewport from egui panel coverage.

**Workaround — CentralPanel state transition bug**: `CentralPanel::default()` and `egui::Area` with
full-screen coverage do not visually render in the editor after `AppScreen::Launcher → Editor` state
transition (egui rendering loop continues but screen does not update). `SidePanel::left` with
`exact_width(available_width)` works correctly. Root cause undiagnosed — likely egui internal
repaint optimization or bevy_egui frame caching. Launcher changed from `CentralPanel` to
`egui::Area` + `layer_painter` background to avoid ID collision.

**Implementation decisions**:

- **DockTab enum**: 4 variants (Viewport, ToolPalette, Inspector, Validation). Minimal for
  prototype; full decomposition (entity_type_panel, ontology_panel, etc.) deferred to Scope 3.
- **DockLayoutState resource**: Wraps `egui_dock::DockState<DockTab>`. Manual Debug impl (DockState
  doesn't derive Debug). Default impl calls `create_default_dock_layout()`.
- **Four-zone layout**: Left 20% (ToolPalette) | Center ~60% (Viewport) | Right ~20% (Inspector) |
  Bottom ~12% (Validation). Created via `split_left` → `split_right` → `split_below`.
- **DockParams SystemParam**: Bundles `ResMut<DockLayoutState>`. To stay within Bevy's 16-param
  limit, moved `selected_hex` into `SelectionParams` (logically correct — it's selection state).
- **EditorDockViewer**: Implements `egui_dock::TabViewer`. ToolPalette renders full sidebar content.
  Inspector shows placeholder (Query lifetime complexity deferred to Scope 3). Validation renders
  `render_validation_tab`. Viewport is empty/transparent.
- **Inspector tab placeholder**: Storing `Query` references in the TabViewer struct creates complex
  lifetime issues (`Query<'w, 's, ...>` lifetimes tied to ECS world borrow). Scope 3 will solve this
  by either pre-extracting data or restructuring the rendering pipeline.
- **`render_inspector`/`render_unit_inspector`**: Marked `#[allow(dead_code)]` — still needed for
  Scope 3 inspector tab migration.

**Verification**:

- `cargo build` — clean, zero warnings
- `cargo deny check` — passes (no version splits)
- `cargo test` — 285/285 pass (3 new dock tests: variants distinct, four zones, resource init)
- `cargo clippy --all-targets` — zero warnings
- `mise check:unwrap` — no unwrap in production code
- `mise check:boundary` — no cross-plugin imports

### 2026-02-20 — 0.11.0: Kickoff — Dockable panel architecture (#135)

**Pitch**: Replace monolithic 280px sidebar with four-zone dockable layout
(Left/Center/Right/Bottom) using egui_dock, decompose `editor_panel_system` into independent panel
systems, add 4 workspace presets (Cmd+1–4), and persist active preset + panel visibility.

**Research consumed**:

- **Design Tool Interface Patterns** (wiki): Validates four-zone layout as universal template (Maya,
  Unity, Photoshop, Fusion 360 all converge on Left/Center/Right/Bottom). Confirms "Viewport
  Primacy" tenet — viewport always largest, always present. Validates 4 fixed presets as Fusion
  360-style constraint appropriate for domain-specific tool. Adobe icon-collapse pattern noted but
  out of scope.
- **UI Architecture Survey** (wiki): Recommends staying on egui (Option E / Phase 1). egui_dock
  provides DockArea with tabs/splits/drag-to-dock. Warns against building custom UI framework at
  this team size.

**First piece**: Scope 1 — egui_dock evaluation (throwaway prototype).

- **Core**: Entire architecture depends on whether egui_dock works
- **Small**: Designed as throwaway evaluation, not production code
- **Novel**: Three unknowns — egui_dock maturity with Bevy 0.18, 3D viewport in docked region, panel
  registration API

**Initial observations**:

- `systems.rs` is 4,462 lines with 22 render functions coupled through one system's parameter list
- `ViewportMargins` contract will need to evolve from fixed sidebar margins to dynamic zone margins
- 23 existing tests provide regression baseline for decomposition in Scope 3
- Pitch #134 (Build Discipline) running in parallel — should land first per delivery order

### 2026-02-18 — 0.10.0: Editor QoL — 7 scopes (#121)

**Scopes delivered**: (1) Toast notifications, (2) configurable font size, (3) multi-selection
system, (4) grid overlay toggle, (5) fullscreen toggle, (6) About panel + Help menu, (7) viewport
discoverability hints.

**Key decisions**:

- **Toast system**: Single-slot design (no stacking). `ToastEvent` is a contract type so any plugin
  can trigger toasts. Observer pattern in `editor_ui` catches events and writes into `ToastState`.
  2.5-second auto-dismiss via `Time::delta_secs()`.
- **Font size**: Added `font_size_base` field to `EditorState` (range 10–24, default 15). Applied
  via scale factor in `configure_theme`. No persistence — resets on restart. Acceptable for small
  batch appetite.
- **Multi-selection**: `Selection` resource with `HashSet<Entity>` added to contracts. Shift+click
  toggles individual tiles; Cmd+A selects all. Teal ring indicators spawned as standalone entities
  (avoided Bevy 0.18 `Parent→ChildOf` rename by not using parent-child hierarchy). `Selection` added
  to `SelectionParams` SystemParam bundle to stay under 16-param limit.
- **Grid overlay**: Camera `world_to_viewport` projects tile positions to screen. Bevy 0.18 returns
  `Result<Vec2, ViewportConversionError>` not `Option<Vec2>`. Uses `layer_painter` at
  `Order::Foreground` for text rendering.
- **About panel**: `render_about_panel` is a helper function called from panel systems (not a
  separate system), so no extra system params needed.
- **First-run hints**: `layer_painter` overlay with dark backdrop, dismissed on any click. Uses
  `content_rect()` not deprecated `screen_rect()`.

**Lessons**:

- `doc_markdown` clippy lint catches unbackticked identifiers in doc comments — happened 3 times
  across scopes. Always backtick type names like `` `egui::Window` `` in doc comments.
- `StatesPlugin` must be added explicitly when using `insert_state` in test apps (not included in
  `MinimalPlugins`).

### 2026-02-16 — 0.9.0: BrandTheme struct for named color constants

**Decision**: Introduce `BrandTheme` as a plain struct with `const` associated constants in
`components.rs`. Not a Bevy Resource — consumed at compile time by `configure_theme` and render
functions. **Rationale**: Zero runtime overhead, no system parameter slot consumed. Provides
namespaced vocabulary (`BrandTheme::ACCENT_AMBER`) for all 17 brand palette colors.

### 2026-02-16 — 0.9.0: Fonts from Monospace to Proportional

**Decision**: Switch Heading, Body, Small, Button TextStyles from `FontFamily::Monospace` to
`FontFamily::Proportional`. Add explicit `Monospace` TextStyle entry. Add `.monospace()` to
coordinate displays, IDs, and version strings. **Rationale**: Brand doc specifies proportional for
UI text, monospace for data values. System sans-serif (SF Pro on macOS) gives the editor a more
polished feel.

### 2026-02-16 — 0.9.0: fg_stroke text color hierarchy

**Decision**: Set explicit `fg_stroke` colors in Visuals: noninteractive = TEXT_PRIMARY (224),
inactive = TEXT_SECONDARY (128), hovered/active/open = TEXT_PRIMARY. Do not override disabled
fg_stroke. **Rationale**: Creates visual hierarchy — body text is brighter than egui defaults,
inactive controls are dimmer. Egui's disabled opacity handling is sufficient without override.

### 2026-02-08 — bevy_egui for M1 editor UI

**Decision**: Use bevy_egui for all editor UI. Single side panel with tool mode, terrain palette,
and tile info. **Rationale**: Fastest path to interactive editor UI. egui has direct ECS access,
well-supported in Bevy ecosystem.

### 2026-02-08 — bevy_egui 0.39 API adaptations

**Decision**: `EguiPlugin::default()`, `ctx_mut()` returns `Result`, `rect_stroke()` takes 4 args
with StrokeKind, systems in `EguiPrimaryContextPass`.

### 2026-02-08 — Left panel at 200px

**Decision**: Left panel. Follows convention of tools-on-left in design applications.

### 2026-02-09 — M2: Dark theme via Visuals::dark() customization

**Decision**: Start from `egui::Visuals::dark()`, customize panel fills (gray 25), widget states,
selection highlight (teal). Monospace font for all body/button text. **Rationale**: System/monospace
fonts give the editor a professional tool feel. Dark background contrasts well with the 3D viewport.

### 2026-02-09 — M2: Deferred action pattern for mutations

**Decision**: Create/delete cell types and add/remove properties are captured as `EditorAction` enum
variants inside the egui closure, then applied after the closure returns. **Rationale**: Multi-pass
safety — egui may re-run closures. Side effects inside closures can cause double-execution. In-place
edits (name, color, property values) are idempotent and safe inside closures.

### 2026-02-09 — M2: EditorState resource for persistent UI state

**Decision**: Feature-local `EditorState` resource stores text buffers and selection state for the
cell type editor (new type name/color, new property name/type/options). **Rationale**: Egui
immediate-mode widgets need persistent `String` buffers. A single resource is simpler than
per-widget state management.

### 2026-02-09 — M2: Inspector panel at bottom of left panel

**Decision**: Inspector is a collapsible section at the bottom of the existing left panel, not a
separate right panel. **Rationale**: Keeps the UI compact for M2. Can be moved to a right panel
later if needed.

### 2026-02-09 — M2: configure_theme uses Local<bool> guard

**Decision**: Theme configuration runs in `EguiPrimaryContextPass` with a `Local<bool>` flag to
ensure it only executes once, rather than running in `Startup` where the egui context might not be
available. **Rationale**: `EguiPrimaryContextPass` guarantees the context exists. Startup may run
before camera spawn.

### 2026-02-09 — M2: Inline enum definition creation

**Decision**: When adding an Enum property, the user types comma-separated options inline. An
`EnumDefinition` is created automatically and stored in the registry. **Rationale**: Simplest UX for
M2. Separate enum management UI can be added later.

### 2026-02-09 — M3: enable_absorb_bevy_input_system required for text input

**Context**: Text fields (`text_edit_singleline`) in the editor panel did not respond to keyboard
input. Clicking into the field showed a cursor but typing produced no characters. **Decision**:
Enable `EguiGlobalSettings::enable_absorb_bevy_input_system = true` in EditorUiPlugin::build().
**Rationale**: Run conditions (`egui_wants_any_keyboard_input`) only prevent our custom systems from
running. Bevy's internal input systems still consume keyboard events before egui processes them. The
absorb system clears Bevy's input buffers when egui has focus, allowing egui text fields to receive
keystrokes. **Lesson**: Run conditions and absorb serve different purposes. Run conditions guard
game systems; absorb guards against Bevy internals. Both are needed when the UI has text input.

### 2026-02-12 -- M4: Ontology UI panels

**Decision**: Add tabbed layout with 5 tabs (Types, Concepts, Relations, Constraints, Validation) to
the editor sidebar. Each tab renders a dedicated panel for its ontology domain.

**Key changes**:

- `EditorState` extended with `OntologyTab` and form state for concepts, relations, constraints.
- `OntologyParams` SystemParam bundle created to keep system param count under Bevy's 16-param
  limit.
- `EntityTypeRegistry` changed from `Option<ResMut<>>` to `ResMut<>` since `GameSystemPlugin` always
  inserts it.
- `GameSystem` changed from `Option<Res<>>` to `Res<>` since it's always present.
- `render_game_system_info` simplified (no Option unwrapping).
- New `EditorAction` variants for concept, relation, and constraint CRUD operations.
- Brand palette extended with success green (#509850 / `from_rgb(80, 152, 80)`) for the "Schema
  Valid" indicator.
- `editor_panel_system` uses `.chain()` instead of `.after(bare_fn)` for system ordering (required
  when system has many params).

**Rationale**: Tabbed layout keeps the sidebar manageable. The ontology panels follow the same
deferred-action pattern as the existing type editor. `OntologyParams` bundle avoids hitting Bevy's
16-parameter system limit.

### 2026-02-12 -- M4: OntologyParams SystemParam bundle

**Context**: Adding 4 new system parameters (3 ontology registries + schema validation) pushed
`editor_panel_system` to 17 parameters, exceeding Bevy 0.18's 16-parameter limit for `IntoSystem`.
**Decision**: Create `OntologyParams` as a `#[derive(SystemParam)]` bundle in `components.rs` that
groups `ConceptRegistry`, `RelationRegistry`, `ConstraintRegistry`, and `SchemaValidation`. The
EditorUiPlugin also calls `init_resource` for all four types to guarantee they exist even without
the OntologyPlugin. **Rationale**: SystemParam derive is the idiomatic Bevy solution for reducing
parameter counts. Dual `init_resource` calls are safe (no-op if resource already exists).

## Test Results

| Date       | Command                       | Result | Notes                                |
| ---------- | ----------------------------- | ------ | ------------------------------------ |
| 2026-02-09 | `cargo build`                 | PASS   | Clean compilation                    |
| 2026-02-09 | `cargo clippy -- -D warnings` | PASS   | Zero warnings                        |
| 2026-02-09 | `cargo test`                  | PASS   | 48/48 tests pass (5 editor_ui tests) |
| 2026-02-12 | `cargo build`                 | PASS   | Clean compilation                    |
| 2026-02-12 | `cargo clippy -- -D warnings` | PASS   | Zero warnings                        |
| 2026-02-12 | `cargo test`                  | PASS   | 90/90 tests pass (5 editor_ui tests) |
| 2026-02-16 | `mise check`                  | PASS   | All checks pass                      |
| 2026-02-16 | `cargo test`                  | PASS   | 167/167 tests pass                   |
| 2026-02-18 | `cargo clippy --all-targets`  | PASS   | Zero warnings                        |
| 2026-02-18 | `cargo test`                  | PASS   | 258/258 tests pass (23 editor_ui)    |
| 2026-02-20 | `cargo build`                 | PASS   | Clean compilation, zero warnings     |
| 2026-02-20 | `cargo deny check`            | PASS   | No version splits from egui_dock     |
| 2026-02-20 | `cargo test`                  | PASS   | 285/285 tests pass (26 editor_ui)    |
| 2026-02-20 | `cargo clippy --all-targets`  | PASS   | Zero warnings                        |
| 2026-02-20 | `mise check:unwrap`           | PASS   | No unwrap in production code         |
| 2026-02-20 | `mise check:boundary`         | PASS   | No cross-plugin imports              |

### Tests (26):

1. `editor_tool_defaults_to_select` — EditorTool default is Select
2. `editor_tool_variants_are_distinct` — Select != Paint
3. `editor_tool_resource_inserts_correctly` — resource works in ECS
4. `editor_state_defaults` — EditorState default values correct
5. `editor_state_resource_inserts_correctly` — resource works in ECS
6. `ontology_tab_default_is_types` — OntologyTab default is Types
7. `ontology_tab_variants_are_distinct` — tab variants differ
8. `toast_state_defaults_to_none` — no active toast on init
9. `toast_kind_variants_are_distinct` — Success != Error != Info
10. `toast_event_observer_populates_toast_state` — event → toast state
11. `toast_event_replaces_previous_toast` — new toast replaces old
12. `editor_state_font_size_defaults_to_15` — font size default 15.0
13. `selection_defaults_to_empty` — empty selection on init
14. `select_all_command_selects_all_hex_tiles` — Cmd+A selects all tiles
15. `delete_command_clears_multi_selection` — bulk delete empties set
16. `delete_command_falls_back_to_selected_unit` — single delete fallback
17. `grid_overlay_defaults_to_hidden` — overlay off by default
18. `toggle_grid_overlay_command_flips_visibility` — G toggles overlay
19. `editor_state_about_panel_defaults_hidden` — about panel hidden
20. `about_command_toggles_about_panel` — help.about toggles panel
21. `editor_state_first_run_not_seen_by_default` — hints not dismissed
22. `toggle_inspector_command_flips_visibility` — Cmd+I toggles inspector
23. `toggle_toolbar_command_flips_visibility` — Cmd+T toggles toolbar
24. `dock_tab_variants_are_distinct` — all DockTab variants differ
25. `dock_layout_creates_four_zones` — default layout produces 4 tabs
26. `dock_layout_state_resource_inserts_correctly` — resource initializes in ECS with 4 tabs

## Blockers

| Blocker | Waiting On | Raised | Resolved |
| ------- | ---------- | ------ | -------- |
| (none)  |            |        |          |

## Status Updates

| Date       | Status   | Notes                                                                                                           |
| ---------- | -------- | --------------------------------------------------------------------------------------------------------------- |
| 2026-02-08 | speccing | Initial spec created                                                                                            |
| 2026-02-08 | complete | M1 plugin implemented                                                                                           |
| 2026-02-08 | speccing | M2 evolution: dark theme, cell type editor, inspector panel                                                     |
| 2026-02-09 | complete | M2 evolution implemented: dark theme, cell palette, type editor, property editors, inspector, game system info  |
| 2026-02-12 | complete | M4 ontology UI: tabbed layout, concepts, relations, constraints, validation panels. All 90 tests pass.          |
| 2026-02-16 | building | 0.9.0 visual polish: BrandTheme, color audit, amber accents, font change, launcher restyle. 167/167 tests pass. |
| 2026-02-18 | complete | 0.10.0 editor QoL: all 7 scopes shipped. 258/258 tests pass.                                                    |
| 2026-02-20 | building | 0.11.0 Scope 1: egui_dock evaluation complete. GO decision. All 3 unknowns resolved. 285/285 tests pass.        |
