# Plugin Log: editor_ui

## Status: in-progress (0.9.0 visual polish)

## Decision Log

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

### Tests (5):

1. `editor_tool_defaults_to_select` — EditorTool default is Select
2. `editor_tool_variants_are_distinct` — Select != Paint
3. `editor_tool_resource_inserts_correctly` — resource works in ECS
4. `editor_state_defaults` — EditorState default values correct
5. `editor_state_resource_inserts_correctly` — resource works in ECS

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
