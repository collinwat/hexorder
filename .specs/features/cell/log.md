# Feature Log: cell

## Status: complete

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-02-09 | CellMaterials keyed by CellTypeId (HashMap) instead of enum | Cell types are dynamic, user-defined. HashMap enables runtime additions/removals. |
| 2026-02-09 | Default cell data uses first registry entry with empty properties map | Starter types have no property definitions, so default properties map is empty |
| 2026-02-09 | sync_cell_materials updates material colors in place via Assets<StandardMaterial> | In-place update means tiles referencing that handle auto-reflect the new color |
| 2026-02-09 | Coordinated retirement of terrain plugin — editor_ui updated simultaneously | Both terrain module and terrain contract removed; editor_ui switched to cell types |
| 2026-02-09 | Editor UI tests that referenced ActiveTerrain/TerrainType removed | Those tests tested terrain contract behavior, not editor_ui behavior. EditorTool tests retained. |
| 2026-02-09 | Moved `assign_default_cell_data` from Startup to Update | Startup ordering is not guaranteed across plugins; hex_grid tiles may not exist yet. `Without<CellData>` filter makes it self-healing in Update — assigns on first frame tiles exist, no-op thereafter. |
| 2026-02-10 | sync_cell_visuals now updates both MeshMaterial3d AND TileBaseMaterial | Hover/selection highlighting can restore the correct cell color by reading TileBaseMaterial instead of losing it on overlay |
| 2026-02-10 | Cell materials changed to unlit: true | Match hex tile materials which are also unlit, ensuring consistent visual appearance |
| 2026-02-10 | Added update_paint_preview system | Keeps PaintPreview resource in sync with ActiveCellType. Used by hex_grid for paint mode hover border. |

## Test Results

| Date | Command | Result | Notes |
|------|---------|--------|-------|
| 2026-02-09 | `cargo build` | PASS | Clean compilation with terrain retired |
| 2026-02-09 | `cargo clippy -- -D warnings` | PASS | Zero warnings |
| 2026-02-09 | `cargo test` | PASS | 46/46 tests pass (8 new cell tests, 3 editor_ui tests retained, 12 terrain tests retired) |

### Tests implemented (8):
1. `cell_materials_created_for_all_types` — CellMaterials has entry for each registry type
2. `assign_default_cell_data_adds_to_tiles` — all tiles get CellData referencing first type
3. `paint_cell_changes_tile_type` — painting changes tile's cell type ID
4. `paint_does_not_affect_other_tiles` — only clicked tile changes
5. `paint_skipped_in_select_mode` — no painting when EditorTool is Select
6. `sync_cell_visuals_updates_material` — material updates when CellData changes
7. `cell_materials_lookup_works` — CellMaterials::get finds by ID
8. `sync_cell_materials_adds_new_type` — adding registry type creates material

## Blockers

| Blocker | Waiting On | Raised | Resolved |
|---------|-----------|--------|----------|
| (none) | | | |

## Status Updates

| Date | Status | Notes |
|------|--------|-------|
| 2026-02-08 | speccing | Initial spec created for M2. Replaces M1 terrain plugin. |
| 2026-02-09 | complete | Plugin implemented, terrain retired, all tests pass, clippy clean |
| 2026-02-10 | complete | Post-M3 polish: TileBaseMaterial sync, paint preview, unlit materials |
