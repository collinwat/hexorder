# Contract: cell

## Purpose

This contract is merged into the `game_system` contract. Cell types (CellTypeId, CellType,
CellTypeRegistry, CellData, ActiveCellType) are Game System definitions and live in
`src/contracts/game_system.rs`.

See `docs/contracts/game-system.md` for all type definitions.

## Rationale

Cell types are not independent of the Game System — they are definitions owned by it. Splitting them
into a separate contract would create an artificial boundary. The `game_system` contract is the
single source of truth for all Game System-owned types.

## Changelog

| Date       | Change                                           | Reason                                                                                                      |
| ---------- | ------------------------------------------------ | ----------------------------------------------------------------------------------------------------------- |
| 2026-02-08 | Created as redirect to game_system (as "vertex") | Cell types are Game System definitions                                                                      |
| 2026-02-09 | Renamed Vertex→Cell terminology                  | Cell is mathematically correct for N-dimensional grid elements; Vertex means hex corner in grid terminology |
