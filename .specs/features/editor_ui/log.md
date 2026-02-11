# Feature Log: editor_ui

## Status: complete (M2)

## Decision Log

### 2026-02-08 — bevy_egui for M1 editor UI
**Decision**: Use bevy_egui for all editor UI. Single side panel with tool mode, terrain palette, and tile info.
**Rationale**: Fastest path to interactive editor UI. egui has direct ECS access, well-supported in Bevy ecosystem.

### 2026-02-08 — bevy_egui 0.39 API adaptations
**Decision**: `EguiPlugin::default()`, `ctx_mut()` returns `Result`, `rect_stroke()` takes 4 args with StrokeKind, systems in `EguiPrimaryContextPass`.

### 2026-02-08 — Left panel at 200px
**Decision**: Left panel. Follows convention of tools-on-left in design applications.

### 2026-02-09 — M2: Dark theme via Visuals::dark() customization
**Decision**: Start from `egui::Visuals::dark()`, customize panel fills (gray 25), widget states, selection highlight (teal). Monospace font for all body/button text.
**Rationale**: System/monospace fonts give the editor a professional tool feel. Dark background contrasts well with the 3D viewport.

### 2026-02-09 — M2: Deferred action pattern for mutations
**Decision**: Create/delete cell types and add/remove properties are captured as `EditorAction` enum variants inside the egui closure, then applied after the closure returns.
**Rationale**: Multi-pass safety — egui may re-run closures. Side effects inside closures can cause double-execution. In-place edits (name, color, property values) are idempotent and safe inside closures.

### 2026-02-09 — M2: EditorState resource for persistent UI state
**Decision**: Feature-local `EditorState` resource stores text buffers and selection state for the cell type editor (new type name/color, new property name/type/options).
**Rationale**: Egui immediate-mode widgets need persistent `String` buffers. A single resource is simpler than per-widget state management.

### 2026-02-09 — M2: Inspector panel at bottom of left panel
**Decision**: Inspector is a collapsible section at the bottom of the existing left panel, not a separate right panel.
**Rationale**: Keeps the UI compact for M2. Can be moved to a right panel later if needed.

### 2026-02-09 — M2: configure_theme uses Local<bool> guard
**Decision**: Theme configuration runs in `EguiPrimaryContextPass` with a `Local<bool>` flag to ensure it only executes once, rather than running in `Startup` where the egui context might not be available.
**Rationale**: `EguiPrimaryContextPass` guarantees the context exists. Startup may run before camera spawn.

### 2026-02-09 — M2: Inline enum definition creation
**Decision**: When adding an Enum property, the user types comma-separated options inline. An `EnumDefinition` is created automatically and stored in the registry.
**Rationale**: Simplest UX for M2. Separate enum management UI can be added later.

### 2026-02-09 — M3: enable_absorb_bevy_input_system required for text input
**Context**: Text fields (`text_edit_singleline`) in the editor panel did not respond to keyboard input. Clicking into the field showed a cursor but typing produced no characters.
**Decision**: Enable `EguiGlobalSettings::enable_absorb_bevy_input_system = true` in EditorUiPlugin::build().
**Rationale**: Run conditions (`egui_wants_any_keyboard_input`) only prevent our custom systems from running. Bevy's internal input systems still consume keyboard events before egui processes them. The absorb system clears Bevy's input buffers when egui has focus, allowing egui text fields to receive keystrokes.
**Lesson**: Run conditions and absorb serve different purposes. Run conditions guard game systems; absorb guards against Bevy internals. Both are needed when the UI has text input.

## Test Results

| Date | Command | Result | Notes |
|------|---------|--------|-------|
| 2026-02-09 | `cargo build` | PASS | Clean compilation |
| 2026-02-09 | `cargo clippy -- -D warnings` | PASS | Zero warnings |
| 2026-02-09 | `cargo test` | PASS | 48/48 tests pass (5 editor_ui tests) |

### Tests (5):
1. `editor_tool_defaults_to_select` — EditorTool default is Select
2. `editor_tool_variants_are_distinct` — Select != Paint
3. `editor_tool_resource_inserts_correctly` — resource works in ECS
4. `editor_state_defaults` — EditorState default values correct
5. `editor_state_resource_inserts_correctly` — resource works in ECS

## Blockers

| Blocker | Waiting On | Raised | Resolved |
|---------|-----------|--------|----------|
| (none) | | | |

## Status Updates

| Date | Status | Notes |
|------|--------|-------|
| 2026-02-08 | speccing | Initial spec created |
| 2026-02-08 | complete | M1 plugin implemented |
| 2026-02-08 | speccing | M2 evolution: dark theme, cell type editor, inspector panel |
| 2026-02-09 | complete | M2 evolution implemented: dark theme, cell palette, type editor, property editors, inspector, game system info |
