# Design: Scope 4 — Tab Support (Pitch #135)

## Context

Scopes 1–3 established egui_dock evaluation (GO), four-zone native panel layout, and panel
decomposition into 4 independent systems. The current layout uses native egui panels (`SidePanel`,
`TopBottomPanel`) with hardcoded content per zone. Scope 4 replaces this with egui_dock's `DockArea`
for full drag-to-dock tab support.

## Architecture

Replace the 4 native panel systems with a single `DockArea` rendering system:

- **Single `DockArea`** renders the entire editor UI using `DockState<DockTab>`
- **`TabViewer` trait impl** dispatches rendering to `render_*` functions per tab
- **`DockTab` enum** expands from 4 variants to 9
- **Default layout** uses the 4-zone template (left/center/right/bottom) — tabs are draggable
  between zones
- **Menu bar** stays as native `TopBottomPanel::top` above the dock area
- **`apply_actions`** runs after `DockArea::show()` — same deferred action pattern as today

### DockTab Variants

| DockTab      | Default zone | Content                                              | Closeable |
| ------------ | ------------ | ---------------------------------------------------- | --------- |
| `Viewport`   | Center       | Empty/transparent — 3D scene shows through           | No        |
| `Palette`    | Left         | Workspace header, tool mode, Play, palettes          | Yes       |
| `Design`     | Left         | Sub-tabs: Types, Enums, Structs, Concepts, Relations | Yes       |
| `Rules`      | Left         | Sub-tabs: Constraints, Validation, Mechanics         | Yes       |
| `Inspector`  | Right        | Tile/unit inspector (placeholder)                    | Yes       |
| `Settings`   | Right        | Font size control                                    | Yes       |
| `Selection`  | Right        | Multi-selection summary                              | Yes       |
| `Validation` | Bottom       | Schema validation output                             | Yes       |

### Content Redistribution

- **Palette tab**: workspace header + tool mode selector + Play button + cell/unit palette
  (context-sensitive on tool mode). Extracted from left zone's top section.
- **Design tab**: reuses existing `render_tab_bar` + `OntologyTab` dispatch for Types, Enums,
  Structs, Concepts, Relations sub-tabs.
- **Rules tab**: reuses `OntologyTab` dispatch for Constraints, Validation, Mechanics sub-tabs.
- **Settings tab**: extracted from left zone's collapsing header → standalone dock tab.
- **Selection tab**: extracted from left zone bottom ("N tiles selected") → standalone dock tab.
- **Validation tab**: already in bottom zone, becomes draggable dock tab.
- **Inspector tab**: placeholder text (real query-based content is future work).

### Sub-Tab Routing

Design and Rules tabs both use the existing `active_tab: OntologyTab` field in `EditorState`. Each
dock tab renders a subset of the `OntologyTab` enum values. No conflict — only one dock tab renders
at a time.

## Input Passthrough

### Problem

`DockArea` covers the full window. `egui_wants_any_pointer_input()` always returns `true`, blocking
camera orbit and hex_grid clicks even when the pointer is over the Viewport tab.

### Solution

Custom run condition `pointer_over_ui_panel`:

1. After `DockArea::show()`, store the Viewport tab's screen rect in a resource (new field on
   `DockLayoutState` or a dedicated `ViewportRect` resource).
2. `pointer_over_ui_panel` checks: is cursor inside any non-Viewport area? If pointer is over the
   viewport rect → return `false` (allow game input). Otherwise → `true`.
3. Replace `egui_wants_any_pointer_input` in `hex_grid/mod.rs` (2 sites) and `camera/mod.rs` (2
   sites).
4. `egui_wants_any_keyboard_input` stays as-is — keyboard passthrough is unaffected.

### Viewport Rect Extraction

In `TabViewer::ui()` for the Viewport tab, capture `ui.max_rect()` and write it to the resource.
This runs every frame as part of egui_dock's rendering.

### Edge Case

Viewport tab cannot be closed (`TabViewer::closeable()` returns `false` for `DockTab::Viewport`). If
the rect is somehow unavailable, fall back to allowing all game input.

## ViewportMargins

With `DockArea` covering the full window, `available_rect()` is zero. Compute margins from the
Viewport tab's rect directly:

```
margins.left = viewport_rect.left()
margins.top = viewport_rect.top()
margins.right = window_width - viewport_rect.right()
margins.bottom = window_height - viewport_rect.bottom()
```

This is simpler and more accurate — margins reflect exactly where the 3D viewport is, regardless of
dock tab arrangement.

## Testing

Preserved: all 26 existing editor_ui tests (Default-based init unchanged).

New/updated tests:

- `dock_tab_variants_expanded` — all 9 variants are distinct
- `default_dock_layout_has_nine_tabs` — updated from 4 to 9
- `viewport_tab_is_not_closeable` — TabViewer returns false for Viewport
- `pointer_over_ui_panel_returns_false_for_viewport` — unit test the condition function

## Not in Scope

- Workspace presets (Scope 5)
- Layout persistence/serialization (Scope 6)
- Real inspector content (tile/unit queries) — stays placeholder
- Floating/detached panels (pitch no-go)
- Custom user-defined presets (pitch no-go)
