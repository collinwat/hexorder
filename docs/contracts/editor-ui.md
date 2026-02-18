# Contract: editor_ui

## Purpose

Defines the editor tool mode shared between the editor UI (producer) and any feature that needs to
check the current tool mode before acting (consumers like cell).

## Consumers

- cell (checks EditorTool before painting)
- unit (checks EditorTool before placing or interacting with units)
- camera (reads ViewportMargins for viewport centering)
- (any future feature that behaves differently based on tool mode)

## Producers

- editor_ui (inserts EditorTool resource, provides UI for switching modes; writes ViewportMargins
  each frame)

## Types

### Resources

```rust
/// The current editor tool mode. Other plugins (e.g., cell, unit) read this
/// to decide whether a click should select, paint, or place.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum EditorTool {
    /// Click to select hex tiles or units. Also handles unit movement.
    #[default]
    Select,
    /// Click to paint cell types onto hex tiles.
    Paint,
    /// Click to place unit tokens on hex tiles.
    Place,
}
```

```rust
/// Holds the material handle for the currently active paint color.
/// Updated by the cell plugin when the active cell type changes.
/// Read by hex_grid to show a paint preview on hover in Paint mode.
#[derive(Resource, Debug, Default)]
pub struct PaintPreview {
    pub material: Option<Handle<StandardMaterial>>,
}
```

```rust
/// Pixel-space margins consumed by editor UI panels. Updated by the editor_ui
/// plugin each frame so other plugins (e.g., camera) can account for panel
/// layout when centering or fitting content.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct ViewportMargins {
    /// Width in logical pixels consumed by the left side panel.
    pub left: f32,
    /// Height in logical pixels consumed by the top menu bar.
    pub top: f32,
    /// Width in logical pixels consumed by the right side panel (e.g., debug inspector).
    pub right: f32,
}
```

## Invariants

- `EditorTool` is inserted during plugin build by the editor_ui plugin
- Default value is `EditorTool::Select`
- Only the editor_ui plugin should write to this resource (other plugins read it)
- `PaintPreview` is initialized by the cell plugin
- Only the cell plugin should write to `PaintPreview` (hex_grid reads it)
- `ViewportMargins` is initialized by the editor_ui plugin and updated each frame after panel
  rendering
- Only the editor_ui plugin should write to `ViewportMargins` (camera reads it)

## Changelog

| Date       | Change                                                  | Reason                                                                |
| ---------- | ------------------------------------------------------- | --------------------------------------------------------------------- |
| 2026-02-08 | Initial definition                                      | Promoted from editor_ui internals to fix contract boundary violations |
| 2026-02-09 | Updated consumer references from "terrain" to "vertex"  | M2 terrain retirement                                                 |
| 2026-02-09 | Renamed vertex→cell in consumer references and comments | Cell terminology adoption                                             |
| 2026-02-09 | Added Place variant, added unit as consumer             | M3 — unit placement tool mode                                         |
| 2026-02-10 | Added PaintPreview resource                             | Paint mode hover preview for ring border overlay                      |
| 2026-02-17 | Added ViewportMargins resource                          | Dynamic viewport centering for camera plugin                          |
