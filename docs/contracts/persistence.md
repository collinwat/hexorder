# Contract: Persistence

## Owner

`persistence` feature plugin (M5)

## Purpose

Types for saving and loading game system definitions and board state to `.hexorder` (RON) files.

## Types

### `GameSystemFile`

Top-level container for a saved game system + board state.

| Field            | Type                 | Description                                     |
| ---------------- | -------------------- | ----------------------------------------------- |
| `format_version` | `u32`                | File format version (migration), currently `3`  |
| `name`           | `String`             | Human-readable project name (v3+, default `""`) |
| `game_system`    | `GameSystem`         | Game system metadata                            |
| `entity_types`   | `EntityTypeRegistry` | All entity types                                |
| `enums`          | `EnumRegistry`       | Enum definitions (0.7.0)                        |
| `structs`        | `StructRegistry`     | Struct definitions (0.7.0)                      |
| `concepts`       | `ConceptRegistry`    | Concepts + bindings                             |
| `relations`      | `RelationRegistry`   | Relations                                       |
| `constraints`    | `ConstraintRegistry` | Constraints                                     |
| `map_radius`     | `u32`                | Hex grid radius                                 |
| `tiles`          | `Vec<TileSaveData>`  | Per-tile cell data                              |
| `units`          | `Vec<UnitSaveData>`  | Placed unit data                                |

### `TileSaveData`

Serialized form of a hex tile's cell data.

| Field            | Type                             | Description             |
| ---------------- | -------------------------------- | ----------------------- |
| `position`       | `HexPosition`                    | Hex coordinates         |
| `entity_type_id` | `TypeId`                         | Cell type               |
| `properties`     | `HashMap<TypeId, PropertyValue>` | Per-instance properties |

### `UnitSaveData`

Serialized form of a placed unit.

| Field            | Type                             | Description             |
| ---------------- | -------------------------------- | ----------------------- |
| `position`       | `HexPosition`                    | Hex coordinates         |
| `entity_type_id` | `TypeId`                         | Unit type               |
| `properties`     | `HashMap<TypeId, PropertyValue>` | Per-instance properties |

### `PersistenceError`

Error type for save/load operations.

| Variant              | Fields                     | Description                 |
| -------------------- | -------------------------- | --------------------------- |
| `Io`                 | `std::io::Error`           | File system error           |
| `Serialize`          | `ron::Error`               | RON serialization failure   |
| `Deserialize`        | `ron::error::SpannedError` | RON deserialization failure |
| `UnsupportedVersion` | `found: u32, max: u32`     | Unknown format version      |

### `AppScreen`

Application screen state.

| Variant    | Description                            |
| ---------- | -------------------------------------- |
| `Launcher` | Startup screen — new/open project      |
| `Editor`   | Main editor — all editing tools active |

### `Workspace`

Tool-level session state for the currently open project.

| Field       | Type              | Description                                       |
| ----------- | ----------------- | ------------------------------------------------- |
| `name`      | `String`          | Human-readable project name (display only)        |
| `file_path` | `Option<PathBuf>` | Path to last-saved file; `None` if unsaved        |
| `dirty`     | `bool`            | Whether project has unsaved changes (placeholder) |

### `PendingBoardLoad`

Temporary resource for deferred board state application after load.

| Field   | Type                | Description        |
| ------- | ------------------- | ------------------ |
| `tiles` | `Vec<TileSaveData>` | Tile data to apply |
| `units` | `Vec<UnitSaveData>` | Units to spawn     |

### `SaveRequestEvent`

Triggers a save operation.

| Field     | Type   | Description                          |
| --------- | ------ | ------------------------------------ |
| `save_as` | `bool` | If true, always show the file dialog |

### `LoadRequestEvent`

Triggers a load operation. No fields.

### `NewProjectEvent`

Triggers creation of a new empty project.

| Field  | Type     | Description                     |
| ------ | -------- | ------------------------------- |
| `name` | `String` | Human-readable name for project |

### `CloseProjectEvent`

Triggers close of the current project and return to the launcher. No fields.

## Consumed By

- `persistence` plugin — save/load systems
- `editor_ui` plugin — launcher screen, file menu

## Dependencies

- `game_system` contract — `GameSystem`, `EntityTypeRegistry`, `EnumRegistry`, `StructRegistry`,
  `TypeId`, `PropertyValue`
- `ontology` contract — `ConceptRegistry`, `RelationRegistry`, `ConstraintRegistry`
- `hex_grid` contract — `HexPosition`
