# Plugin: Export

## Summary

Provides an export pipeline for getting game system designs out of Hexorder. The first export target
is print-and-play PDF output — counter sheets and hex maps suitable for physical prototyping.

## Plugin

- Module: `src/export/`
- Plugin struct: `ExportPlugin`
- Schedule: none (export is triggered on-demand via observer events, not per-frame)

## Appetite

- **Size**: Small Batch (1-2 weeks)
- **Pitch**: #79

## Dependencies

- **Contracts consumed**: `game_system` (EntityTypeRegistry, EntityData, PropertyValue, EntityRole),
  `hex_grid` (HexPosition, HexGridConfig, HexTile)
- **Contracts produced**: none (export is plugin-internal; no shared types exposed)
- **Crate dependencies**: `printpdf` (PDF generation — rectangles, text, color fills)

## Scope

1. **Export plugin skeleton + trait architecture** — define `ExportTarget` trait, `ExportPlugin`
   struct, register in `main.rs`. The trait takes a game system snapshot and produces output bytes.
2. **Counter sheet PDF generation** — implement `PrintAndPlayExporter` for unit counter sheets.
   Counters show attack/defense/movement values and unit name. Grouped by side. Configurable counter
   size (1/2", 5/8", 3/4"). Letter/A4 paper.
3. **Hex map PDF generation** — render the current map as a flat hex grid with terrain coloring and
   hex coordinates. Single page (≤ ~200 hexes at 1/2" counter size on letter paper).
4. **Editor UI integration** — add Export menu action in the editor menu bar to trigger PDF
   generation. Save dialog for output path. Progress feedback.

## Success Criteria

- [ ] [SC-1] ExportTarget trait compiles and has at least one test verifying the interface
- [ ] [SC-2] Counter sheet PDF generates with correct counter count, readable text, and grouped by
      entity role
- [ ] [SC-3] Hex map PDF generates with correct hex count, terrain colors, and readable coordinates
- [ ] [SC-4] Export menu action triggers PDF generation and writes files to disk
- [ ] [SC-BUILD] `cargo build` succeeds with this plugin registered
- [ ] [SC-CLIPPY] `cargo clippy -- -D warnings` passes
- [ ] [SC-TEST] `cargo test` passes (all tests, not just this plugin's)
- [ ] [SC-BOUNDARY] No imports from other plugins' internals — all cross-plugin types come from
      `crate::contracts::`

## UAT Checklist

- [ ] [UAT-1] Launch app, create a game system with 2+ entity types (BoardPosition and Token),
      trigger Export → counter sheet PDF is generated with correct counter layout
- [ ] [UAT-2] Launch app, paint a hex map with terrain, trigger Export → hex map PDF shows correct
      terrain coloring and hex coordinates
- [ ] [UAT-3] Export menu item is accessible from the editor menu bar and shows save dialog

## Decomposition (for agent teams)

Solo plugin — no parallel decomposition needed.

## Constraints

- No `unsafe` code
- No LaTeX dependency
- Counter art is utilitarian (text and colored rectangles), not publication-ready
- Single-page hex maps only (multi-page tiling is deferred)
- PDF crate must handle basic layout: rectangles, text, color fills

## Open Questions

- printpdf vs genpdf — evaluate printpdf first (pitch recommendation), fall back if needed
- Font embedding — printpdf requires explicit font loading; need a bundled font or system font path

## Deferred Items

- Multi-page map tiling with alignment marks (#66 — future scope)
- VASSAL/TTS export (#65)
- Custom counter art/images (#66 — future scope)
- Rules summary PDF (#66 — future scope)
- Scenario setup overlay on map (#66 — future scope)
