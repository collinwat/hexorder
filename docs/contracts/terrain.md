# Contract: terrain — RETIRED

> **Status: RETIRED as of M2.** Replaced by `game_system` contract (CellType, CellData,
> CellTypeRegistry, ActiveCellType). Code removed from `src/contracts/`. Kept for historical
> reference.

## Changelog

| Date       | Change              | Reason                                                                          |
| ---------- | ------------------- | ------------------------------------------------------------------------------- |
| 2026-02-08 | Initial definition  | M1 terrain painting feature                                                     |
| 2026-02-08 | Added ActiveTerrain | Promoted from terrain internals to fix contract boundary violations             |
| 2026-02-09 | RETIRED             | M2 replaced fixed terrain types with dynamic cell types in game_system contract |
